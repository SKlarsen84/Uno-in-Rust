mod card;
mod deck;
mod game_state;
mod lobby;
mod player;
mod websocket;

use std::sync::Arc;

use lobby::Lobby;
use tokio::sync::Mutex;
use warp::ws::WebSocket;
use warp::Filter;

#[tokio::main]
async fn main() {
    // Wrap the Lobby in an Arc<Mutex<...>>
    let lobby = Arc::new(Mutex::new(Lobby::new()));

    // Create a channel for sending commands to the Lobby
    let (lobby_tx, mut lobby_rx) = tokio::sync::mpsc::channel(32); // 32 is the buffer size

    let lobby_clone = lobby.clone();

    // Spawn the task
    tokio::spawn(async move {
        while let Some(command) = lobby_rx.recv().await {
            let mut lobby = lobby_clone.lock().await; // Lock the Mutex here
            if let Err(e) = lobby.handle_command(command).await {
                eprintln!("Error handling lobby command: {}", e);
            }
        }
    });

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || lobby.clone()))
        .map(move |ws: warp::ws::Ws, _lobby: Arc<Mutex<Lobby>>| {
            let lobby_tx_clone = lobby_tx.clone(); // Clone the sender here
            ws.on_upgrade(move |ws: WebSocket| {
                websocket::handle_connection(ws, lobby_tx_clone.clone()) // Clone again here
            })
        });

    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}
