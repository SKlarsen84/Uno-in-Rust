use crate::card::Card;
use crate::player;
use futures_util::SinkExt;
use futures_util::StreamExt;
use rand::Rng;
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender as TokioSender;

#[derive(Deserialize)]
pub struct ClientMessage {
    pub action: String,
    pub game_id: Option<usize>,
    pub card: Option<Card>, // Assuming Card is serializable
}
pub enum LobbyCommand {
    JoinGame { game_id: usize, player_id: usize },
    FetchGames { response: TokioSender<String> },
    // Add more commands as needed
}

fn generate_player_id() -> usize {
    // Generate a random number
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub async fn handle_connection(
    ws_stream: warp::ws::WebSocket,
    lobby_tx: mpsc::Sender<LobbyCommand>,
) {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Generate a player_id for this connection
    let player_id = generate_player_id(); // Implement this function to generate a unique ID
    let player = player::Player::new(player_id);

    // Inform the client of their player_id by sending them a JSON with { "id": <player_id> }
    let player_id_json = serde_json::to_string(&player_id).unwrap();
    let _ = ws_sender
        .send(warp::ws::Message::text(player_id_json))
        .await;

    while let Some(result) = ws_receiver.next().await {
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

            match client_msg.action.as_str() {
                "fetch_games" => {
                    let (response_tx, mut response_rx) = mpsc::channel(1);
                    let _ = lobby_tx
                        .send(LobbyCommand::FetchGames {
                            response: response_tx,
                        })
                        .await;

                    // Wait for the response
                    if let Some(response) = response_rx.recv().await {
                        let _ = ws_sender.send(warp::ws::Message::text(response)).await;
                    }
                }
                "join_game" => {
                    if let Some(game_id) = client_msg.game_id {
                        // Send a command to the Lobby to join a game
                        let _ = lobby_tx
                            .send(LobbyCommand::JoinGame {
                                game_id,
                                player_id: 0, /* Replace with actual player ID */
                            })
                            .await;
                    }
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
