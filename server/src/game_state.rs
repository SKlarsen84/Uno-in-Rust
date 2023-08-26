use crate::{
    card::{Card, Value},
    deck::Deck,
    player::Player,
};

pub struct GameState {
    pub players: Vec<Player>, // Now just storing players

    deck: Deck,
    discard_pile: Vec<Card>,
    current_turn: usize,
    direction: i8, // 1 for clockwise, -1 for counter-clockwise
    pub round_in_progress: bool,
    pub is_waiting_for_players: bool,
}

impl GameState {
    pub fn new(_num_players: usize) -> Self {
        let mut deck = Deck::new();
        deck.shuffle();

        let players = Vec::new();

        let discard_pile = vec![deck.draw().unwrap()]; // Draw the initial card
        let current_turn = 0;
        let direction = 1;

        Self {
            players,
            deck,
            discard_pile,
            current_turn,
            direction,
            round_in_progress: false,
            is_waiting_for_players: true,
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

        // Step 1: Find the position of the card in the player's hand
        let pos_option = {
            let player = &self.players[player_id];
            player.hand.iter().position(|c| *c == card)
        };

        // Check if the card is in the player's hand
        if let Some(pos) = pos_option {
            self.apply_card_effect(&card);
            let player = &mut self.players[player_id];
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

    pub fn add_player(&mut self, mut player: Player) -> Result<(), &'static str> {
        if self.players.len() >= 6 {
            return Err("Game is full");
        }

        if self.round_in_progress {
            player.is_spectator = true;
        }

        if !self.round_in_progress {
            player.set_hand(self.deck.draw_n(7)); // Draw 7 cards for the new player

            self.players.push(player);
            Ok(())
        } else {
            Err("Could not add player")
        }
    }

    pub fn remove_player(&mut self, player_id: usize) -> Result<(), &'static str> {
        if let Some(pos) = self.players.iter().position(|p| p.id == player_id) {
            self.players[pos].hand.clear();
            self.players.remove(pos);

            if self.players.len() == 1 {
                self.go_into_waiting_state();
            }
            Ok(())
        } else {
            Err("Player not found")
        }
    }

    pub fn go_into_waiting_state(&mut self) {
        self.is_waiting_for_players = true;
        self.round_in_progress = false;
        // ... other resets
    }

    pub fn get_player(&self, player_id: usize) -> Option<&Player> {
        self.players.iter().find(|p| p.id == player_id)
    }

    pub fn get_all_players(&self) -> &Vec<Player> {
        &self.players
    }

    pub fn check_and_start_round(&mut self) {
        if self.players.len() >= 2 && !self.round_in_progress {
            self.is_waiting_for_players = false;
            self.start_round();
        }
    }

    pub fn start_round(&mut self) {
        if self.players.len() >= 2 {
            self.round_in_progress = true;
            self.direction = 1;
            self.deck = Deck::new();
            self.deck.shuffle();
            self.discard_pile = vec![self.deck.draw().unwrap()];
            self.current_turn = 0;

            for player in &mut self.players {
                player.is_spectator = false;
            }

            for player in &mut self.players {
                player.set_hand(self.deck.draw_n(7));
            }
        }
    }

    pub fn end_round(&mut self) {
        self.round_in_progress = false;
        // Reset game state for next round
    }

    pub fn end_game(&mut self) {
        self.round_in_progress = false;
        self.players.clear();
        self.deck = Deck::new();
        self.deck.shuffle();
        self.discard_pile = vec![self.deck.draw().unwrap()];
        self.current_turn = 0;
    }
}
