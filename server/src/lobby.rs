use std::collections::HashMap;

use crate::{game_state::GameState, player::Player};

pub(crate) struct Lobby {
    games: HashMap<usize, GameState>, // Mapping of game IDs to game states
    next_game_id: usize,              // Counter for generating unique game IDs
    players: Vec<Player>,             // List of players
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            games: HashMap::new(),
            next_game_id: 1,
            players: Vec::new(),
        }
    }

    pub fn add_player(&mut self, player: Player) {
        self.players.push(player);
    }

    pub fn remove_player(&mut self, player_id: usize) {
        self.players.retain(|p| p.id != player_id);
    }

    pub fn create_game(&mut self) -> usize {
        let game_id = self.next_game_id;
        self.games.insert(game_id, GameState::new(6));
        self.next_game_id += 1;
        game_id
    }

    pub fn join_game(&mut self, game_id: usize, player: Player) -> Result<(), String> {
        if let Some(game) = self.games.get_mut(&game_id) {
            game.add_player(player.clone())?;

            if let Some(player) = self.players.iter_mut().find(|p| p.id == player.id) {
                player.current_game = Some(game_id);
            }

            Ok(())
        } else {
            Err("Game not found".to_string())
        }
    }

    pub fn leave_game(&mut self, player_id: usize) -> Result<(), String> {
        if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
            if let Some(game_id) = player.current_game {
                if let Some(game) = self.games.get_mut(&game_id) {
                    game.remove_player(player.id)?;
                    player.current_game = None;
                    Ok(())
                } else {
                    Err("Game not found".to_string())
                }
            } else {
                Err("Player not in a game".to_string())
            }
        } else {
            Err("Player not found".to_string())
        }
    }

    pub fn get_game(&self, game_id: usize) -> Option<&GameState> {
        self.games.get(&game_id)
    }

    // Other methods as needed (e.g., list available games)
}
