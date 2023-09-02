use std::sync::Arc;

use crate::card::Card;
use crate::lobby::Lobby;
use crate::player;
use crate::playerpool::PlayerPool;
use futures_util::SinkExt;
use futures_util::StreamExt;
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
    let mut player = player::Player::new(player_id);
    let (tx, mut rx) = mpsc::channel::<String>(32);
    // Register player in PlayerPool
    {
        let mut player_pool = player_pool.lock().await;
        player_pool.register_connection(tx.clone(), player.clone());
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
                    println!("Received message: {:?}", msg);
                    let text = msg.to_str().unwrap_or_default();
                    let client_msg: Result<ClientMessage, _> = serde_json::from_str(&text);
                    if client_msg.is_err() {
                        continue;
                    }
                    let client_msg = client_msg.unwrap();



                    match client_msg.action.as_str() {
                        "fetch_games" => {
                            // Lock, fetch, and then immediately unlock

                                let lobby = lobby.lock().await;
                               let _ = lobby.broadcast_lobby_gamelist().await;
                                //release the lock


                        }


                        "join_game" => {

                            println!("Player {} is joining game {}", player_id, client_msg.game_id.unwrap());
                            let game_id = client_msg.game_id.unwrap();

                            
                            let mut lobby = lobby.lock().await;
                            let game = lobby.games.get_mut(&game_id).unwrap();

                            println!("Game {} has {} players", game_id, game.players.len());

                            //Then, get the player from the player_pool
                            let player_pool = player_pool.lock().await;
                            let mut player = player_pool.get_player_by_id(player_id).unwrap();
                            println!("Found the player in the player pool: {:?}", player.id);

                            //Then, set the player's gameId to the game_id
                            player.join_game(game_id).unwrap();

                            //Then, add the player to the game's list of players
                            game.players.push(player.clone());
                            println!("Added player {} to game {}", player.id, game_id);
                            println!("Game {} now has {} players", game_id, game.players.len());


                            lobby.publish_update().await;


                            println!("players current set to: {}", player.current_game.unwrap_or(0));

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
