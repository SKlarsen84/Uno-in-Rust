import React, { createContext, useContext, useEffect, useState } from 'react'

const WebSocketContext = createContext<WebSocket | null>(null)

export const useWebSocket = () => {
  return useContext(WebSocketContext)
}

interface IWebSocketProviderProps {
  children: React.ReactNode
}

export const WebSocketProvider: React.FC<IWebSocketProviderProps> = ({ children }) => {
  const [ws, setWs] = useState<WebSocket | null>(null)

  useEffect(() => {
    const newWs = new WebSocket('ws://localhost:3030')
    setWs(newWs)
  }, [])

  return <WebSocketContext.Provider value={ws}>{children}</WebSocketContext.Provider>
}
