use serde::{ Deserialize, Serializer, Serialize };
use serde::de::{ self, Deserializer, Visitor };
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Color {
    Red,
    Yellow,
    Green,
    Blue,
    Wild,
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

// Custom deserialization for Value
impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a card value")
            }

            fn visit_str<E>(self, value: &str) -> Result<Value, E> where E: de::Error {
                match value.to_lowercase().as_str() {
                    "skip" => Ok(Value::Skip),
                    "reverse" => Ok(Value::Reverse),
                    "draw_two" => Ok(Value::DrawTwo),
                    "wild" => Ok(Value::Wild),
                    "wild_draw_four" => Ok(Value::WildDrawFour),
                    num_str => {
                        if let Ok(num) = num_str.parse::<u8>() {
                            Ok(Value::Number(num))
                        } else {
                            Err(E::custom(format!("Unexpected value: {}", value)))
                        }
                    }
                }
            }
        }

        deserializer.deserialize_str(ValueVisitor)
    }
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
    pub id: usize,
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
