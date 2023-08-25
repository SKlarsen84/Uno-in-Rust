use serde::Serialize;

use crate::card::Card;
use crate::game_state::GameState; // Assuming this will contain the game logic
use crate::lobby::Lobby; // Assuming this will contain the lobby logic
use tokio::sync::mpsc::UnboundedSender;
#[derive(Clone)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub hand: Vec<Card>,
    pub current_game: Option<usize>, // Game ID or reference to the current game
    pub tx: UnboundedSender<String>, // Add this line
}

#[derive(Serialize, Clone)]
pub struct SerializablePlayer {
    pub id: usize,
    pub name: String,
}

impl Player {
    pub fn new(id: usize, tx: UnboundedSender<String>) -> Self {
        Self {
            id,
            name: format!("Player {}", id),
            hand: Vec::new(),
            current_game: None,
            tx, // Add this line
        }
    }

    pub fn join_game(&mut self, game_id: usize, lobby: &mut Lobby) -> Result<(), String> {
        // Logic to join a game through the lobby
        // Update the current_game attribute
        // Return success or error message
        return Ok(());
    }

    pub fn leave_game(&mut self, lobby: &mut Lobby) -> Result<(), String> {
        // Logic to leave the current game through the lobby
        // Update the current_game attribute
        // Return success or error message
        return Ok(());
    }

    pub fn play_card(
        &mut self,
        card_index: usize,
        game_state: &mut GameState,
    ) -> Result<(), String> {
        // Logic to play a card in the current game
        // Update the hand and game state
        // Return success or error message
        return Ok(());
    }

    pub fn add_card(&mut self, card: Card) {
        self.hand.push(card);
    }

    pub fn remove_card(&mut self, card_index: usize) -> Option<Card> {
        if card_index < self.hand.len() {
            Some(self.hand.remove(card_index))
        } else {
            None
        }
    }

    pub fn to_serializable(&self) -> SerializablePlayer {
        SerializablePlayer {
            id: self.id,
            name: self.name.clone(),
        }
    }

    // pub(crate) fn clone(&self) -> Player {
    //     Player {
    //         id: self.id,
    //         hand: self.hand.clone(),
    //         current_game: None,
    //     }
    // }

    // Other methods as needed
}
