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
    pub card: Option<serde_json::Value>,
}

fn generate_player_id() -> usize {
    // Generate a random number
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub async fn handle_connection(
    mut ws: WebSocket,
    lobby: Arc<Mutex<Lobby>>,
    player_pool: Arc<Mutex<PlayerPool>>
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
    //build the player as json and send it to the client
    let player_json =
        json!({
        "id": player.id,
        "name": player.name,
        "hand:": player.hand,
        "current_game": player.current_game,
        "is_spectator": player.is_spectator
    }).to_string();

    //let player_json = serde_json::to_string(&player).unwrap();
    let response = create_websocket_message("player", &player_json);
    let _ = ws.send(Message::text(response)).await;

    // Send the list of games to the client
    let games = lobby.lock().await.list_games();
    let games_json = serde_json::to_string(&games).unwrap();
    let response = create_websocket_message("update_lobby_games_list", &games_json);
    let _ = ws.send(Message::text(response)).await;
    // Main event loop for this connection
    loop {
        tokio::select! {
                // Receiving a message from the WebSocket
                result = ws.next() => {
                    let msg = match result {
                        Some(Ok(msg)) => msg,
                        
                         None => {
                        // WebSocket connection was closed or an error occurred.
                        println!("WebSocket connection closed for player_id: {}", player_id);

                        // Deregister the player from the PlayerPool
                        let mut player_pool = player_pool.lock().await;
                        player_pool.remove_connection(player);

                            //get a list of games from the lobby and - if the player is found in any games player_pool. remove them from that pool
                            let mut lobby = lobby.lock().await;
                            let games = lobby.games.values_mut();
                            for game in games {
                                //if the game has the player in its player_pool, remove them from the pool
                                if game.game_player_pool.connections.iter().any(|conn| conn.player.id == player_id) {
                                    println!("Player {} found and removed from game {}", player_id, game.id);
                                   let _ =  game.remove_player(player_id).await;
                                }
                            }
                            println!("Player {} removed from player_pool", player_id);
                            let _ = create_websocket_message("update_lobby_games_list", &games_json);   
                        break; // Exit the loop
                    },
                    _ => continue,
                    };

                    if msg.is_text() {
                    let text = msg.to_str().unwrap_or_default();
                    println!("Received message from player {}: {}", player_id, text);
                    let client_msg: Result<ClientMessage, _> = serde_json::from_str(&text);
                    if let Err(e) = client_msg {
                        println!("Failed to parse client message: {}", e);
                        continue;
                    }
                    let client_msg = client_msg.unwrap();



                        match client_msg.action.as_str() {
                            "fetch_games" => {
                                println!("fetch_games");
                                    let lobby = lobby.lock().await;
                                   let _ = lobby.broadcast_lobby_gamelist().await;
                            }

                            "create_game" => {
                                let mut lobby = lobby.lock().await;
                                println!("Creating game");
                                let _ = {

                                    lobby.create_game().await
                                };


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
                                            let response = create_websocket_message("you_joined_game", &game_id.to_string());
                                            let _ = player_pool.send_message(&player, response).await;
                                        }

                                        // Add the player to the game   
                                        println!("Adding player to game_state");
                                        let mut lobby = lobby.lock().await;
                                        let game = lobby.games.get_mut(&game_id).unwrap();
                                        println!("identified game as {:?}", game.id);
                                        let _ = game.add_player(tx.clone(), player.clone()).await;
                                        let _ = lobby.broadcast_lobby_gamelist().await;
                                    },
                                    Err(err_msg) => {

                                        // If Err, send a message to the client and continue to the next iteration
                                        let response = create_websocket_message("error", &err_msg);
                                        let _ = ws.send(Message::text(response)).await;
                                        continue;
                                    }
                                }
                            }

                            "play_card" => {
                                println!("got play_card message: {:?}", client_msg.card);
                                //we will be getting a string from the client, so we need to convert it to a card              

                             if let Some(card_json) = &client_msg.card {
        match serde_json::from_value::<Card>(card_json.clone()) {
            Ok(card) => {
                println!("Card: {:?}", card);
            
                                    //get the game_id from the client message
                                    let game_id = client_msg.game_id.unwrap();
                                    let mut lobby = lobby.lock().await;
                                    if let Some(game) = lobby.games.get_mut(&game_id) {
                                        match game.play_card(player_id, card).await {
                                            Ok(_) => {
                                                // Notify the player that the card was successfully played
                                                let message = create_websocket_message("card_played", "ok");
                                                let _ = ws.send(Message::text(message)).await;
                                                
                                                // Update game state for all players
                                                game.update_game_state().await;
                                            },
                                            Err(err) => {
                                                // Notify the player of the error
                                                let message = create_websocket_message("error", err);
                                                let _ = ws.send(Message::text(message)).await;
                                            }
                                        }
                                    }
                                


 },
            Err(e) => {
                println!("Failed to deserialize card: {}", e);
            }
        }
    }

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

//helper function to craft {sv: string, data: string} json messages
pub fn create_websocket_message(sv: &str, data: &str) -> String {
    json!({
        "sv": sv,
        "data": data
    }).to_string()
}
