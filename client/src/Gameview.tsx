import { useEffect, useState } from 'react'
import { ICard, useWebSocket } from './WebSocketContext'
import { useNavigate } from 'react-router-dom'

const GameView = () => {
  const context = useWebSocket()
  const navigate = useNavigate()
  const [selectedCards, setSelectedCards] = useState<ICard[]>([])

  if (!context) {
    return <div>Loading...</div>
  }
  const { players, ws, player, gameState, isMyTurn } = context

  const canPlayCard = (card: ICard) => {
    const topCard = gameState?.discard_pile?.[gameState?.discard_pile.length - 1]
    const topSelectedCard = selectedCards?.[selectedCards?.length - 1]
    if (!topCard) return false

    // If a card is already selected, only allow cards with the same value to be selected
    if (topSelectedCard) {
      return card.value === topSelectedCard.value
    }

    // Otherwise, allow cards that match the top card's color or value
    console.log('top card: ' + JSON.stringify(topCard))
    console.log('card: ' + JSON.stringify(card))
    return card.color === topCard.color || card.value === topCard.value
  }

  const playSelectedCards = () => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      console.log(
        'sending play cards: ' + JSON.stringify({ action: 'play_cards', cards: selectedCards, game_id: gameState?.id })
      )
      ws.send(JSON.stringify({ action: 'play_cards', cards: selectedCards, game_id: gameState?.id }))
    }
    setSelectedCards([])
  }

  const drawCard = () => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ action: 'draw_card', game_id: gameState?.id }))
    }
    setSelectedCards([])
  }

  const toggleCardSelection = (card: ICard) => {
    if (selectedCards.includes(card)) {
      setSelectedCards(selectedCards.filter(c => c !== card))
    } else {
      if (canPlayCard(card)) {
        setSelectedCards([...selectedCards, card])
      }
    }
  }

  return (
    <div>
      <h1>Game View</h1>
      <h3>{player?.id}</h3>
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
        <h2>Top Card in Pile</h2>
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
              {isMyTurn && canPlayCard(card) && (
                <button onClick={() => toggleCardSelection(card)}>
                  {selectedCards.includes(card) ? 'Deselect' : 'Select'}
                </button>
              )}
            </li>
          ))}
        </ul>
      </div>
      {isMyTurn && (
        <>
          <div>
            <button disabled={selectedCards.length === 0} onClick={playSelectedCards}>
              Play Selected Cards
            </button>
          </div>
          <div>
            <h2>Draw a card</h2>
            {isMyTurn ? <button onClick={drawCard}>Draw Card</button> : null}
          </div>
        </>
      )}
      <div>
        <button onClick={() => navigate('/')}>Back to Lobby</button>
      </div>
    </div>
  )
}

export default GameView
