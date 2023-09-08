import React, { useState, useEffect } from 'react'
import { ICard, useWebSocket } from './WebSocketContext'
import { useNavigate } from 'react-router-dom'

const GameView = () => {
  const context = useWebSocket()
  const navigate = useNavigate()

  if (!context) {
    return <div>Loading...</div>
  }

  const { players, ws, player, gameState, isMyTurn } = context

  const canPlayCard = (card: ICard, discardPile: ICard[] | undefined) => {
    const topCard = discardPile?.[discardPile.length - 1]
    if (!topCard) return false
    return card.color === topCard.color || card.value === topCard.value
  }

  const playCard = (card: ICard) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      console.log('Sending play card message')
      console.log(`Raw message: ${JSON.stringify({ action: 'play_card', card, game_id: gameState?.id })}`)
      ws.send(JSON.stringify({ action: 'play_card', card, game_id: gameState?.id }))
    }
  }

  return (
    <div>
      <h1>Game View</h1>
      <div>
        <h2>Game {gameState?.id}</h2>
        <ul>
          <li>Round in progress: {gameState?.round_in_progress ? 'Yes' : 'No'}</li>
          <li>Direction: {gameState?.direction.toString()}</li>
          <li>Deck size: {gameState?.deck_size}</li>
          <li>Player count: {gameState?.player_count}</li>
          <li>Player turn: {gameState?.player_to_play}</li>
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
        <h2>Top Card in Draw Pile</h2>
        {gameState?.discard_pile.length ? (
          <div>
            {` ${gameState.discard_pile[gameState.discard_pile.length - 1].color}: ${
              gameState.discard_pile[gameState.discard_pile.length - 1].value
            }`}
          </div>
        ) : (
          <div>No cards in discard pile</div>
        )}
      </div>
      <div>
        <h2>Your Hand</h2>
        <ul>
          {player?.hand?.map((card, index) => (
            <li key={index}>
              {` ${card.color}: ${card.value}`}
              {isMyTurn && canPlayCard(card, gameState?.discard_pile) ? (
                <button onClick={() => playCard(card)}>Play Card</button>
              ) : null}
            </li>
          ))}
        </ul>
      </div>
    </div>
  )
}

export default GameView
