mod card;
mod deck;
mod game_state;
mod lobby;
mod player;
mod playerpool;
mod websocket;
mod game_state_player_management;
mod game_state_card_management;
use std::sync::Arc;

use crate::playerpool::PlayerPool;
use lobby::Lobby;
use tokio::sync::Mutex;
use warp::ws::WebSocket;
use warp::Filter;

#[tokio::main]
async fn main() {
    // Wrap the Lobby and PlayerPool in an Arc<Mutex<...>>
    let player_pool = Arc::new(Mutex::new(PlayerPool::new()));
    let lobby = Arc::new(Mutex::new(Lobby::new(player_pool.clone()))); // Clone player_pool here

    println!("Server running on http://127.0.0.1:3030");
    let ws_route = warp
        ::ws()
        .and(warp::any().map(move || lobby.clone()))
        .map(move |ws: warp::ws::Ws, lobby: Arc<Mutex<Lobby>>| {
            let player_pool = player_pool.clone(); // Clone player_pool here
            ws.on_upgrade(move |ws: WebSocket| {
                websocket::handle_connection(ws, lobby.clone(), player_pool) // Use the cloned player_pool
            })
        });

    warp::serve(ws_route).run(([0, 0, 0, 0], 3030)).await;
}
