use std::collections::HashMap;

use crate::{
    game_state::{GameState, GameStatus},
    player::Player,
    websocket::LobbyCommand,
};
use futures_util::SinkExt;
use tokio::sync::mpsc::Sender as TokioSender;
use warp::filters::ws::WebSocket;

pub struct Lobby {
    games: HashMap<usize, GameState>, // Mapping of game IDs to game states
    next_game_id: usize,              // Counter for generating unique game IDs
    players: Vec<Player>,             // List of players
    connections: Vec<TokioSender<String>>, // Add this line
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            games: HashMap::new(),
            next_game_id: 1,
            players: Vec::new(),
            connections: Vec::new(),
        }
    }
    pub fn register_connection(&mut self, tx: TokioSender<String>) {
        self.connections.push(tx);
    }

    pub fn add_player_to_lobby(&mut self, player: Player) {
        self.players.push(player)
    }

    pub async fn broadcast_lobby_gamelist(&self) {
        let games = self.list_games();
        let games_json = serde_json::to_string(&games).unwrap();
        let response = format!(
            "{{\"sv\": \"update_lobby_games_list\", \"data\": {}}}",
            games_json
        );

        for tx in &self.connections {
            println!("Sending message: {}", response);
            if let Err(e) = tx.send(response.clone()).await {
                println!("Error sending message: {}", e);
            }
        }
    }

    pub fn remove_player_from_lobby(&mut self, player_id: usize) {
        self.players.retain(|p| p.id != player_id);
    }

    pub async fn create_game(&mut self) -> usize {
        let game_id = self.next_game_id;
        self.games.insert(game_id, GameState::new(game_id));
        self.next_game_id += 1;
        println!("Created game {}", game_id);
        self.broadcast_lobby_gamelist().await;
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
            self.broadcast_lobby_gamelist();
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
}
