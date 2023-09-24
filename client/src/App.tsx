import React from 'react'
import Lobby from './Lobby'
//get the react-router dom imports
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom'
import GameView from './Gameview'
import { WebSocketProvider } from './WebSocketContext'
import styled from 'styled-components'
import { NextUIProvider } from '@nextui-org/react'
import 'tailwindcss/tailwind.css'

const Root = styled.div`
  min-height: 100vh;
  width: 100%;
  overflow-x: hidden;
  /* background: radial-gradient(#5065da, #20295a); */
  background: linear-gradient(#e1e1e1, #747889);
`

const App: React.FC = () => {
  return (
    <NextUIProvider>
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
    </NextUIProvider>
  )
}

export default App

