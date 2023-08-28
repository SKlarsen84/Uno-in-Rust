import React, { useEffect, useState } from 'react'
import ws, { fetchGames } from './WebsocketClient'

const Lobby: React.FC = () => {
  const [games, setGames] = useState<any[]>([])

  useEffect(() => {
    const setupWebSocket = () => {
      ws.onopen = () => {
        fetchGames(ws)
      }

      ws.onmessage = message => {
        try {
          const response = JSON.parse(message.data)
          console.log('Received message:', response)

          if (response.sv === 'update_lobby_games_list') {
            console.log('Updating games list:', response)
            const gameList = JSON.parse(response.data)
            setGames(gameList)
          }
        } catch (e) {
          console.error('Error handling message:', e)
        }
      }

      ws.onerror = error => {
        console.error('WebSocket Error:', error)
      }

      ws.onclose = () => {
        console.log('WebSocket closed. Attempting to reconnect...')
        setupWebSocket()
      }
    }

    setupWebSocket()
  }, [])

  const handleCreateGameClick = () => {
    // Send a message to the Rust server to create a new game
    console.log('Creating game...')
    ws.send(JSON.stringify({ action: 'create_game' }))
  }

  return (
    <div>
      <h1>Lobby</h1>
      {JSON.stringify(games)}
      <ul>
        {/* {games.map((game, index) => (
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
        ))} */}
      </ul>
      <button onClick={handleCreateGameClick}>Create Game</button>
    </div>
  )
}

export default Lobby
