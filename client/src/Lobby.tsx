import React, { useEffect, useState } from 'react'
import { useWebSocket } from './WebSocketContext'
import { useNavigate } from 'react-router-dom'
import { Button, Card, CardFooter, Divider } from '@nextui-org/react'

const Lobby: React.FC = () => {
  const context = useWebSocket()
  const navigate = useNavigate()

  if (!context) {
    return <div>Loading...</div>
  }

  const { games, ws, player } = context

  const handleCreateGameClick = () => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ action: 'create_game' }))
    }
  }

  const handleJoinGameClick = (gameId: string) => {
    console.log(`Joining game ${gameId}...`)
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ action: 'join_game', game_id: gameId }))
      navigate(`/game/${gameId}`)
    }
  }

  return (
    <div style={{ padding: '20px', display: 'flex', flexDirection: 'column', alignItems: 'center' }}>
      <p className='text-blue-700 font-bold text-6xl text-bold'>
        Uno<span className='text-red-600'>On</span>
        <span className='text-green-600'>li</span>
        <span className='text-yellow-400'>ne</span>
      </p>

      <div style={{ display: 'flex', padding: 20, flexWrap: 'wrap', gap: '20px', justifyContent: 'center' }}>
        <Divider />
        {games &&
          games.map((game, index) => (
            <Card
              key={index}
              style={{
                width: '222px',
                padding: '20px',
                paddingBottom: '0px',
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center'
              }}
            >
              <p className='text-navy-900 font-bold'>Game: {game.id}</p>
              <p className='text-sm'>{game.player_count} players</p>
              <h4>{game.round_in_progress ? 'Round in progress' : 'Waiting for players'}</h4>
              <CardFooter className='justify-center'>
                <Button color='primary' onClick={() => handleJoinGameClick(game.id)}>
                  {game.round_in_progress ? 'Spectate' : 'Join Game'}
                </Button>
              </CardFooter>
            </Card>
          ))}
      </div>
      <Button color='secondary' onClick={handleCreateGameClick}>
        Create New Game
      </Button>
    </div>
  )
}

export default Lobby
