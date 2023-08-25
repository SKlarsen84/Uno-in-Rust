#[derive(Debug, Clone, PartialEq)]
pub enum Color {
    Red,
    Yellow,
    Green,
    Blue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(u8),
    Skip,
    Reverse,
    DrawTwo,
    Wild,
    WildDrawFour,
}

#[derive(Debug, Clone)]
pub struct Card {
    pub color: Option<Color>,
    pub value: Value,
}