import React from 'react'
import Lobby from './Lobby'
//get the react-router dom imports
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import GameView from './Gameview'
import { WebSocketProvider } from './WebSocketContext'
import styled from 'styled-components'

const Root = styled.div`
  min-height: 100vh;
  width: 100%;
  overflow-x: hidden;
  /* background: radial-gradient(#5065da, #20295a); */
  background: radial-gradient(#3d50ba, #161d3f);
`

const App: React.FC = () => {
  return (
    <Root>
      <Router>
        <WebSocketProvider>
          <Routes>
            <Route path='/' element={<Lobby />} />
            <Route path='/game/:gameId' element={<GameView />} />
          </Routes>
        </WebSocketProvider>
      </Router>
    </Root>
  )
}

export default App

