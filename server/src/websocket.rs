use futures_util::StreamExt;
use serde::Deserialize;
use tokio::sync::mpsc;


use crate::card::Card;

#[derive(Deserialize)]
pub struct ClientMessage {
    pub action: String,
    pub game_id: Option<usize>,
    pub card: Option<Card>, // Assuming Card is serializable
}

pub enum LobbyCommand {
    JoinGame { game_id: usize, player_id: usize },
    // Add more commands as needed
}

pub async fn handle_connection(
    ws_stream: warp::ws::WebSocket,
    lobby_tx: mpsc::Sender<LobbyCommand>,
) {
    let (mut _ws_sender, mut ws_receiver) = ws_stream.split();

    while let Some(result) = ws_receiver.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(_) => {
                // Handle the error
                continue;
            }
        };
        if msg.is_text()  {
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
