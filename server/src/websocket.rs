use std::sync::Arc;

use crate::card::Card;
use crate::lobby::Lobby;
use crate::player;
use crate::playerpool::PlayerPool;
use futures_util::SinkExt;
use futures_util::StreamExt;
use serde_json::json;
use warp::filters::ws::Message;
use warp::ws::WebSocket;

use rand::Rng;
use serde::Deserialize;
use tokio::sync::mpsc;

use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct ClientMessage {
    pub action: String,
    pub game_id: Option<usize>,
    pub card: Option<Card>, // Assuming Card is serializable
}

fn generate_player_id() -> usize {
    // Generate a random number
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub async fn handle_connection(
    mut ws: WebSocket,
    lobby: Arc<Mutex<Lobby>>,
    player_pool: Arc<Mutex<PlayerPool>>,
) {
    let player_id = generate_player_id();
    let player = player::Player::new(player_id);
    let (tx, mut rx) = mpsc::channel::<String>(32);
    // Register player in PlayerPool
    {
        let mut player_pool = player_pool.lock().await;
        player_pool.register_connection(tx.clone(), player.clone());
    }

    // Send the player ID to the client
    let player_id_json = serde_json::to_string(&player_id).unwrap();
    let response = json!({
        "sv": "user_id",
        "data": player_id_json
    })
    .to_string();
    let _ = ws.send(Message::text(response)).await;

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
                                    let lobby = lobby.lock().await;
                                   let _ = lobby.broadcast_lobby_gamelist().await;
                            }


                            "join_game" => {

                                let game_id = client_msg.game_id.unwrap();
                                let result;
                                {
            let mut player_pool = player_pool.lock().await;
            if let Some(mut player) = player_pool.get_player_by_id(player_id) {
                result = player.join_game(game_id);
                if result.is_ok() {
                    player.current_game = Some(game_id);
                    // Update the player in the pool
                    player_pool.update_player(player);
                }
            } else {
                // Handle player not found
                continue;
            }
        }

                                // Use match to handle the Result
                                match result {
                                    Ok(_) => {

                                        {
                                            let player_pool = player_pool.lock().await;
                                            let mut player = player_pool.get_player_by_id(player_id).unwrap();
                                            player.current_game = Some(game_id);
                                            let _ = player_pool.send_message(player.clone(), "You joined the game".to_string()).await;
                                        }


                                        let mut lobby = lobby.lock().await;
                                        let game = lobby.games.get_mut(&game_id).unwrap();
                                        let _ = game.add_player(player.clone());
                                        let _ = lobby.broadcast_lobby_gamelist().await;
                                    },
                                    Err(err_msg) => {

                                        // If Err, send a message to the client and continue to the next iteration
                                        let response = json!({
                                            "sv": "error",
                                            "data": err_msg
                                        })
                                        .to_string();
                                        let _ = ws.send(Message::text(response)).await;
                                        continue;
                                    }
                                }
                            }


                            "create_game" => {
                                let mut lobby = lobby.lock().await;

                                let _ = {

                                    lobby.create_game().await
                                };


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
