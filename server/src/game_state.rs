use std::{ collections::HashMap, sync::Arc };

use rand::Rng;
use serde_json::json;
use tokio::sync::{ Mutex, mpsc::Sender };

use crate::{
    card::{ Card, Value },
    player::{ Player, SerializablePlayer },
    playerpool::PlayerPool,
    websocket::create_websocket_message,
    deck::Deck,
};

pub struct GameState {
    pub id: usize,
    pub deck: Deck,
    pub discard_pile: Vec<Card>,

    pub player_to_play: usize,
    pub direction: i8, // 1 for clockwise, -1 for counter-clockwise
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

    pub async fn next_turn(&mut self) {
        //increment the current turn
        let next_player = self.get_next_player();
        //update the next_player's hand for them via the pool connection
        let _ = self.update_single_player(&next_player).await;
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

    // HELPER FUNCTIONS

    pub fn check_winner(&self) -> Option<usize> {
        for entry in &self.game_player_pool.connections {
            if entry.player.hand.is_empty() {
                return Some(entry.player.id);
            }
        }
        None
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

    pub async fn check_and_start_round(&mut self) {
        println!("Checking if we can start a round");
        if self.game_player_pool.connections.len() >= 2 && !self.round_in_progress {
            self.is_waiting_for_players = false;
            let _ = self.start_round().await;
        }
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
                    let _ = self.update_single_player(&conn.player).await;
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
}
