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
    // Generate a player_id for this connection
    let player_id = generate_player_id(); // Implement this function to generate a unique ID
    let player = player::Player::new(player_id);
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    lobby.lock().await.register_connection(tx);

    //add the player to the lobby
    lobby.lock().await.add_player_to_lobby(player.clone());

    // Inform the client of their player_id
    let player_id_json = serde_json::to_string(&player_id).unwrap();
    ws.send(Message::text(player_id_json))
        .await
        .expect("Failed to send message");

    while let Some(result) = ws.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(_) => {
                // Handle the error
                continue;
            }
        };
        if msg.is_text() {
            //convert the message to a string
            let text = msg.to_str().unwrap_or_default();
            let client_msg: Result<ClientMessage, _> = serde_json::from_str(&text);
            if client_msg.is_err() {
                // Handle parsing error
                // ...
                continue; // Skip the rest of the loop iteration
            }
            let client_msg = client_msg.unwrap();

            //no matter what our action merits of response, we need to send a response with a json eg {"sv": xxxx, "data": xxx

            match client_msg.action.as_str() {
                "fetch_games" => {
                    println!("received fetch_games websocket action");
                    let mut lobby = lobby.lock().await;
                    let games = lobby.list_games();
                    // Send the list of games back to the client
                    let game_list = serde_json::to_string(&games).unwrap();
                    let response = format!(
                        "{{\"sv\": \"update_lobby_games_list\", \"data\": {}}}",
                        game_list
                    );

                    // Lock the mutex to get mutable access to the SplitSink

                    let _ = ws.send(warp::ws::Message::text(response)).await;
                }

                "join_game" => {
                    println!("received join_game websocket action");
                    // Send a command to the Lobby to join a game
                }
                "create_game" => {
                    println!("received create_game websocket action");
                    let mut lobby = lobby.lock().await;
                    let game_id = lobby.create_game().await;
                    let _ = lobby.join_game(game_id, player.clone());
                }

                "play_card" => {
                    // Send a command to the GameState to play a card
                    // ...
                }
                _ => {
                    // Unknown action
                    // ...
                }
            }
        } // ... (rest of the code remains the same)
    }
}
