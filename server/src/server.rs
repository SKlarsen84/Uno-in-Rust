use crate::lobby::{Lobby, Room};
use std::sync::{Arc, RwLock};
use warp::{Filter, ws::WebSocket};

// All the WebSocket and server-related functions
enum ClientMessage {
    JoinGame { game_id: usize },
    PlayCard { card: Card },
    // Other client actions
}

enum ServerMessage {
    GameStateUpdate { game_state: GameState },
    Error { message: String },
    // Other server updates
}