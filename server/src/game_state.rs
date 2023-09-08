use std::{ collections::HashMap, sync::Arc };

use rand::Rng;
use serde_json::json;
use tokio::sync::{ Mutex, mpsc::Sender };

use crate::{
    card::{ Card, Value },
    deck::Deck,
    player::{ Player, SerializablePlayer },
    playerpool::PlayerPool,
    websocket::create_websocket_message,
};

pub struct GameState {
    pub id: usize,
    deck: Deck,
    discard_pile: Vec<Card>,

    player_to_play: usize,
    direction: i8, // 1 for clockwise, -1 for counter-clockwise
    pub round_in_progress: bool,
    pub is_waiting_for_players: bool,
    pub game_player_pool: PlayerPool,
}

impl GameState {
    pub fn new(id: usize, player_pool: PlayerPool) -> Self {
        let mut deck = Deck::new();
        deck.shuffle();

        let discard_pile = vec![deck.draw().unwrap()]; // Draw the initial card
        let direction = 1;

        Self {
            id: id,
            player_to_play: 0,
            deck,
            discard_pile,
            direction,
            round_in_progress: false,
            is_waiting_for_players: true,
            game_player_pool: player_pool,
        }
    }

    // In your GameState struct
    pub async fn update_players(&self) {
        println!("game state starting update_players");
        let players_data = self.game_player_pool.connections
            .iter()
            .map(|p| p.player.to_serializable())
            .collect::<Vec<SerializablePlayer>>();

        println!("game state {} sending status update to: {} players", self.id, players_data.len());
        //convert players_data to json string
        let players_data_json = serde_json::to_string(&players_data).unwrap();
        self.send_update("update_players", &players_data_json).await;
        self.update_game_state().await;
    }

    pub async fn update_player(&self, player: &Player) {
        let player_data_json = serialize_player_data(player);
        let message = create_websocket_message("update_player", &player_data_json);
        self.game_player_pool.send_message(&player, message).await;
    }

    //function to let players receive an update about the game state via the pool connection
    pub async fn update_game_state(&self) {
        //build a json object with the game status details
        let info_object =
            json!({
            "id": self.id,
            "round_in_progress": self.round_in_progress,
            "player_to_play": self.player_to_play,
            "direction": self.direction,
            "discard_pile": self.discard_pile,
            "deck_size": self.deck.cards.len(),
            "player_count": self.game_player_pool.connections.len(),
        });

        let game_state_data_json = serde_json::to_string(&info_object).unwrap();
        let message = create_websocket_message("update_game_state", &game_state_data_json);
        self.game_player_pool.broadcast_message(message).await;
    }

    pub async fn play_card(&mut self, player_id: usize, card: Card) -> Result<(), &'static str> {
        println!("Player {} attempting to play card {:?}", player_id, card);
        self.validate_card_play(player_id, &card)?;

        // Step 1: Find the position of the card in the player's hand
        let pos_option = {
            let player = &self.game_player_pool.get_player_by_id(player_id).unwrap();
            player.hand.iter().position(|c| *c == card)
        };

