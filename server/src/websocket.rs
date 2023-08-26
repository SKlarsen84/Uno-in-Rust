use futures::SinkExt;
use futures::StreamExt;
use serde::Deserialize;
use serde::Serialize;

use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task;
use warp::ws::{Message, WebSocket}; // Import Deserialize trait

use crate::lobby::Lobby;
use crate::player::Player;
use crate::player::SerializablePlayer;

// Derive Deserialize for ClientMessage
#[derive(Deserialize)]
enum ClientMessage {
    JoinGame { game_id: usize }, // ... other variants
    PlayCard { card: usize },
}

#[derive(Serialize)]
enum ServerMessage {
    PlayerJoined { player: SerializablePlayer }, // Now includes the entire Player object
                                                 // ... other variants
}

pub async fn handle_connection(ws: WebSocket, lobby: Arc<Mutex<Lobby>>) {
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, mut rx): (UnboundedSender<String>, UnboundedReceiver<String>) =
        mpsc::unbounded_channel();
    // Generate a random player id and create a new player
    let player_id = rand::random::<usize>();
    let player = Player::new(player_id, tx.clone());
    let player_clone = player.clone(); // Clone the player before the loop

    {
        let mut lobby = lobby.lock().unwrap();
        lobby.add_player_to_lobby(player.clone());
    }

    task::spawn(async move {
        while let Some(message) = rx.recv().await {
            let _ = user_ws_tx.send(Message::text(message)).await;
        }
    });

    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    let text = msg.to_str().unwrap();
                    let client_msg: ClientMessage = serde_json::from_str(text).unwrap();

                    let mut lobby = lobby.lock().unwrap(); // Lock the lobby once

                    match client_msg {
                        ClientMessage::JoinGame { game_id } => {
                            match lobby.join_game(game_id, player_clone.clone()) {
                                Ok(_) => {
                                    // Notify all players in the game that a new player has joined
                                    let game = lobby.get_game(game_id).unwrap();
                                    for p in game.get_all_players() {
                                        let msg = ServerMessage::PlayerJoined {
                                            player: player_clone.clone().to_serializable(),
                                        };
                                        let response = json!(msg);
                                        p.tx.send(response.to_string()).unwrap();
                                        // Assuming each Player has a tx (transmitter)
                                    }
                                    let response = json!({
                                        "type": "JoinGameResponse",
                                        "success": true
                                    });
                                    tx.send(response.to_string()).unwrap();
                                }
                                Err(err) => {
                                    let response = json!({
                                        "type": "JoinGameResponse",
                                        "success": false,
                                        "error": err
                                    });
                                    tx.send(response.to_string()).unwrap();
                                }
                            }
                        }
                        ClientMessage::PlayCard { card } => {
                            // Update game state and notify players
                        } // Handle other client messages...
                    }
                }
            }
            Err(_) => {
                // Handle WebSocket errors
            }
        }
    }
}
