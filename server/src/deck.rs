use rand::seq::SliceRandom;

use crate::card::{Card, Color};
use crate::card::Value;

pub(crate) struct Deck {
   pub cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::new();
        for mut color in &[Color::Red, Color::Yellow, Color::Green, Color::Blue, Color::Wild] {
            for value in 0..10 {
                cards.push(Card {
                    color: color.clone(),
                    value: Value::Number(value),
                });
            }
            // Add special cards like Skip, Reverse, etc.
        }
        Self { cards }
    }

    pub fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        self.cards.shuffle(&mut rng);
    }

    pub fn draw(&mut self) -> Option<Card> {
        self.cards.pop()
    }

    pub fn draw_n(&mut self, n: usize) -> Vec<Card> {
        (0..n).filter_map(|_| self.draw()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deck_new() {
        let deck = Deck::new();
        assert_eq!(deck.cards.len(), 108); // Assuming a standard UNO deck size
    }

    // Additional tests for Deck
}
