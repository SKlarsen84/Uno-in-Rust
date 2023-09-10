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

    pub async fn play_cards(
        &mut self,
        player_id: usize,
        cards: Vec<Card>
    ) -> Result<(), &'static str> {
        // Validation: All cards must have the same value and be valid plays
        let first_card = cards.first().ok_or("No cards provided")?;
        self.validate_card_play(player_id, &first_card)?;

        //having made sure the first card is valid, we can now check that all cards in the vector are the same value as the first card
        if !cards.iter().all(|card| card.value == first_card.value && self.is_valid_play(card)) {
            return Err("Invalid cards");
        }

        // Find and remove the cards from the player's hand
        let mut played_cards: Vec<Card> = Vec::new();
        {
            if
                let Some(player_conn) = self.game_player_pool.connections
                    .iter_mut()
                    .find(|conn| conn.player.id == player_id)
            {
                for card in &cards {
                    if let Some(pos) = player_conn.player.hand.iter().position(|c| c == card) {
                        played_cards.push(player_conn.player.hand.remove(pos));
                    } else {
                        return Err("Card not in hand");
                    }
                }
            } else {
                return Err("Player not found");
            }
        }

        //if there's a draw two card in the played cards, we need to draw two cards for the next player
        if played_cards.iter().any(|card| card.value == Value::DrawTwo) {
            println!("Player {} played a draw two card", player_id);
            let next_player_id = self.next_player_id();
            println!("Drawing two cards for player {}", next_player_id);
            let _ = self.draw_cards(next_player_id, 2).await;
        }

        //if there's a wild draw four card in the played cards, we need to draw four cards for the next player
        if played_cards.iter().any(|card| card.value == Value::WildDrawFour) {
            println!("Player {} played a draw 4 card", player_id);
            let next_player_id = self.next_player_id();
            println!("Drawing two cards for player {}", next_player_id);
            let _ = self.draw_cards(next_player_id, 4).await;
        }

        let _ = self.update_player(
            &self.game_player_pool.get_player_by_id(player_id).unwrap()
        ).await;

        let played_cards_json =
            json!({
                "player_id": player_id,
                "cards": played_cards,
            }).to_string();
        let message = create_websocket_message("card_played", &played_cards_json);
        self.game_player_pool.send_message(
            &self.game_player_pool.get_player_by_id(player_id).unwrap(),
            message
        ).await;

        //push the played cards to the discard pile with the last card played on top
        self.discard_pile.extend(played_cards);

        if let Some(_winner_id) = self.check_winner() {
            self.end_round();
            return Ok(());
        }

        self.next_turn().await;
        Ok(())
    }

    pub async fn next_turn(&mut self) {
        //increment the current turn
        let next_player = self.get_next_player();
        //update the next_player's hand for them via the pool connection
        let _ = self.update_player(&next_player).await;
        self.player_to_play = next_player.id;
        let your_turn_json =
            json!({
                "player_id": next_player.id,
                "message": "your turn!"
            }).to_string();
        let message = create_websocket_message("your_turn", &your_turn_json);
        //update the game's player_to_pl
        self.game_player_pool.send_message(&next_player, message).await;
    }

    // Method to put the game into a "waiting" state
    pub fn go_into_waiting_state(&mut self) {
        self.round_in_progress = false;
        // Don't clear the players; just wait for more to join
    }

    // DRAW CARD FUNCTIONS

    //single card

    pub async fn draw_cards(&mut self, player_id: usize, count: usize) -> Result<(), &'static str> {
        println!("Player {} drawing {} cards", player_id, count);

        let mut cards_to_draw = Vec::new();

        // Draw 'count' number of cards and store them temporarily
        for _ in 0..count {
            if self.deck.is_empty() {
                self.shuffle_discard_into_deck();
            }
            if let Some(card) = self.deck.draw() {
                println!("Drew card: {:?} for player_id {}", card, player_id);
                cards_to_draw.push(card);
            }
        }

        // Now, add the drawn cards to the player's hand
        if let Some(player) = self.get_player_by_id_mut(player_id) {
            player.hand.extend(cards_to_draw);
        } else {
            // Handle the case where the player is not found, if needed
            println!("Player not found");
        }

        Ok(())
    }

    //multiple cards
    pub async fn draw_card(&mut self, player_id: usize) -> Result<(), &'static str> {
        if self.player_to_play != player_id {
            return Err("Not your turn");
        }

        let card = {
            if self.deck.is_empty() {
                self.shuffle_discard_into_deck();
            }
            self.deck.draw()
        };

        if let Some(card) = card {
            if let Some(player) = self.get_player_by_id_mut(player_id) {
                player.hand.push(card);
            } else {
                return Err("Player not found");
            }
        } else {
            return Err("Deck is empty");
        }

        self.next_turn().await;

        Ok(())
    }

    // HELPER FUNCTIONS

    pub fn check_winner(&self) -> Option<usize> {
        for entry in &self.game_player_pool.connections {
            if entry.player.hand.is_empty() {
                return Some(entry.player.id);
            }
        }
        None
    }

    fn get_player_by_id_mut(&mut self, player_id: usize) -> Option<&mut Player> {
        //get a mutable reference to the player in the player pool
        if
            let Some(player_conn) = self.game_player_pool.connections
                .iter_mut()
                .find(|conn| conn.player.id == player_id)
        {
            Some(&mut player_conn.player)
        } else {
            None
        }
    }

    pub fn next_player_id(&self) -> usize {
        let mut current_player_id = self.player_to_play;

        //find the index of our current player in the player pool
        let mut player_index = self.game_player_pool.connections
            .iter()
            .position(|conn| conn.player.id == current_player_id)
            .unwrap();

        //increment or decrement the player index depending on the direction of the round
        player_index = ((player_index as i8) + self.direction) as usize;
        //if we have reached the end of the player pool, loop back to the start
        if player_index >= self.game_player_pool.connections.len() {
            player_index = 0;
        }
        //if we have reached the start of the player pool, loop back to the end
        if player_index <= 0 {
            player_index = self.game_player_pool.connections.len() - 1;
        }
        //get the player at the new index
        current_player_id = self.game_player_pool.connections[player_index].player.id;

        current_player_id
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
            return Err("Game is full");
        }

        //clone the player so we have a copy to store in the player pool
        let player_clone = player.clone();

        //if the same player is already in the player_pool, return an error
        if self.game_player_pool.connections.iter().any(|conn| conn.player.id == player.id) {
            return Err("Player already in game");
        }

        //add the player to the player pool
        self.game_player_pool.register_connection(tx, player_clone);

        //if the player is the first player to join, set them as the host
        if self.game_player_pool.connections.len() == 1 {
            self.player_to_play = player.id;
        }

        if self.round_in_progress {
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

            let your_turn_json =
                json!({
                "player_id": self.player_to_play,
                "message": "your turn!"
            }).to_string();
            let message = create_websocket_message("your_turn", &your_turn_json);
            self.game_player_pool.send_message(&players[0], message).await;
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
            println!("Not your turn");
            return Err("Not your turn");
        }
        if !self.is_valid_play(&card) {
            println!("Invalid play for card: {:?}", card);
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
