import React, { useEffect, useState } from 'react'
import ws, { fetchGames } from './WebsocketClient'

const Lobby: React.FC = () => {
  const [games, setGames] = useState<any[]>([])

  useEffect(() => {
    const setupWebSocket = () => {
      ws.onopen = () => {
        console.log('WebSocket connected. Fetching games...')
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
        //  setupWebSocket()
      }
    }

    setupWebSocket()

    return () => {
      // Close the WebSocket connection when the component unmounts
      ws.close()
    }
  }, [])

  // //periodically refetch lobby
  // useEffect(() => {
  //   const interval = setInterval(() => {
  //     fetchGames(ws)
  //   }, 1000)
  //   return () => clearInterval(interval)
  // }, [])

  const handleCreateGameClick = () => {
    // Send a message to the Rust server to create a new game
    console.log('Creating game...')
    ws.send(JSON.stringify({ action: 'create_game' }))
  }

  const handleJoinGameClick = (gameId: string) => {
    // Send a message to the Rust server to join this game
    console.log(`Joining game ${gameId}...`)
    ws.send(JSON.stringify({ action: 'join_game', game_id: gameId }))
  }

  return (
    <div>
      <h1>Lobby</h1>
      <ul>
        {games.map((game, index) => (
          <li key={index}>
            game: {game.id} {game.player_count} players (
            {game.round_in_progress ? 'Round in progress' : 'Waiting for players'})
            <button onClick={() => handleJoinGameClick(game.id)}>Join</button>
          </li>
        ))}
      </ul>
      <button onClick={handleCreateGameClick}>Create Game</button>
    </div>
  )
}

export default Lobby
