import React, { createContext, useContext, useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'

interface IPlayer {
  id: number
  name: string
  hand?: ICard[]
  current_game?: number
  is_spectator?: boolean
}

export interface ICard {
  name: string
  color: string
  value: string
}

interface WebSocketContextProps {
  ws: WebSocket | null
  games: any[]
  player: IPlayer | null
  players: IPlayer[]
  gameState: {
    round_in_progress: Boolean
    player_to_play: number
    direction: number
    discard_pile: ICard[]
    deck_size: number
    player_count: number
    id: number
  }
  isMyTurn: boolean
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
  const [gameState, setGameState] = useState<{
    round_in_progress: Boolean
    player_to_play: number
    id: number
    direction: number
    discard_pile: ICard[]
    deck_size: number
    player_count: number
  }>({
    id: 0,
    round_in_progress: false,
    player_to_play: 0,
    direction: 1,
    discard_pile: [],
    deck_size: 102,
    player_count: 0
  })
  const [isMyTurn, setIsMyTurn] = useState<boolean>(false)
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
      try {
        const response = JSON.parse(message.data)

        let data = response.data
        try {
          data = JSON.parse(response.data)
        } catch (e) {
          // Ignore
        }
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
          case 'update_player':
            setPlayer(data)
            break
          case 'update_game_state':
            setGameState(data)
            break
          case 'your_turn':
            setIsMyTurn(true)
            break
          case 'card_played':
            setIsMyTurn(false)
            break
          case 'card_drawn':
            setIsMyTurn(false)
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
    <WebSocketContext.Provider value={{ ws, games, player, players, gameState, isMyTurn }}>
      {children}
    </WebSocketContext.Provider>
  )
}
