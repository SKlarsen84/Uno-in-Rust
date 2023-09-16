import { useEffect, useState } from 'react'
import { ICard, useWebSocket } from './WebSocketContext'
import { useNavigate } from 'react-router-dom'
import Card from './components/Card/Card'
import styled from 'styled-components'
// import Card from './components/Card/Card'

interface CardRowProps {
  cardsCnt?: number
  highlight?: boolean
}

const CardRow = styled.div<CardRowProps>`
  display: flex;
  justify-content: center;

  filter: ${props => (props.highlight ? 'drop-shadow(0 0 10px white)' : 'brightness(0.6)')};

  --cardsCnt: ${props => props.cardsCnt};
  --containerMaxWidth: 55vw;
  .card-container {
    &:not(:last-of-type) {
      margin-right: calc(
        -1 * max(calc((var(--cardWidth) * var(--cardsCnt) - var(--containerMaxWidth)) / (var(--cardsCnt)-1)), calc(var(
                  --cardWidth
                ) / 3))
      );
    }
  }
`

const GameView = () => {
  const context = useWebSocket()
  const navigate = useNavigate()
  const [selectedCards, setSelectedCards] = useState<ICard[]>([])
  const [showColorModal, setShowColorModal] = useState(false)

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
    return (
      card.color === topCard.color || card.value === topCard.value || card.color === 'Wild' || topCard.color === 'Wild'
    )
  }

  const playSelectedCards = () => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      const wildCardIndex = selectedCards.findIndex(card => card.color === 'Wild')
      if (wildCardIndex !== -1) {
        setShowColorModal(true)
        return
      }
      console.log(`Playing cards to websocket:`)
      console.log(selectedCards)
      ws.send(JSON.stringify({ action: 'play_cards', cards: selectedCards, game_id: gameState?.id }))
    }
    setSelectedCards([])
  }

  const handleColorSelect = (color: string) => {
    const wildCardIndex = selectedCards.findIndex(card => card.color === 'Wild')
    if (wildCardIndex !== -1) {
      selectedCards[wildCardIndex].color = color
    }
    setShowColorModal(false)
    console.log(`Selected color: ${color}`)
    playSelectedCards() // Automatically play the card after color selection
  }

  const drawCard = () => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ action: 'draw_card', game_id: gameState?.id }))
    }
    setSelectedCards([])
  }

  const toggleCardSelection = (card: ICard) => {
    if (!isMyTurn) return
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
        <h2>Top Card in Pile</h2>
        {gameState?.discard_pile.length ? (
          <div>
            <div style={{ display: 'flex', justifyContent: 'center' }}>
              <div style={{ width: '190px' }}>
                <Card
                  color={gameState.discard_pile[gameState.discard_pile.length - 1].color}
                  value={gameState.discard_pile[gameState.discard_pile.length - 1].value}
                  selectable={false}
                  playable={false}
                  onCardClick={() => {}}
                  cardIsSelected={false}
                  flip={false}
                  rotationY={0}
                />
              </div>
            </div>
          </div>
        ) : (
          <div>No cards in discard pile</div>
        )}
      </div>
      <div>
        <h2>Your Hand</h2>
        <div style={{ display: 'flex', justifyContent: 'center' }}>
          <CardRow>
            {player?.hand?.map((card, index) => (
              <div key={index} className='card-container'>
                <div style={{ width: '190px' }}>
                  <Card
                    color={card.color}
                    value={card.value}
                    selectable={isMyTurn && canPlayCard(card)}
                    playable={isMyTurn && canPlayCard(card)}
                    onCardClick={() => toggleCardSelection(card)}
                    cardIsSelected={selectedCards.includes(card)}
                    flip={false}
                    rotationY={0}
                  />
                </div>
              </div>
            ))}
          </CardRow>
        </div>
      </div>
      {isMyTurn && (
        <>
          {showColorModal && (
            <div className='color-modal'>
              <h3>Select a color for the Wild card:</h3>
              <button onClick={() => handleColorSelect('Red')}>Red</button>
              <button onClick={() => handleColorSelect('Blue')}>Blue</button>
              <button onClick={() => handleColorSelect('Green')}>Green</button>
              <button onClick={() => handleColorSelect('Yellow')}>Yellow</button>
            </div>
          )}
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
