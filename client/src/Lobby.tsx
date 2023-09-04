import React, { useEffect, useState } from 'react'
import { useWebSocket } from './WebSocketContext'
import { useNavigate } from 'react-router-dom'

const Lobby: React.FC = () => {
  const context = useWebSocket()
  const navigate = useNavigate()

  if (!context) {
    return <div>Loading...</div>
  }

  const { games, ws, player } = context

  const handleCreateGameClick = () => {
    // Send a message to the Rust server to create a new game
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ action: 'create_game' }))
    }
  }

  const handleJoinGameClick = (gameId: string) => {
    // Send a message to the Rust server to join this game
    console.log(`Joining game ${gameId}...`)
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ action: 'join_game', game_id: gameId }))

      // Navigate to the game view
      navigate(`/game/${gameId}`)
    }
  }

  return (
    <div>
      <h1>Lobby</h1>
      <h2>{player?.name}</h2>
      <ul>
        {games && games.map((game, index) => (
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
