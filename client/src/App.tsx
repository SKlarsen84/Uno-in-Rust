import React from 'react'
import Lobby from './Lobby'
//get the react-router dom imports
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import GameView from './Gameview'
import { WebSocketProvider } from './WebSocketContext'

const App: React.FC = () => {
  return (
    <Router>
      <WebSocketProvider>
        <Routes>
          <Route path='/' element={<Lobby />} />
          <Route path='/game/:gameId' element={<GameView />} />
        </Routes>
      </WebSocketProvider>
    </Router>
  )
}

export default App

