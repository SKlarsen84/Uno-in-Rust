use std::{ collections::HashMap, sync::Arc };

use crate::{ game_state::GameState, player::Player, playerpool::PlayerPool };
use serde_json::json;
use tokio::sync::Mutex;

pub struct Lobby {
    pub games: HashMap<usize, GameState>, // Mapping of game IDs to game states
    next_game_id: usize,
    player_pool: Arc<Mutex<PlayerPool>>,
}

impl Lobby {
    pub fn new(player_pool: Arc<Mutex<PlayerPool>>) -> Self {
        Self {
            games: HashMap::new(),
            next_game_id: 1,
            player_pool,
        }
    }

    pub async fn get_all_players_in_lobby(&self) -> Vec<Player> {
        let player_pool = self.player_pool.lock().await;

        //check our player_pool for players that do not have a current_game
        let players_in_pool = player_pool.connections
            .iter()
            .map(|conn| conn.player.clone())
            .collect::<Vec<Player>>();

        let players_not_in_game = players_in_pool
            .into_iter()
            .filter(|player| player.current_game.is_none())
            .collect();
        players_not_in_game
    }

    pub async fn broadcast_lobby_gamelist(&self) -> Result<(), &'static str> {
        let games = self.list_games();
        let games_json = serde_json::to_string(&games).unwrap();
        let response =
            json!({
            "sv": "update_lobby_games_list",
            "data": games_json
        }).to_string();

        // Get all players in the lobby
        let players = self.get_all_players_in_lobby().await; // This locks and releases player_pool

        // Lock player_pool only once here
        let player_pool = self.player_pool.lock().await;

        // Use the player_pool to send a message to each player in the lobby
        for player in players {
            player_pool.send_message(player, response.clone()).await;
        }

        Ok(())
    }

    pub async fn create_game(&mut self) -> usize {
        let game_id = self.next_game_id;

        //create a new player_pool for this game
        let game_player_pool = PlayerPool::new();
        self.games.insert(game_id, GameState::new(game_id, game_player_pool));
        self.next_game_id += 1;

        let _ = self.broadcast_lobby_gamelist().await;

        game_id
    }

    // list all games in the lobby with details about player count and round in progress
    pub fn list_games(&self) -> Vec<serde_json::Value> {
        let mut games = Vec::new();
        for (game_id, game_state) in &self.games {
            let game =
                json!({
                "id": game_id,
                "player_count": game_state.get_player_count(),
                "round_in_progress": game_state.round_in_progress
            });
            games.push(game);
        }
        games
    }
}