        // Check if the card is in the player's hand
        if let Some(pos) = pos_option {
            self.apply_card_effect(&card);
            let mut player = self.game_player_pool.get_player_by_id(player_id).unwrap();
            player.hand.remove(pos);

            // Add the card to the discard pile
            self.discard_pile.push(card);

            // Check if the player has won
            if let Some(_winner_id) = self.check_winner() {
                self.end_round();
                return Ok(());
            }

            // Move to the next turn
            self.next_turn().await;

            Ok(())
        } else {
            Err("Card not in hand")
        }
    }

    pub async fn next_turn(&mut self) {
        //increment the current turn
        let next_player = self.get_next_player();
        let your_turn_json =
            json!({
                "player_id": next_player.id,
                "message": "your turn!"
            }).to_string();
        let message = create_websocket_message("your_turn", &your_turn_json);
        self.game_player_pool.send_message(&next_player, message).await;
    }

    pub async fn apply_card_effect(&mut self, card: &Card) {
        match card.value {
            Value::Skip => self.next_turn().await,
            Value::Reverse => {
                self.direction *= -1;
                self.next_turn().await;
            }
            Value::DrawTwo => self.draw_cards(self.next_player_id(), 2),
            Value::Wild => {} // Handle Wild card (e.g., allow player to choose color)
            Value::WildDrawFour => {
                self.draw_cards(self.next_player_id(), 4);
                // Handle Wild Draw Four (e.g., allow player to choose color)
            }
            _ => {}
        }
    }

    // Method to put the game into a "waiting" state
    pub fn go_into_waiting_state(&mut self) {
        self.round_in_progress = false;
        // Don't clear the players; just wait for more to join
    }

    pub fn draw_cards(&mut self, player_id: usize, count: usize) {
        let mut player = self.game_player_pool.get_player_by_id(player_id).unwrap();
        player.hand.extend(self.deck.draw_n(count));
    }

    pub fn check_winner(&self) -> Option<usize> {
        for entry in &self.game_player_pool.connections {
            if entry.player.hand.is_empty() {
                return Some(entry.player.id);
            }
        }
        None
    }

    pub fn next_player_id(&self) -> usize {
        let mut next_player = self.player_to_play;
        next_player =
            (next_player + (self.direction as usize)) % self.game_player_pool.connections.len();
        next_player
    }

    pub fn draw_card(&mut self, player_id: usize) -> Result<(), &'static str> {
        if self.player_to_play != player_id {
            return Err("Not your turn");
        }

        if self.deck.is_empty() {
            self.shuffle_discard_into_deck();
        }

        let mut player = self.game_player_pool.get_player_by_id(player_id).unwrap();
        if let Some(card) = self.deck.draw() {
            player.hand.push(card);
            Ok(())
        } else {
            Err("Deck is empty")
        }
    }
    pub fn shuffle_discard_into_deck(&mut self) {
        let top_card = self.discard_pile.pop().unwrap();
        self.deck.cards.extend(self.discard_pile.drain(..));
        self.deck.shuffle();
        self.discard_pile.push(top_card);
    }

    pub fn is_valid_play(&self, card: &Card) -> bool {
        let top_card = self.discard_pile.last().unwrap();
        match card.value {
            Value::Wild | Value::WildDrawFour => true,
            _ => card.color == top_card.color || card.value == top_card.value,
        }
    }

    pub async fn add_player(
        &mut self,
        tx: Sender<String>,
        mut player: Player
    ) -> Result<(), &'static str> {
        println!("Player {} attempting to join game_state {}", player.id, self.id);
        if self.game_player_pool.connections.len() >= 6 {
            println!("Game is full");
            return Err("Game is full");
        }

        //clone the player so we have a copy to store in the player pool
        let player_clone = player.clone();

        //if the same player is already in the player_pool, return an error
        if self.game_player_pool.connections.iter().any(|conn| conn.player.id == player.id) {
            println!("Player {} already in game_state {}", player.id, self.id);
            return Err("Player already in game");
        }

        //add the player to the player pool
        self.game_player_pool.register_connection(tx, player_clone);

        //if the player is the first player to join, set them as the host
        if self.game_player_pool.connections.len() == 1 {
            println!("Player {} is the host", player.id);
            self.player_to_play = player.id;
        }

        if self.round_in_progress {
            println!("Player {} is a spectator", player.id);
            //Set the player_pools copy of the player to spectator
            player.is_spectator = true;
        }

        let _ = self.update_players().await;

        self.check_and_start_round().await;

        Ok(())
    }

    pub async fn remove_player(&mut self, player_id: usize) -> Result<(), &'static str> {
        if
            let Some(pos) = self.game_player_pool.connections
                .iter()
                .position(|conn| conn.player.id == player_id)
        {
            self.game_player_pool.connections.remove(pos);
            let _ = self.update_players().await;
            Ok(())
        } else {
            Err("Player not found")
        }
    }

    pub async fn check_and_start_round(&mut self) {
        println!("Checking if we can start a round");
        if self.game_player_pool.connections.len() >= 2 && !self.round_in_progress {
            self.is_waiting_for_players = false;
            let _ = self.start_round().await;
        }
    }

    pub fn get_player_count(&self) -> usize {
        self.game_player_pool.connections.len()
    }

    pub async fn start_round(&mut self) {
        //get a list of players

        let players = self.get_all_players_in_game();
        println!("Player count from start_round: {}", players.len());
        if players.len() >= 2 {
            println!("Starting round now");
            self.round_in_progress = true;
            self.direction = 1;
            self.deck = Deck::new();
            self.deck.shuffle();
            self.discard_pile = vec![self.deck.draw().unwrap()]; // Draw the initial card
            self.player_to_play = players[0].id;

            let deck = &mut self.deck;

            println!("Dealing cards to players");
            for conn in self.game_player_pool.connections.iter_mut() {
                if !conn.player.is_spectator {
                    let hand = deck.draw_n(7);
                    conn.player.set_hand(hand);
                }
            }

            println!("Sending player hands to players");
            for conn in &self.game_player_pool.connections {
                if !conn.player.is_spectator {
                    let _ = self.update_player(&conn.player).await;
                }
            }

            //choose a random non-spectator player to start the round

            println!("Player {} will start the round", self.player_to_play);
            //send the game state to all players
            let _ = self.update_game_state().await;

            //get the first player from the pool. We need to find the "next" player that is not a spectator - and we need to take the current direction of the round into account.
            let current_player = self.get_next_player();
            let your_turn_json =
                json!({
                "player_id": current_player.id,
                "message": "your turn!"
            }).to_string();
            let message = create_websocket_message("your_turn", &your_turn_json);
            self.game_player_pool.send_message(&current_player, message).await;
        }
    }

    //logic goes as follow - we know who our current player is. We simply look at current game directioon and step left or right through player pool until we find a non-spectator player
    pub fn get_next_player(&self) -> Player {
        let mut found_player = false;
        let mut player = self.game_player_pool.connections[0].player.clone();
        //find the index of the self.player_to_play in the player pool
        let mut player_index = self.game_player_pool.connections
            .iter()
            .position(|conn| conn.player.id == self.player_to_play)
            .unwrap();

        //loop until we find a non-spectator player to the left or right of the current player (depending on the direction of the round)
        while !found_player {
            //increment or decrement the player index depending on the direction of the round
            player_index = ((player_index as i8) + self.direction) as usize;
            //if we have reached the end of the player pool, loop back to the start
            if player_index >= self.game_player_pool.connections.len() {
                player_index = 0;
            }
            //if we have reached the start of the player pool, loop back to the end
            if player_index < 0 {
                player_index = self.game_player_pool.connections.len() - 1;
            }
            //get the player at the new index
            player = self.game_player_pool.connections[player_index].player.clone();
            //if the player is not a spectator, we have found our next player
            if !player.is_spectator {
                found_player = true;
            }
        }

        //return the player if we found one, otherwise we need to return an error
        player
    }

    //helper function to get all players that are not spectators
    pub fn get_all_players_in_game(&self) -> Vec<Player> {
        self.game_player_pool.connections
            .iter()
            .filter(|conn| !conn.player.is_spectator)
            .map(|conn| conn.player.clone())
            .collect()
    }

    pub fn end_round(&mut self) {
        let players = self.get_all_players_in_game();
        for conn in &mut self.game_player_pool.connections {
            conn.player.hand.clear();
        }
        self.deck = Deck::new();
        self.deck.shuffle();
        self.round_in_progress = false;
        self.discard_pile = vec![self.deck.draw().unwrap()];
        self.player_to_play = players[0].id;
        // Reset game state for next round
    }

    pub fn calculate_points(&self) -> HashMap<usize, i32> {
        let mut points = HashMap::new();

        for conn in &self.game_player_pool.connections {
            let player_points: i32 = conn.player.hand
                .iter()
                .map(|card| card.value.to_points())
                .sum();
            points.insert(conn.player.id, player_points);
        }

        points
    }

    // Helper function to send updates to players
    async fn send_update(&self, event: &str, data: &str) {
        let message = create_websocket_message(event, data);
        self.game_player_pool.broadcast_message(message).await;
    }

    // Simplified card validation in play_card
    fn validate_card_play(&self, player_id: usize, card: &Card) -> Result<(), &'static str> {
        if self.player_to_play != player_id {
            return Err("Not your turn");
        }
        if !self.is_valid_play(&card) {
            return Err("Invalid play");
        }
        Ok(())
    }
}

//tests
// Helper function to serialize player data to JSON
fn serialize_player_data(player: &Player) -> String {
    let json =
        json!({
        "id": player.id,
        "name": player.name,
        "hand": player.hand,
        "current_game": player.current_game,
        "is_spectator": player.is_spectator
    });
    serde_json::to_string(&json).unwrap()
}
