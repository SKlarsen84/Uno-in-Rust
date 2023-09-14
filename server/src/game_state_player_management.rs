use crate::{
    game_state::GameState,
    player::{ Player, SerializablePlayer },
    websocket::create_websocket_message,
};
use serde_json::json;
use tokio::sync::mpsc::Sender;

// player_management.rs
impl GameState {
    pub async fn add_player(
        &mut self,
        tx: Sender<String>,
        mut player: Player
    ) -> Result<(), &'static str> {
        if self.game_player_pool.connections.len() >= 6 {
            return Err("Game is full");
        }

        if self.round_in_progress {
            //Set the player_pools copy of the player to spectator
            player.is_spectator = true;
        }

        //clone the player so we have a copy to store in the player pool
        let player_clone = player.clone();

        //if the same player is already in the player_pool, return an error
        if self.game_player_pool.connections.iter().any(|conn| conn.player.id == player.id) {
            return Err("Player already in game");
        }

        //add the player to the player pool
        self.game_player_pool.register_connection(tx, player_clone);

        //if the player is the first player to join, set them as the host
        if self.game_player_pool.connections.len() == 1 {
            self.player_to_play = player.id;
        }

        let _ = self.update_list_of_players().await;

        self.check_and_start_round().await;

        Ok(())
    }

    pub async fn remove_player(&mut self, player_id: usize) -> Result<(), &'static str> {
        if
            let Some(pos) = self.game_player_pool.connections
                .iter()
                .position(|conn| conn.player.id == player_id)
        {
            self.game_player_pool.connections.remove(pos);
            let _ = self.update_list_of_players().await;
            Ok(())
        } else {
            Err("Player not found")
        }
    }

    pub fn get_player_count(&self) -> usize {
        self.game_player_pool.connections.len()
    }

    /***** SOCKET UPDATERS */
    pub async fn update_list_of_players(&self) {
        let players_data = self.game_player_pool.connections
            .iter()
            .map(|p| p.player.to_serializable())
            .collect::<Vec<SerializablePlayer>>();
        //convert players_data to json string
        let players_data_json = serde_json::to_string(&players_data).unwrap();
        let message = create_websocket_message("update_players", &players_data_json);
        self.game_player_pool.broadcast_message(message).await;
    }

    pub async fn update_single_player(&self, player: &Player) {
        let player_data_json = serialize_player_data(player);
        let message = create_websocket_message("update_player", &player_data_json);
        self.game_player_pool.send_message(&player, message).await;
    }

    //function to let players receive an update about the game state via the pool connection
    pub async fn update_game_state(&self) {
        //build a json object with the game status details
        let info_object =
            json!({
            "id": self.id,
            "round_in_progress": self.round_in_progress,
            "player_to_play": self.player_to_play,
            "direction": self.direction,
            "discard_pile": self.discard_pile,
            "deck_size": self.deck.cards.len(),
            "player_count": self.game_player_pool.connections.len(),
        });

        let game_state_data_json = serde_json::to_string(&info_object).unwrap();
        let message = create_websocket_message("update_game_state", &game_state_data_json);
        self.game_player_pool.broadcast_message(message).await;
    }

    /*** PLAYER HELPER FUNCS */
    pub fn get_all_players_in_game(&self) -> Vec<Player> {
        self.game_player_pool.connections
            .iter()
            .filter(|conn| !conn.player.is_spectator)
            .map(|conn| conn.player.clone())
            .collect()
    }

    pub fn get_player_by_id_mut(&mut self, player_id: usize) -> Option<&mut Player> {
        //get a mutable reference to the player in the player pool
        if
            let Some(player_conn) = self.game_player_pool.connections
                .iter_mut()
                .find(|conn| conn.player.id == player_id)
        {
            Some(&mut player_conn.player)
        } else {
            None
        }
    }

    pub fn get_next_player(&self) -> Player {
        println!("Getting next player");

        //we want to loop through the player pool and get a list of players who are not spectators

        let players = self.game_player_pool.connections
            .iter()
            .filter(|conn| !conn.player.is_spectator)
            .map(|conn| &conn.player) // Map to the player field
            .collect::<Vec<&crate::player::Player>>();

        //get the index of the current player
        let mut player_index = players
            .iter()
            .position(|p| p.id == self.player_to_play)
            .unwrap();

        // Step to the next player in the player list - the game's direction determines whether we increment or decrement the index
        player_index = if self.direction == 1 {
            (player_index + 1) % players.len()
        } else {
            (player_index + players.len() - 1) % players.len()
        };

        //get the player at the new index
        let player = players[player_index].clone();

        //return the player if we found one, otherwise we need to return an error
        player
    }

    pub fn get_next_player_id(&self) -> usize {
        self.get_next_player().id
    }
}

// Helper function to serialize player data to JSON
fn serialize_player_data(player: &Player) -> String {
    let json =
        json!({
        "id": player.id,
        "name": player.name,
        "hand": player.hand,
        "current_game": player.current_game,
        "is_spectator": player.is_spectator
    });
    serde_json::to_string(&json).unwrap()
}
