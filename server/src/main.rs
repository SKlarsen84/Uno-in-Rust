mod card;
mod deck;
mod player;
mod game_state;
mod lobby;
mod server;

use warp::Filter;
use std::sync::{Arc, RwLock};
use crate::lobby::Lobby;
use crate::server::ws_endpoint;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    // Initialize the lobby
    let lobby = Arc::new(RwLock::new(Lobby {
        rooms: HashMap::new(),
    }));

    // Define the WebSocket endpoint
    let ws_route = ws_endpoint(lobby);

    // Define other routes (e.g., HTTP endpoints)
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));

    // Combine the WebSocket and HTTP routes
    let routes = ws_route.or(hello);

    // Start the server
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
