import styled from 'styled-components'

interface RootProps {
  ratio: number
}

const Root = styled.div<RootProps>`
  position: relative;
  padding-top: ${props => props.ratio * 100}%;

  img {
    position: absolute;
    width: 100%;
    height: 100%;
    top: 0;
    left: 0;
    object-fit: cover;
    cursor: inherit;
  }
`

export default function Image({ src = '', alt = ' ', ratio = 9 / 16, ...props }) {
  return (
    <Root ratio={ratio} {...props}>
      <img src={src} alt={alt} />
    </Root>
  )
}
