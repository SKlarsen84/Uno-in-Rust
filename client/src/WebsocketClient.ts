const ws: WebSocket = new WebSocket("ws://localhost:8000");

ws.addEventListener("open", (event: Event) => {
  console.log("WebSocket is open now.");
});

ws.addEventListener("message", (event: MessageEvent) => {
  console.log("Message from server ", event.data);
});

export const fetchGames = (ws: WebSocket) => {
  ws.send(JSON.stringify({ type: "FETCH_GAMES" }));
};

export default ws;
