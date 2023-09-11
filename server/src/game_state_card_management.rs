use serde_json::json;

use crate::{ game_state::GameState, card::{ Card, Value }, websocket::create_websocket_message };

impl GameState {
    pub async fn draw_cards(
        &mut self,
        player_id: usize,
        count: usize,
        advance_turn: bool
    ) -> Result<(), &'static str> {
        println!("Player {} drawing {} cards", player_id, count);

        let mut cards_to_draw = Vec::new();

        // Draw 'count' number of cards and store them temporarily
        for _ in 0..count {
            if self.deck.is_empty() {
                self.shuffle_discard_into_deck();
            }
            if let Some(card) = self.deck.draw() {
                println!("Drew card: {:?} for player_id {}", card, player_id);
                cards_to_draw.push(card);
            }
        }

        // Now, add the drawn cards to the player's hand
        if let Some(player) = self.get_player_by_id_mut(player_id) {
            player.hand.extend(cards_to_draw);
        } else {
            // Handle the case where the player is not found, if needed
            println!("Player not found");
        }

        if advance_turn {
            self.next_turn().await;
        }

        Ok(())
    }

    pub async fn play_cards(
        &mut self,
        player_id: usize,
        cards: Vec<Card>
    ) -> Result<(), &'static str> {
        // Validation: All cards must have the same value and be valid plays
        let first_card = cards.first().ok_or("No cards provided")?;
        self.validate_card_play(player_id, &first_card)?;

        //having made sure the first card is valid, we can now check that all cards in the vector are the same value as the first card
        if !cards.iter().all(|card| card.value == first_card.value && self.is_valid_play(card)) {
            return Err("Invalid cards");
        }

        // Find and remove the cards from the player's hand
        let mut played_cards: Vec<Card> = Vec::new();
        {
            if
                let Some(player_conn) = self.game_player_pool.connections
                    .iter_mut()
                    .find(|conn| conn.player.id == player_id)
            {
                for card in &cards {
                    if let Some(pos) = player_conn.player.hand.iter().position(|c| c == card) {
                        played_cards.push(player_conn.player.hand.remove(pos));
                    } else {
                        return Err("Card not in hand");
                    }
                }
            } else {
                return Err("Player not found");
            }
        }

        //if there's a draw two card in the played cards, we need to draw two cards for the next player
        if played_cards.iter().any(|card| card.value == Value::DrawTwo) {
            let next_player_id = self.get_next_player_id();
            let _ = self.draw_cards(next_player_id, 2, false).await;

            //we need to update the next player's hand for them via the pool connection
            let _ = self.update_single_player(
                &self.game_player_pool.get_player_by_id(next_player_id).unwrap()
            ).await;
        }

        //if there's a wild draw four card in the played cards, we need to draw four cards for the next player
        if played_cards.iter().any(|card| card.value == Value::WildDrawFour) {
            let next_player_id = self.get_next_player_id();
            let _ = self.draw_cards(next_player_id, 4, false).await;

            //we need to update the next player's hand for them via the pool connection
            let _ = self.update_single_player(
                &self.game_player_pool.get_player_by_id(next_player_id).unwrap()
            ).await;
        }

        let _ = self.update_single_player(
            &self.game_player_pool.get_player_by_id(player_id).unwrap()
        ).await;

        let played_cards_json =
            json!({
                "player_id": player_id,
                "cards": played_cards,
            }).to_string();
        let message = create_websocket_message("card_played", &played_cards_json);
        self.game_player_pool.send_message(
            &self.game_player_pool.get_player_by_id(player_id).unwrap(),
            message
        ).await;

        //push the played cards to the discard pile with the last card played on top
        self.discard_pile.extend(played_cards);

        if let Some(_winner_id) = self.check_winner() {
            self.end_round();
            return Ok(());
        }

        self.next_turn().await;
        Ok(())
    }

    // Simplified card validation in play_card
    fn validate_card_play(&self, player_id: usize, card: &Card) -> Result<(), &'static str> {
        if self.player_to_play != player_id {
            println!("Not your turn");
            return Err("Not your turn");
        }
        if !self.is_valid_play(&card) {
            println!("Invalid play for card: {:?}", card);
            return Err("Invalid play");
        }
        Ok(())
    }
}
