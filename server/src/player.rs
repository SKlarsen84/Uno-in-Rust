use crate::card::Card;
use serde::Serialize;


#[derive(Clone)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub hand: Vec<Card>,
    pub current_game: Option<usize>, // Game ID or reference to the current game
    pub is_spectator: bool,
}

#[derive(Serialize, Clone)]
pub struct SerializablePlayer {
    pub id: usize,
    pub name: String,
}

impl Player {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            name: format!("Player {}", id),
            hand: Vec::new(),
            current_game: None,

            is_spectator: false,
        }
    }

    // Removed lobby from the method signature
    pub fn join_game(&mut self, game_id: usize) -> Result<(), String> {
        self.current_game = Some(game_id);
        Ok(())
    }

    // Removed lobby from the method signature
    pub fn leave_game(&mut self) -> Result<(), String> {
        self.current_game = None;
        Ok(())
    }

    // ... (rest of your methods)

    pub fn to_serializable(&self) -> SerializablePlayer {
        SerializablePlayer {
            id: self.id,
            name: self.name.clone(),
        }
    }

    pub fn set_hand(&mut self, hand: Vec<Card>) {
        self.hand = hand;
    }
}
