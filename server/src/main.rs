mod card;
mod deck;
mod game_state;
mod lobby;
mod player;
mod websocket;

use std::collections::HashMap;
use std::sync::Arc;

use lobby::Lobby;
use tokio::sync::Mutex;
use warp::ws::WebSocket;
use warp::Filter;

#[tokio::main]

async fn main() {
    // Wrap the Lobby in an Arc<Mutex<...>>
    let lobby = Arc::new(Mutex::new(Lobby::new()));
    let lobby_clone = lobby.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            println!("Trying to acquire lock...");

            {
                let mut lobby = lobby_clone.lock().await;
                println!("Lock acquired.");
                let e = lobby.update().await;
                println!("Update result: {:?}", e)
            } // Mutex is unlocked here

            println!("Lock released.");
        }

    });

    println!("Server running on http://127.0.0.1:3030");
    let ws_route = warp::ws().and(warp::any().map(move || lobby.clone())).map(
        move |ws: warp::ws::Ws, lobby: Arc<Mutex<Lobby>>| {
            ws.on_upgrade(move |ws: WebSocket| websocket::handle_connection(ws, lobby))
        },
    );

    warp::serve(ws_route).run(([0, 0, 0, 0], 3030)).await
}
