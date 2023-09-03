import React, { useState, useEffect } from 'react'

interface IPlayer {
  id: number
  name: string
}

interface ICard {
  name: string
  color: string
  value: string
}

interface GameViewProps {
  ws: WebSocket
}

const GameView: React.FC<GameViewProps> = ({ ws }) => {
  const [players, setPlayers] = useState<IPlayer[]>([])
  const [hand, setHand] = useState<ICard[]>([])
  const [gameStatus, setGameStatus] = useState<string>('Waiting for players...')

  useEffect(() => {
    // Listen for WebSocket updates here
    ws.addEventListener('message', event => {
      const data = JSON.parse(event.data)
      switch (data.sv) {
        case 'update_players':
          setPlayers(data.data)
          break
        case 'update_hand':
          setHand(data.data)
          break
        case 'update_status':
          setGameStatus(data.data)
          break
        default:
          break
      }
    })
  }, [ws])

  return (
    <div>
      <h1>Game View</h1>
      <div>
        <h2>Status: {gameStatus}</h2>
      </div>
      <div>
        <h2>Players</h2>
        <ul>
          {players.map((player, index) => (
            <li key={index}>{player.name}</li>
          ))}
        </ul>
      </div>
      <div>
        <h2>Your Hand</h2>
        <ul>
          {hand.map((card, index) => (
            <li key={index}>{card.name}</li>
          ))}
        </ul>
      </div>
    </div>
  )
}

export default GameView
