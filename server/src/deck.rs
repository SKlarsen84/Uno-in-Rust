use rand::seq::SliceRandom;

use crate::card::{ Card, Color };
use crate::card::Value;

pub(crate) struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::new();
        for color in &[Color::Red, Color::Yellow, Color::Green, Color::Blue] {
            // Add numbers from 0 to 9, twice each (except for 0)
            for value in 0..10 {
                let card = Card {
                    color: color.clone(),
                    value: Value::Number(value),
                };
                cards.push(card.clone());
                if value != 0 {
                    cards.push(card);
                }
            }
            // Add special cards (Skip, Reverse, DrawTwo), twice each
            for value in &[Value::Skip, Value::Reverse, Value::DrawTwo] {
                let card = Card {
                    color: color.clone(),
                    value: value.clone(),
                };
                cards.push(card.clone());
                cards.push(card);
            }
        }
        // Add Wild and Wild Draw Four cards, 4 each
        for value in &[Value::Wild, Value::WildDrawFour] {
            let card = Card {
                color: Color::Wild,
                value: value.clone(),
            };
            cards.push(card.clone());
            cards.push(card.clone());
            cards.push(card.clone());
            cards.push(card);
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
