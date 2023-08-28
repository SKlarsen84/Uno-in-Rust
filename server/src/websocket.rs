use std::collections::HashMap;
use std::sync::Arc;

use crate::card::Card;
use crate::lobby::Lobby;
use crate::player;
use futures_util::SinkExt;
use futures_util::StreamExt;
use warp::filters::ws::Message;
use warp::ws::WebSocket;

use rand::Rng;
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender as TokioSender;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct ClientMessage {
    pub action: String,
    pub game_id: Option<usize>,
    pub card: Option<Card>, // Assuming Card is serializable
}

pub enum LobbyCommand {
    JoinGame { game_id: usize, player_id: usize },
    FetchGames { response: TokioSender<String> },
    CreateGame { player_id: usize },
    // Add more commands as needed
}

fn generate_player_id() -> usize {
    // Generate a random number
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub async fn handle_connection(mut ws: WebSocket, lobby: Arc<Mutex<Lobby>>) {
    let player_id = generate_player_id();
    let player = player::Player::new(player_id);
    let (tx, mut rx) = mpsc::channel::<String>(32);
    {
        let mut lobby = lobby.lock().await;
        lobby.register_connection(tx.clone());
        lobby.add_player_to_lobby(player.clone());
    }
    // Send the player ID to the client
    let player_id_json = serde_json::to_string(&player_id).unwrap();
    ws.send(Message::text(player_id_json))
        .await
        .expect("Failed to send message");

    // Main event loop for this connection
    loop {
        tokio::select! {
            // Receiving a message from the WebSocket
            result = ws.next() => {
                let msg = match result {
                    Some(Ok(msg)) => msg,
                    _ => continue,
                };

                if msg.is_text() {
                    let text = msg.to_str().unwrap_or_default();
                    let client_msg: Result<ClientMessage, _> = serde_json::from_str(&text);
                    if client_msg.is_err() {
                        continue;
                    }
                    let client_msg = client_msg.unwrap();

                    match client_msg.action.as_str() {
                        "fetch_games" => {
                            // Lock, fetch, and then immediately unlock
                            let games = {
                                let lobby = lobby.lock().await;
                                lobby.list_games()
                            };

                            let games_json = serde_json::to_string(&games).unwrap();
                            let response = format!(
                                "{{\"sv\": \"update_lobby_games_list\", \"data\": {}}}",
                                games_json
                            );
                            let _ = ws.send(warp::ws::Message::text(response)).await;
                        }
                        "join_game" => {
                            // Implement join_game logic
                        }
                        "create_game" => {
                            println!("Received create_game action.");
                            let game_id = {
                                let mut lobby = lobby.lock().await;
                                lobby.create_game().await
                            };
                            println!("Created game with ID: {}", game_id);
                            let response = format!(
                                "{{\"sv\": \"game_created\", \"data\": {}}}",
                                game_id
                            );
                            let _ = ws.send(warp::ws::Message::text(response)).await;

                            // Send the updated game list to all clients
                            let _ = lobby.lock().await.broadcast_lobby_gamelist().await;
                        }
                        "play_card" => {
                            // Implement play_card logic
                        }
                        _ => {}
                    }
                }
            },
            // Receiving a message from the lobby (via the channel)
            Some(message) = rx.recv() => {
                // Forward the message to the WebSocket
                let _ = ws.send(Message::text(message)).await;
            }
        }
    }
}
