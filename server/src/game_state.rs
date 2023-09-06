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
    current_turn: usize,
    direction: i8, // 1 for clockwise, -1 for counter-clockwise
    pub round_in_progress: bool,
    pub is_waiting_for_players: bool,
    pub game_player_pool: PlayerPool,
}
pub enum GameStatus {
    Active,
    Empty,
}

impl GameState {
    pub fn new(id: usize, player_pool: PlayerPool) -> Self {
        let mut deck = Deck::new();
        deck.shuffle();

        let discard_pile = vec![deck.draw().unwrap()]; // Draw the initial card
        let current_turn = 0;
        let direction = 1;

        Self {
            id: id,

            deck,
            discard_pile,
            current_turn,
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
        let message = create_websocket_message("update_players", &players_data_json);
        self.game_player_pool.broadcast_message(message).await;
        self.update_game_state().await;
    }

    //function to let the player receive an update about their hand content via the pool connection
    pub async fn update_player(&self, player: &Player) {
        // let player = self.game_player_pool.get_player_by_id(player_id).unwrap();
        let player_data = player.clone();
        //construct a json string from the player data
        let json =
            json!({
            "id": player_data.id,
            "name": player_data.name,
            "hand": player_data.hand,
            "current_game": player_data.current_game,
            "is_spectator": player_data.is_spectator
        });

        let player_data_json = serde_json::to_string(&json).unwrap();
        let message = create_websocket_message("update_player", &player_data_json);
        println!("game state {} sending player update to  player {}", self.id, player.id);
        self.game_player_pool.send_message(player, message).await;
    }

    //function to let players receive an update about the game state via the pool connection
    pub async fn update_game_state(&self) {
        //build a json object with the game status details
        let info_object =
            json!({
            "round_in_progress": self.round_in_progress,
            "current_turn": self.current_turn,
            "direction": self.direction,
            "discard_pile": self.discard_pile,
            "deck_size": self.deck.cards.len(),
            "player_count": self.game_player_pool.connections.len(),
        });

        let game_state_data_json = serde_json::to_string(&info_object).unwrap();
        let message = create_websocket_message("update_game_state", &game_state_data_json);
        self.game_player_pool.broadcast_message(message).await;
    }

    pub fn play_card(&mut self, player_id: usize, card: Card) -> Result<(), &'static str> {
        // Check if it's the player's turn
        if self.current_turn != player_id {
            return Err("Not your turn");
        }

        //check if the card is valid
        if !self.is_valid_play(&card) {
            return Err("Invalid play");
        }

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
            self.next_turn();

            Ok(())
        } else {
            Err("Card not in hand")
        }
    }

    pub fn next_turn(&mut self) {
        self.current_turn =
            (self.current_turn + (self.direction as usize)) %
            self.game_player_pool.connections.len();
    }

    // Method to handle when all players leave
    pub fn handle_all_players_left(&mut self) {
        //check if player pool is empty
        if self.game_player_pool.connections.is_empty() {
            self.end_game();
        }
    }

    pub fn apply_card_effect(&mut self, card: &Card) {
        match card.value {
            Value::Skip => self.next_turn(),
            Value::Reverse => {
                self.direction *= -1;
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

    pub fn next_player_id(&self) -> usize {
        (self.current_turn + (self.direction as usize)) % self.game_player_pool.connections.len()
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

    pub fn draw_card(&mut self, player_id: usize) -> Result<(), &'static str> {
        if self.current_turn != player_id {
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

    pub fn get_status(&self) -> GameStatus {
        if self.game_player_pool.connections.is_empty() {
            GameStatus::Empty
        } else {
            GameStatus::Active
        }
    }

    pub fn get_player(&self, player_id: usize) -> Option<Player> {
        self.game_player_pool.get_player_by_id(player_id)
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
            self.current_turn = 0;

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

            self.current_turn = 1;
            println!("Player {} will start the round", self.current_turn);
            //send the game state to all players
            let _ = self.update_game_state().await;
        }
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
        for conn in &mut self.game_player_pool.connections {
            conn.player.hand.clear();
        }
        self.deck = Deck::new();
        self.deck.shuffle();
        self.round_in_progress = false;
        self.discard_pile = vec![self.deck.draw().unwrap()];
        self.current_turn = 0;
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

    pub fn end_game(&mut self) {
        self.round_in_progress = false;
        self.game_player_pool.connections.clear();
        self.deck = Deck::new();
        self.deck.shuffle();
        self.discard_pile = vec![self.deck.draw().unwrap()];
        self.current_turn = 0;
    }
}

//tests
