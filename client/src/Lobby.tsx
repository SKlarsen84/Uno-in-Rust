import React, { useEffect, useState } from 'react'
import ws, { fetchGames } from './WebsocketClient'

const Lobby: React.FC = () => {
  const [games, setGames] = useState<any[]>([])

  useEffect(() => {
    // Initialize WebSocket connection
    const ws = new WebSocket('ws://localhost:3030')

    ws.onopen = () => {
      // Fetch lobbies once connected
      fetchGames(ws)
    }

    ws.onmessage = message => {
      const response = JSON.parse(message.data)

      if (response.sv === 'fetch_games') {
        console.log('data', response.data)
        setGames(response.data)
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
          ws.send(JSON.stringify({ action: 'create_game' }))
        }}
      >
        Create Game
      </button>
    </div>
  )
}

export default Lobby
