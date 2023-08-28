const ws: WebSocket = new WebSocket("ws://localhost:3030");

ws.addEventListener("open", (event: Event) => {
});

ws.addEventListener("message", (event: MessageEvent) => {
});

export const fetchGames = (ws: WebSocket) => {
  ws.send(JSON.stringify({ action: "fetch_games" }));
};

export default ws;
