import React, { useEffect, useState } from 'react'
import ws from './WebSocketClient'

const Lobby: React.FC = () => {
  const [games, setGames] = useState<string[]>([])

  useEffect(() => {
    // Fetch existing games from the server
    // For now, let's mock this
    setGames(['Game 1', 'Game 2'])
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
