use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Color {
    Red,
    Yellow,
    Green,
    Blue,
    Wild,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Value {
    Number(u8),
    Skip,
    Reverse,
    DrawTwo,
    Wild,
    WildDrawFour,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Card {
    pub color: Color,
    pub value: Value,
}