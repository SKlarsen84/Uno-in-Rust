use tokio::sync::mpsc::Sender;

use crate::player::Player;

#[derive(Debug)]
pub struct PlayerConnection {
    sender: Sender<String>,
    pub player: Player,
}
#[derive(Debug)]
pub struct PlayerPool {
    pub connections: Vec<PlayerConnection>,
}

impl PlayerPool {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
        }
    }

    pub fn get_player_by_id(&self, player_id: usize) -> Option<Player> {
        for conn in &self.connections {
            if conn.player.id == player_id {
                return Some(conn.player.clone());
            }
        }
        None
    }

    pub fn update_player(&mut self, updated_player: Player) {
        if
            let Some(conn) = self.connections
                .iter_mut()
                .find(|conn| conn.player.id == updated_player.id)
        {
            conn.player = updated_player;
        }
    }

    pub fn register_connection(&mut self, sender: Sender<String>, player: Player) {
        self.connections.push(PlayerConnection { sender, player });
    }

    pub fn remove_connection(&mut self, player: Player) {
        self.connections.retain(|conn| conn.player.id != player.id);
    }

    pub async fn send_message(&self, player: &Player, message: String) {
        for conn in &self.connections {
            if conn.player.id == player.id {
                if let Err(e) = conn.sender.send(message.clone()).await {
                    println!("Failed to send message to player {}: {}", player.id, e);
                }
            }
        }
    }

    //broadcast message - sends to all players in the pool
    pub async fn broadcast_message(&self, message: String) {
        for conn in &self.connections {
            if let Err(e) = conn.sender.send(message.clone()).await {
                println!("Failed to send message to player {}: {}", conn.player.id, e);
            }
        }
    }
}
