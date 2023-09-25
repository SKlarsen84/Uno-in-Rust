import { useEffect, useState } from 'react'
import { ICard, useWebSocket } from './WebSocketContext'
import { useNavigate } from 'react-router-dom'
import Card from './components/Card/Card'
import styled from 'styled-components'
import { Button } from '@nextui-org/button'
import Image from './components/Image/Image'
import { table } from 'console'
// import Card from './components/Card/Card'

interface CardRowProps {
  cardsCnt?: number
  highlight?: boolean
}

const CardRow = styled.div<CardRowProps>`
  filter: ${props => (props.highlight ? 'drop-shadow(0 0 10px white)' : 'brightness(0.6)')};
  display: flex;
  justify-content: center;
  .card-container {
    &:not(:last-of-type) {
      margin-right: -55px; // Assuming each card is 110px wide, 50% overlap would be -55px
    }
  }
`

// For the opponent's hand, cards will be stacked tighter
const OpponentCardRow = styled.div<CardRowProps>`
  filter: ${props => (props.highlight ? 'drop-shadow(0 0 10px white)' : 'brightness(0.6)')};
  display: flex;
  justify-content: center;
  .card-container {
    &:not(:last-of-type) {
      margin-right: -35px; // Assuming each card is 110px wide, 50% overlap would be -55px
    }
  }
`

const Table = styled.div`
  position: relative;
  width: 800px;
  height: 800px;
  margin: auto;
  border-radius: 50%;
  background-color: green;
  z-index: 0;
`

const DiscardPile = styled.div`
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 1;
`

const PlayerSeat = styled.div`
  position: absolute;
  bottom: 0;
  left: 50%; // Center the div horizontally
  transform-origin: 0 -400px; // Set the rotation point to the center of the table
  z-index: 2;
`

const OpponentCard = styled.div`
  border-radius: 7px;
  background: #ffffff;
  box-shadow: 0 0 10px #292727

  top: 0;
  left: 0;
  width: 100%;
  height: 100px;
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

  const currentPlayerIndex = players?.findIndex(p => p.id === player?.id) || 0

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

  useEffect(() => {
    console.log('Current players state:', players)
  }, [players])

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
      <Table>
        <DiscardPile>
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
        </DiscardPile>

        {players?.map((tablePlayer, index, arr) => {
          const isCurrentPlayer = tablePlayer.id === player?.id
          if (isCurrentPlayer) {
            return (
              <PlayerSeat key={tablePlayer.id} style={{ transform: 'translateX(-50%)' }}>
                <CardRow highlight={gameState?.player_to_play === player.id}>
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
              </PlayerSeat>
            )
          }

          const relativeIndex = (index - currentPlayerIndex + arr.length) % arr.length
          const angleSpan = 180
          const angleOffset = 90
          const angle = angleOffset + (angleSpan / (arr.length - 1)) * relativeIndex
          const positionStyle = {
            transform: `rotate(${angle}deg) translateX(-50%)`
          }
          return (
            <PlayerSeat key={tablePlayer.id} style={positionStyle}>
              {/* spawn a row of cards same length as the players_hand count */}
              <OpponentCardRow
                cardsCnt={tablePlayer.card_count}
                highlight={gameState?.player_to_play === tablePlayer.id}
              >
                {Array.from(Array(tablePlayer.card_count).keys()).map((_, index) => (
                  <div key={`card_hand_index_${index}`} className='card-container'>
                    <div style={{ width: '60px' }}>
                      <Card />
                    </div>
                  </div>
                ))}
              </OpponentCardRow>
            </PlayerSeat>
          )
        })}

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
            <div>{isMyTurn ? <Button onClick={drawCard}>Draw Card</Button> : null}</div>
          </>
        )}
        <div>
          <Button onClick={() => navigate('/')}>Back to Lobby</Button>
        </div>
      </Table>
    </div>
  )
}

export default GameView
