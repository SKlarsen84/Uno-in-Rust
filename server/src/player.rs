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
        //get all games in the lobby and check if the game_id is in the list
        //if it is, set current_game to Some(game_id)
        //if it is not, return an error

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
