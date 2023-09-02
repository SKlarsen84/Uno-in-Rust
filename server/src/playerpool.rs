use tokio::sync::mpsc::Sender;

use crate::player::Player;

pub struct PlayerConnection {
    sender: Sender<String>,
    player: Player,
}

pub struct PlayerPool {
    pub connections: Vec<PlayerConnection>,
}

impl PlayerPool {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
        }
    }

    pub fn list_all_players(&self) -> Vec<Player> {
        self.connections
            .iter()
            .map(|conn| conn.player.clone())
            .collect()
    }

    pub fn get_player_by_id(&self, player_id: usize) -> Option<Player> {
        for conn in &self.connections {
            if conn.player.id == player_id {
                return Some(conn.player.clone());
            }
        }
        None
    }

    pub fn register_connection(&mut self, sender: Sender<String>, player: Player) {
        println!("Registering connection for player {}", player.id);
        self.connections.push(PlayerConnection { sender, player });
    }

    pub fn remove_connection(&mut self, player: Player) {
        self.connections.retain(|conn| conn.player.id != player.id);
    }

    pub async fn send_message(&self, player: Player, message: String) {
        for conn in &self.connections {
            if conn.player.id == player.id {
                conn.sender.send(message.clone()).await.unwrap();
            }
        }
    }
}
