
use crate::{
    card::{Card, Value},
    deck::Deck,
    lobby::Lobby,
    player::Player,
};

pub(crate) struct GameState {
    players: Vec<Player>, // Now just storing players

    deck: Deck,
    discard_pile: Vec<Card>,
    current_turn: usize,
    direction: i8, // 1 for clockwise, -1 for counter-clockwise
}

impl GameState {
    pub fn new(num_players: usize) -> Self {
        let mut deck = Deck::new();
        deck.shuffle();

        let mut players = Vec::new();

        let discard_pile = vec![deck.draw().unwrap()]; // Draw the initial card
        let current_turn = 0;
        let direction = 1;

        Self {
            players,
            deck,
            discard_pile,
            current_turn,
            direction,
        }
    }

    pub fn play_card(&mut self, player_id: usize, card: Card) -> Result<(), &'static str> {
        // Check if it's the player's turn
        if self.current_turn != player_id {
            return Err("Not your turn");
        }

        //check if the card is valid
        if !self.is_valid_play(&card) {
            return Err("Invalid play");
        }

        // Check if the card is in the player's hand
        let player = &mut self.players[player_id];
        if let Some(pos) = player.hand.iter().position(|&c| c == card) {
            self.apply_card_effect(&card);
            // Remove the card from the player's hand
            player.hand.remove(pos);

            // Add the card to the discard pile
            self.discard_pile.push(card);

            // Move to the next turn
            self.next_turn();

            Ok(())
        } else {
            Err("Card not in hand")
        }
    }

    pub fn next_turn(&mut self) {
        self.current_turn = (self.current_turn + self.direction as usize) % self.players.len();
    }

    pub fn apply_card_effect(&mut self, card: &Card) {
        match card.value {
            Value::Skip => self.next_turn(),
            Value::Reverse => self.direction *= -1,
            Value::DrawTwo => self.draw_cards(self.next_player_id(), 2),
            Value::Wild => {} // Handle Wild card (e.g., allow player to choose color)
            Value::WildDrawFour => {
                self.draw_cards(self.next_player_id(), 4);
                // Handle Wild Draw Four (e.g., allow player to choose color)
            }
            _ => {}
        }
    }

    pub fn next_player_id(&self) -> usize {
        (self.current_turn + self.direction as usize) % self.players.len()
    }

    pub fn draw_cards(&mut self, player_id: usize, count: usize) {
        let player = &mut self.players[player_id];
        player.hand.extend(self.deck.draw_n(count));
    }

    pub fn check_winner(&self) -> Option<usize> {
        for player in &self.players {
            if player.hand.is_empty() {
                return Some(player.id);
            }
        }
        None
    }

    pub fn draw_card(&mut self, player_id: usize) -> Result<(), &'static str> {
        if self.current_turn != player_id {
            return Err("Not your turn");
        }

        if self.deck.is_empty() {
            self.shuffle_discard_into_deck();
        }

        let player = &mut self.players[player_id];
        if let Some(card) = self.deck.draw() {
            player.hand.push(card);
            Ok(())
        } else {
            Err("Deck is empty")
        }
    }
    pub fn shuffle_discard_into_deck(&mut self) {
        let top_card = self.discard_pile.pop().unwrap();
        self.deck.cards.extend(self.discard_pile.drain(..));
        self.deck.shuffle();
        self.discard_pile.push(top_card);
    }

    pub fn is_valid_play(&self, card: &Card) -> bool {
        let top_card = self.discard_pile.last().unwrap();
        match card.value {
            Value::Wild | Value::WildDrawFour => true,
            _ => card.color == top_card.color || card.value == top_card.value,
        }
    }

    pub fn add_player(&mut self, player: Player) -> Result<(), &'static str> {
        if self.players.len() < 10 {
            self.players.push(player);
            Ok(())
        } else {
            Err("Game is full")
        }
    }

    pub fn remove_player(&mut self, player_id: usize) -> Result<(), &'static str> {
        if let Some(pos) = self.players.iter().position(|p| p.id == player_id) {
            self.players.remove(pos);
            Ok(())
        } else {
            Err("Player not found")
        }
    }

    pub fn get_player(&self, player_id: usize) -> Option<&Player> {
        self.players.iter().find(|p| p.id == player_id)
    }

    pub fn get_all_players(&self) -> &Vec<Player> {
        &self.players
    }
}
