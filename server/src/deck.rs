struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::new();
        for &color in &[Color::Red, Color::Yellow, Color::Green, Color::Blue] {
            for value in 0..10 {
                cards.push(Card {
                    color: Some(color),
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
