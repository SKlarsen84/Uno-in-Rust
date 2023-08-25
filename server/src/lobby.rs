use crate::game_state::GameState;
use warp::ws::WebSocket;
use std::collections::HashMap;

pub struct Lobby {
    pub rooms: HashMap<String, Room>,
}

pub struct Room {
    pub game_state: GameState,
    pub players: HashMap<String, WebSocket>,
}
