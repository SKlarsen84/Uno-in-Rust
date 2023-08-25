use futures::SinkExt;
use futures::StreamExt;
use serde::Deserialize;
use serde_json::json;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task;
use warp::ws::{Message, WebSocket}; // Import Deserialize trait

use crate::lobby::Lobby;
use crate::player::Player;

// Derive Deserialize for ClientMessage
#[derive(Deserialize)]
enum ClientMessage {
    JoinGame { game_id: usize }, // ... other variants
    PlayCard { card: usize },
}

pub async fn handle_connection(ws: WebSocket, lobby: Arc<Mutex<Lobby>>) {
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Generate a random player id and create a new player
    let player_id = rand::random::<usize>();
    let player = Player::new(player_id);

    // Add the new player to the lobby
    {
        let mut lobby = lobby.lock().unwrap();
        lobby.add_player(player.clone());
    }

    let (tx, mut rx): (UnboundedSender<String>, UnboundedReceiver<String>) =
        mpsc::unbounded_channel();
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
                            match lobby.join_game(game_id, player.clone()) {
                                Ok(_) => {
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
