import React, { useState, useEffect } from 'react'
import { useWebSocket } from './WebSocketContext'
import { useNavigate } from 'react-router-dom'

const GameView = () => {
  const context = useWebSocket()
  const navigate = useNavigate()

  if (!context) {
    return <div>Loading...</div>
  }

  const { players, ws, player, gameState } = context

  return (
    <div>
      <h1>Game View</h1>
      <div>
        <h2>Game State</h2>
        <ul>
          <li>Round in progress: {gameState?.round_in_progress ? 'Yes' : 'No'}</li>
          <li>Current turn: {gameState?.current_turn.toString()}</li>
          <li>Direction: {gameState?.direction.toString()}</li>
          <li>Deck size: {gameState?.deck_size}</li>
          <li>Player count: {gameState?.player_count}</li>
        </ul>
      </div>
      <div>
        <h2>Players</h2>
        <ul>
          {players?.map((player, index) => (
            <li key={index}>{player.id}</li>
          ))}
        </ul>
      </div>
      <div>
        <h2>Your Hand</h2>

        <ul>
          {player?.hand?.map((card, index) => (
            <li key={index}>{` ${card.color}: ${card.value}`}</li>
          ))}
        </ul>
      </div>
    </div>
  )
}

export default GameView
