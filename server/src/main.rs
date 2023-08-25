mod card;
mod deck;
mod game_state;
mod lobby;
mod player;
mod websocket;
use lobby::Lobby;
use std::sync::{Arc, Mutex};
use warp::Filter;
#[tokio::main]

async fn main() {
    let lobby = Arc::new(Mutex::new(Lobby::new()));
    let with_lobby = warp::any().map(move || lobby.clone());

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_lobby.clone())
        .map(|ws: warp::ws::Ws, lobby| {
            ws.on_upgrade(move |ws| websocket::handle_connection(ws, lobby))
        });
    warp::serve(ws_route).run(([127, 0, 0, 1], 3030)).await;
}
