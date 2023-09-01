use std::collections::HashMap;

use crate::{
    game_state::{GameState, GameStatus},
    player::Player,
};
use serde_json::json;
use tokio::sync::mpsc::Sender;
use warp::filters::ws;

pub struct Lobby {
    games: HashMap<usize, GameState>, // Mapping of game IDs to game states
    next_game_id: usize,              // Counter for generating unique game IDs
    players: Vec<Player>,             // List of players
    websocket_senders: Vec<Sender<String>>,
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            games: HashMap::new(),
            next_game_id: 1,
            players: Vec::new(),
            websocket_senders: Vec::new(),
        }
    }

    pub fn register_connection(&mut self, tx: Sender<String>) {
        self.websocket_senders.push(tx);
    }
    pub fn add_player_to_lobby(&mut self, player: Player) {
        println!("Added player to lobby: {:?}", player.id);
        self.players.push(player);
    }

    pub async fn broadcast_lobby_gamelist(&self) -> Result<(), &'static str> {
        let games = self.list_games();
        let games_json = serde_json::to_string(&games).unwrap();
        let response = json!({
            "sv": "update_lobby_games_list",
            "data": games_json
        })
        .to_string();

        for sender in &self.websocket_senders {
            sender.send(response.clone()).await.unwrap();
        }
        Ok(())
    }

    //call broadcast_lobby_gamelist every 10 seconds

    pub async fn update(&mut self) -> Result<(), &'static str> {
        let _ = self.broadcast_lobby_gamelist().await;

        return Ok(());
    }

    // pub fn remove_player_from_lobby(&mut self, player_id: usize) {
    //     self.players.retain(|p| p.id != player_id);
    // }

    pub async fn create_game(&mut self) -> usize {
        let game_id = self.next_game_id;
        self.games.insert(game_id, GameState::new(game_id));
        self.next_game_id += 1;
        let _ = self.broadcast_lobby_gamelist().await;
        game_id
    }

    // list all games in the lobby with details about player count and round in progress
    pub fn list_games(&self) -> Vec<serde_json::Value> {
        let mut games = Vec::new();
        for (game_id, game_state) in &self.games {
            let game = json!({
                "id": game_id,
                "player_count": game_state.players.len(),
                "round_in_progress": game_state.round_in_progress
            });
            games.push(game);
        }
        games
    }
}
