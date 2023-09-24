import { useEffect, useState } from 'react'
import { ICard, useWebSocket } from './WebSocketContext'
import { useNavigate } from 'react-router-dom'
import Card from './components/Card/Card'
import styled from 'styled-components'
import { Button } from '@nextui-org/button'
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
  const navigate = useNavigate()
  const [selectedCards, setSelectedCards] = useState<ICard[]>([])
  const [showColorModal, setShowColorModal] = useState(false)
  // In your GameView component
  const [cardsBeingPlayed, setCardsBeingPlayed] = useState<ICard[]>([])
  const [shouldAnimateAndSend, setShouldAnimateAndSend] = useState(false)

  const context = useWebSocket()
  const players = context?.players
  const ws = context?.ws
  const player = context?.player
  const gameState = context?.gameState
  const isMyTurn = context?.isMyTurn

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
      //set every card in the card to be played stacks as cards being playde
      setCardsBeingPlayed(selectedCards)

      //WAIT FOR ANIMATION TO FINISH
      setTimeout(() => {
        ws.send(JSON.stringify({ action: 'play_cards', cards: selectedCards, game_id: gameState?.id }))
        setSelectedCards([])
        setCardsBeingPlayed([])
      }, 1000)
    }
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
    <div style={{ padding: 50 }}>
      <div>
        {gameState?.discard_pile.length ? (
          <div>
            <div style={{ display: 'flex', justifyContent: 'center' }}>
              <div style={{ width: '110px' }} id='discard-pile'>
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
        <div style={{ display: 'flex', justifyContent: 'center', marginTop: 100 }}>
          <CardRow>
            {player?.hand?.map((card, index) => (
              <div key={`card_hand_index_${card.id}`} className='card-container'>
                <div style={{ width: '110px' }}>
                  <Card
                    id={card.id.toString()}
                    color={card.color}
                    value={card.value}
                    selectable={isMyTurn && canPlayCard(card)}
                    playable={isMyTurn && canPlayCard(card)}
                    onCardClick={() => toggleCardSelection(card)}
                    cardIsSelected={selectedCards.includes(card)}
                    flip={false}
                    rotationY={0}
                    cardBeingPlayed={cardsBeingPlayed.includes(card)}
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
              <Button onClick={() => handleColorSelect('Red')}>Red</Button>
              <Button onClick={() => handleColorSelect('Blue')}>Blue</Button>
              <Button onClick={() => handleColorSelect('Green')}>Green</Button>
              <Button onClick={() => handleColorSelect('Yellow')}>Yellow</Button>
            </div>
          )}
          <div>
            <Button disabled={selectedCards.length === 0} onClick={playSelectedCards}>
              Play Selected Cards
            </Button>
          </div>
          <div>
            <h2>Draw a card</h2>
            {isMyTurn ? <Button onClick={drawCard}>Draw Card</Button> : null}
          </div>
        </>
      )}
      <div>
        <Button onClick={() => navigate('/')}>Back to Lobby</Button>
      </div>
    </div>
  )
}

export default GameView
