import styled from 'styled-components'
import Image from '../Image/Image'
import { motion, useAnimation } from 'framer-motion'
import { useEffect, useState } from 'react'
import { Button } from '@nextui-org/button'

type RootProps = {
  selectable?: boolean
  playable?: boolean
  disableShadow?: boolean
  color?: string
  width: number
}
const Root = styled.div<RootProps>`
  /* overflow: hidden; */
  padding-top: 141%;
  border-radius: calc(var(--cardWidth) / 10);

  box-shadow: ${(props: { disableShadow?: any }) => (!props.disableShadow ? '0 0 10px #292727' : 'none')};
  position: relative;
  transform-style: preserve-3d;

  cursor: ${(props: { playable?: any }) => (props.playable ? 'pointer' : 'inherit')};
  filter: ${(props: { selectable?: any; playable?: any }) =>
    props.selectable && !props.playable ? 'contrast(.5)' : 'none'};

  .front,
  .back {
    border-radius: calc(var(--cardWidth) / 10);
    background: whitesmoke;
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
    backface-visibility: hidden;
  }

  .front {
    transform: translateZ(1px);
    font-family: sans-serif;

    .value {
      position: absolute;
      top: 25%;
      left: 50%;
      transform: translate(-50%, -50%);
      color: var(--color);
      font-size: 64px;
      font-family: sans-serif !important;
      font-weight: bold;
    }

    .card-icon {
      width: 80%;
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
    }

    .value-small {
      position: absolute;
      color: white;
      -webkit-text-stroke: black 1.5px;
      font-weight: bold;
      font-size: 24px;
      font-style: italic;
      font-family: sans-serif !important;

      &.value-tl {
        top: 14px;
        left: 22px;
      }

      &.value-br {
        bottom: 14px;
        right: 22px;
        transform: scale(-1);
      }

      @media screen and (max-width: 1000px) {
        -webkit-text-stroke: black 1px;

        .value {
          text-shadow: 3px 3px black;
        }

        &.value-tl {
          top: 9px;
          left: 13px;
        }

        &.value-br {
          bottom: 9px;
          right: 13px;
          transform: scale(-1);
        }
      }
    }

    .icon-small {
      position: absolute;
      width: 20%;
      &.icon-tl {
        top: 25px;
        left: 20px;
      }

      &.icon-br {
        bottom: 25px;
        right: 20px;
        transform: scale(-1);
      }
      @media screen and (max-width: 1000px) {
        &.icon-tl {
          top: 14px;
          left: 11px;
        }

        &.icon-br {
          bottom: 14px;
          right: 11px;
          transform: scale(-1);
        }
      }
    }
  }

  .back {
    transform: rotateY(180deg);
  }
`

interface CardProps {
  id?: string
  color?: string
  value?: string
  flip?: boolean
  rotationY?: number
  layoutId?: string
  selectable?: boolean
  playable?: boolean
  disableShadow?: boolean
  cardIsSelected?: boolean
  cardBeingPlayed?: boolean
  onCardClick?: () => void
}

export default function Card({
  id = '',
  color = '',
  value = '',
  flip = false,
  rotationY = 180,
  layoutId,
  selectable,
  playable,
  disableShadow,
  cardIsSelected,
  cardBeingPlayed,
  onCardClick = () => {}
}: CardProps) {
  const controls = useAnimation()

  const [checkAnimation, setCheckAnimation] = useState(false)

  useEffect(() => {
    console.log('useEffect triggered')
    console.log('Current ID:', id)
    console.log('Cards being played:', cardBeingPlayed)

    if (cardBeingPlayed) {
      const cardElement = document.getElementById('card_id_' + id)
      const discardPileElement = document.getElementById('discard-pile')

      if (cardElement && discardPileElement) {
        console.log(`Animating card with ID: ${id}`)
        console.log('Card Element:', cardElement)
        console.log('Discard Pile Element:', discardPileElement)

        const cardRect = cardElement.getBoundingClientRect()
        const discardPileRect = discardPileElement.getBoundingClientRect()

        const moveToX = discardPileRect.x - cardRect.x
        const moveToY = discardPileRect.y - cardRect.y
        controls.start({
          x: moveToX,
          y: moveToY,
          transition: { duration: 0.5 }
        })
      }
    }
  }, [cardBeingPlayed, controls, id])

  const getFrontContent = () => {
    if (color.toLowerCase() === 'wild' && value.toLowerCase() === 'wild')
      return (
        <>
          <Image src={`../assets/images/wild.png`} ratio={590 / 418} />
        </>
      )

    if (color === 'wild' && value === 'wild_draw_four')
      return (
        <>
          <Image src={`../assets/images/front-${color}.png`} ratio={590 / 418} />
          <img src='../assets/images/draw4.png' className='card-icon' alt='' />
          <img className='icon-small icon-tl' src={`../assets/images/${value}-blank.png`} alt='' />
          <img className='icon-small icon-br' src={`../assets/images/${value}-blank.png`} alt='' />
        </>
      )

    if (
      value === 'draw_two' ||
      value === 'skip' ||
      value === 'reverse' ||
      value === 'wild' ||
      value === 'wild_draw_four'
    )
      return (
        <>
          <Image src={`../assets/images/front-${color.toLowerCase()}.png`} ratio={590 / 418} />
          <img
            src={`../assets/images/${value.toLowerCase()}-${color.toLowerCase()}.png`}
            className='card-icon'
            alt=''
          />
          <img className='icon-small icon-tl' src={`../assets/images/${value.toLowerCase()}-blank.png`} alt='' />
          <img className='icon-small icon-br' src={`../assets/images/${value.toLowerCase()}-blank.png`} alt='' />
        </>
      )
    return (
      <>
        <Image src={`../assets/images/front-${color.toLowerCase()}.png`} ratio={590 / 418} />
        <p className='value'>{value.toLowerCase()}</p>
        <p className='value-small value-tl'>{value.toLowerCase()}</p>
        <p className='value-small value-br'>{value.toLowerCase()}</p>
      </>
    )
  }

  return (
    <motion.div initial={{ x: 0, y: 0 }} animate={controls}>
      <Root
        as={motion.div}
        color={color}
        className='noselect'
        layoutId={layoutId}
        initial={{
          rotateY: flip ? Math.abs(180 - rotationY) : rotationY,
          y: 0
        }}
        width={200}
        whileHover={selectable ? { y: -40, transition: { duration: 0.3 } } : { y: 0, transition: { duration: 0.3 } }}
        animate={{
          rotateY: rotationY,
          y: cardIsSelected ? -40 : 0 // Raise the card if it's selected
        }}
        transition={{ duration: 0.5, ease: 'easeInOut' }}
        selectable={selectable}
        playable={selectable}
        disableShadow={false}
        onClick={onCardClick}
        //if the card is selected, add a border and keep it a little raised
        style={{
          border: cardIsSelected ? '2px solid white' : 'none'
        }}
      >
        <div className='front' id={`card_id_${id}`}>
          {getFrontContent()}
        </div>
        <div className='back'>
          <Image src={`../assets/images/backside.png`} ratio={590 / 418} />
        </div>
      </Root>
      <Button onClick={() => setCheckAnimation(!checkAnimation)}>Click me</Button>
    </motion.div>
  )
}
