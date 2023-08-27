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

    //also serve a http server to simply show "hello world"
    println!("Server running on http://127.0.0.1:3030");
    let ws_route = warp::ws().and(warp::any().map(move || lobby.clone())).map(
        move |ws: warp::ws::Ws, lobby: Arc<Mutex<Lobby>>| {
            let lobby_clone = lobby.clone(); // Clone it here
            ws.on_upgrade(move |ws: WebSocket| {
                websocket::handle_connection(ws, lobby_clone.clone())
            })
        },
    );

    warp::serve(ws_route).run(([0, 0, 0, 0], 3030)).await
}
