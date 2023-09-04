import React, { createContext, useContext, useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'

interface IPlayer {
  id: number
  name: string
  hand?: ICard[]
  current_game?: number
  is_spectator?: boolean
}

interface ICard {
  name: string
  color: string
  value: string
}

interface WebSocketContextProps {
  ws: WebSocket | null
  games: any[]
  player: IPlayer | null
  players: IPlayer[]
  hand: ICard[]
  gameState: 'waiting' | 'active'
}

const WebSocketContext = createContext<WebSocketContextProps | null>(null)

export const useWebSocket = () => {
  return useContext(WebSocketContext)
}

interface IWebSocketProviderProps {
  children: React.ReactNode
}

export const WebSocketProvider: React.FC<IWebSocketProviderProps> = ({ children }) => {
  const [ws, setWs] = useState<WebSocket | null>(null)
  const [games, setGames] = useState<any[]>([])
  const [player, setPlayer] = useState<IPlayer | null>(null)
  const [players, setPlayers] = useState<IPlayer[]>([])
  const [hand, setHand] = useState<ICard[]>([])
  const [gameState, setGameState] = useState<'waiting' | 'active'>('waiting')
  const navigate = useNavigate()

  useEffect(() => {
    const newWs = new WebSocket('ws://localhost:3030')
    newWs.addEventListener('open', () => {
      console.log('WebSocket connected.')
    })
    newWs.addEventListener('close', () => {
      console.log('WebSocket closed.')
    })

    newWs.onmessage = message => {
      console.log('Received message:', message.data)
      try {
        const response = JSON.parse(message.data)
        const data = JSON.parse(response.data)
        switch (response.sv) {
          case 'player':
            console.log('Received user id:', response)
            setPlayer(data)
            break
          case 'update_lobby_games_list':
            setGames(data)
            break
          case 'you_joined_game':
            navigate(`/game/${response.data}`)
            break
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
      } catch (e) {
        console.error('Error handling message:', e)
      }
    }

    setWs(newWs)

    return () => {
      newWs.close()
    }
  }, [])

  return (
    <WebSocketContext.Provider value={{ ws, games, player, players, hand, gameState }}>
      {children}
    </WebSocketContext.Provider>
  )
}
