use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Red,
    Yellow,
    Green,
    Blue,
    Wild,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Number(u8),
    Skip,
    Reverse,
    DrawTwo,
    Wild,
    WildDrawFour,
}

impl Value {
    pub fn to_points(&self) -> i32 {
        match self {
            Value::Number(n) => *n as i32,
            Value::Skip => 20,
            Value::Reverse => 20,
            Value::DrawTwo => 20,
            Value::Wild => 50,
            Value::WildDrawFour => 50,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub color: Color,
    pub value: Value,
}
