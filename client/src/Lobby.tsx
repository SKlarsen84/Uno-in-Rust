import React, { useEffect, useState } from 'react'
import ws from './WebSocketClient'
import { fetchGames } from './WebSocketClient'

const Lobby: React.FC = () => {
  const [games, setGames] = useState<any[]>([])

  useEffect(() => {
    // Initialize WebSocket connection
    const ws = new WebSocket('ws://localhost:8000')

    ws.onopen = () => {
      // Fetch lobbies once connected
      fetchGames(ws)
    }

    ws.onmessage = message => {
      const data = JSON.parse(message.data)
      if (data.type === 'GAMES') {
        setGames(data.games)
      }
    }

    return () => {
      ws.close()
    }
  }, [])

  return (
    <div>
      <h1>Lobby</h1>
      <ul>
        {games.map((game, index) => (
          <li key={index}>
            {game}
            <button
              onClick={() => {
                // Send a message to the Rust server to join this game
                ws.send(`join_game:${game}`)
              }}
            >
              Join
            </button>
          </li>
        ))}
      </ul>
      <button
        onClick={() => {
          // Send a message to the Rust server to create a new game
          ws.send('create_game')
        }}
      >
        Create Game
      </button>
    </div>
  )
}

export default Lobby