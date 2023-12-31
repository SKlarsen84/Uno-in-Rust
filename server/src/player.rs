use crate::card::Card;
use serde::Serialize;

#[derive(Clone, Debug)]
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
    pub card_count: usize,
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
        //we cannot join a game if we are already in a game
        if self.current_game.is_some() {
            return Err("Player is already in a game".to_string());
        }

        //we cannot join a game if we are already spectating
        if self.is_spectator {
            return Err("Player is already spectating".to_string());
        }

        self.current_game = Some(game_id);

        return Ok(());
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
            card_count: self.hand.len(),
        }
    }

    pub fn set_hand(&mut self, hand: Vec<Card>) {
        self.hand = hand;
    }
}

//tests

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_new() {
        let player = Player::new(0);
        assert_eq!(player.id, 0);
        assert_eq!(player.name, "Player 0");
        assert!(player.hand.is_empty());
        assert!(player.current_game.is_none());
        assert!(!player.is_spectator);
    }

    #[test]
    fn test_player_join_game() {
        let mut player = Player::new(0);
        player.join_game(0).unwrap();
        assert_eq!(player.current_game, Some(0));
    }

    #[test]
    fn test_player_leave_game() {
        let mut player = Player::new(0);
        player.join_game(0).unwrap();
        player.leave_game().unwrap();
        assert_eq!(player.current_game, None);
    }
}
