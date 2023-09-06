use serde::{ Deserialize, Serializer, Serialize };

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Red,
    Yellow,
    Green,
    Blue,
    Wild,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
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

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            Value::Number(n) => serializer.serialize_str(&n.to_string()),
            Value::Skip => serializer.serialize_str("skip"),
            Value::Reverse => serializer.serialize_str("reverse"),
            Value::DrawTwo => serializer.serialize_str("draw_two"),
            Value::Wild => serializer.serialize_str("wild"),
            Value::WildDrawFour => serializer.serialize_str("wild_draw_four"),
        }
    }
}
