import React, { useState, useEffect } from 'react'
import { useWebSocket } from './WebSocketContext'

interface IPlayer {
  id: number
  color: string
  value: string
}

interface ICard {
  name: string
  color: string
  value: string
}

const GameView = () => {
  const [players, setPlayers] = useState<IPlayer[]>([])
  const [hand, setHand] = useState<ICard[]>([])
  const [gameState, setGameState] = useState<'waiting' | 'active'>('waiting')
  const ws = useWebSocket() as WebSocket

  useEffect(() => {
    if (ws) {
      ws.onopen = () => {
        console.log('Gameview WebSocket connected.')
      }
      // Listen for WebSocket updates here
      ws.addEventListener('message', event => {
        const msg = JSON.parse(event.data)
        const data = JSON.parse(msg.data)

        console.log('Received message:', msg.sv)
        console.log(data)

        switch (msg.sv) {
          case 'update_players':
            setPlayers(data)
            break
          case 'update_player_hand':
            setHand(data)
            break
          case 'update_game_state':
            setGameState(data)
            break
          default:
            break
        }
      })
    }
  }, [ws])

  return (
    <div>
      <h1>Game View</h1>
      <div>
        <h2>Status: {gameState}</h2>
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
          {hand?.map((card, index) => (
            <li key={index}>{` ${card.color}: ${card.value}`}</li>
          ))}
        </ul>
      </div>
    </div>
  )
}

export default GameView
