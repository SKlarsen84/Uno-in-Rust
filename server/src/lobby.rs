use std::collections::HashMap;

use crate::{
    game_state::{GameState, GameStatus},
    player::Player,
    websocket::LobbyCommand,
};

pub struct Lobby {
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

    pub fn add_player_to_lobby(&mut self, player: Player) {
        self.players.push(player);

        // Collect all game IDs first
        let game_ids: Vec<usize> = self.games.keys().cloned().collect();

        // Now check each game for its status
        for game_id in game_ids {
            self.check_game_status(game_id);
        }
    }

    fn broadcast_lobby_gamelist_changes(&self) {
        // Collect all game IDs first
        let game_ids: Vec<usize> = self.games.keys().cloned().collect();

        //broadcast the list of games to all players
        let games = self.list_games();
        let games_json = serde_json::to_string(&games).unwrap();
        //broadcast the list of games to all players
    }

    pub fn remove_player_from_lobby(&mut self, player_id: usize) {
        self.players.retain(|p| p.id != player_id);
    }

    pub fn create_game(&mut self) -> usize {
        let game_id = self.next_game_id;
        self.games.insert(game_id, GameState::new(game_id));
        self.next_game_id += 1;
        game_id
    }

    pub fn join_game(&mut self, game_id: usize, player: Player) -> Result<(), String> {
        if let Some(game) = self.games.get_mut(&game_id) {
            if let Some(player) = self.players.iter_mut().find(|p| p.id == player.id) {
                player.current_game = Some(game_id);
            }
            game.add_player(player)?;

            Ok(())
        } else {
            Err("Game not found".to_string())
        }
    }

    pub fn leave_game(&mut self, game_id: usize, player_id: usize) -> Result<(), String> {
        if let Some(game) = self.games.get_mut(&game_id) {
            if let Some(player) = self.players.iter_mut().find(|p| p.id == player_id) {
                player.current_game = None;
            }
            game.remove_player(player_id)?;
            // Check if the game is empty and remove it if it is
            self.check_game_status(game_id);

            Ok(())
        } else {
            Err("Game not found".to_string())
        }
    }

    pub fn check_game_status(&mut self, game_id: usize) {
        if let Some(game) = self.games.get(&game_id) {
            match game.get_status() {
                GameStatus::Empty => {
                    self.games.remove(&game_id);
                }
                GameStatus::Active => {}
            }
        }
    }
    pub fn cleanup_empty_games(&mut self) {
        let empty_game_ids: Vec<usize> = self
            .games
            .iter()
            .filter(|&(_, game)| game.players.is_empty())
            .map(|(&id, _)| id)
            .collect();

        for id in empty_game_ids {
            self.games.remove(&id);
        }
    }

    pub fn get_game(&self, game_id: usize) -> Option<&GameState> {
        self.games.get(&game_id)
    }

    // list all games in the lobby
    pub fn list_games(&self) -> Vec<usize> {
        self.games.keys().cloned().collect()
    }

    pub async fn handle_command(&mut self, command: LobbyCommand) -> Result<(), String> {
        match command {
            LobbyCommand::JoinGame { game_id, player_id } => {
                // Find the player and the game, then try to join
                if let Some(player) = self.players.iter().find(|p| p.id == player_id) {
                    self.join_game(game_id, player.clone())
                } else {
                    Err("Player not found".to_string())
                }
            }
            LobbyCommand::FetchGames { response } => {
                println!("received fetch_games websocket action from websocket handler");
                let games = self.list_games();
                let games_json = serde_json::to_string(&games).unwrap();
                let _ = response.send(games_json).await; // Send the JSON string back
                Ok(())
            }
            LobbyCommand::CreateGame { player_id } => {
                // Find the player and create a game
                let game_id = self.create_game();
                if let Some(player) = self.players.iter().find(|p| p.id == player_id) {
                    self.join_game(game_id, player.clone())?;
                    Ok(())
                } else {
                    //remove the game if the player is not found
                    self.games.remove(&game_id);
                    Err("Player not found".to_string())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_join_game() {
        let mut lobby = Lobby::new();
        let game_id = lobby.create_game();
        //initiate a transaction channel tx for the player

        let player = Player::new(0);

        // Test joining a game that exists
        let result = lobby.join_game(game_id, player.clone());
        assert!(result.is_ok());

        // Test joining a game that doesn't exist
        let result = lobby.join_game(game_id + 1, player.clone());
        assert!(result.is_err());
    }

    //test create lobby
    #[tokio::test]
    async fn test_create_lobby() {
        let mut lobby = Lobby::new();
        let game_id = lobby.create_game();
        assert_eq!(game_id, 1);
    }

    //test list games
    #[tokio::test]

    async fn test_list_games() {
        let mut lobby = Lobby::new();
        let _game_id = lobby.create_game();
        let _game_id2 = lobby.create_game();
        let _game_id3 = lobby.create_game();
        let _game_id4 = lobby.create_game();
        let _game_id5 = lobby.create_game();
        let _game_id6 = lobby.create_game();
        let _game_id7 = lobby.create_game();
        let _game_id8 = lobby.create_game();
        let _game_id9 = lobby.create_game();
        let _game_id10 = lobby.create_game();
        let _game_id11 = lobby.create_game();
        let _game_id12 = lobby.create_game();
        let _game_id13 = lobby.create_game();
        let _game_id14 = lobby.create_game();
        let _game_id15 = lobby.create_game();
        let _game_id16 = lobby.create_game();
        let _game_id17 = lobby.create_game();
        let _game_id18 = lobby.create_game();
        let _game_id19 = lobby.create_game();
        let _game_id20 = lobby.create_game();
        let _game_id21 = lobby.create_game();
        let _game_id22 = lobby.create_game();
        let _game_id23 = lobby.create_game();
        let _game_id24 = lobby.create_game();
        let _game_id25 = lobby.create_game();
        let _game_id26 = lobby.create_game();
        let _game_id27 = lobby.create_game();
        let _game_id28 = lobby.create_game();
        let _game_id29 = lobby.create_game();
        let _game_id30 = lobby.create_game();
        let _game_id31 = lobby.create_game();
        let _game_id32 = lobby.create_game();
        let _game_id33 = lobby.create_game();
        let _game_id34 = lobby.create_game();
        let _game_id35 = lobby.create_game();
        let _game_id36 = lobby.create_game();
        let _game_id37 = lobby.create_game();
        let _game_id38 = lobby.create_game();
        let _game_id39 = lobby.create_game();
        let _game_id40 = lobby.create_game();
        let _game_id41 = lobby.create_game();
        let _game_id42 = lobby.create_game();
        let _game_id43 = lobby.create_game();
        let _game_id44 = lobby.create_game();
        let _game_id45 = lobby.create_game();
        let _game_id46 = lobby.create_game();
        let _game_id47 = lobby.create_game();
        let _game_id48 = lobby.create_game();

        let games = lobby.list_games();
        assert_eq!(games.len(), 48);
    }
}
