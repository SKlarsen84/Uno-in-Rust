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
        let first_card = cards.first().ok_or("No cards provided")?;
        self.validate_card_play(player_id, &first_card)?;

        //having made sure the first card is valid, we can now check that all cards in the vector are the same value as the first card
        if !cards.iter().all(|card| card.value == first_card.value) {
            println!("Invalid cards played - not all cards are the same value");
            return Err("Invalid cards");
        }

        //having made sure the first card is valid, we can now check that all cards in the vector are the same color as the first card

        // Find and remove the cards from the player's hand
        let mut played_cards: Vec<Card> = Vec::new();
        {
            if
                let Some(player_conn) = self.game_player_pool.connections
                    .iter_mut()
                    .find(|conn| conn.player.id == player_id)
            {
                for card in &cards {
                    if let Some(pos) = player_conn.player.hand.iter().position(|c| c.id == card.id) {
                        //we neeed to remove the card from the player's hand (by id) and patch by the played color before adding it to played_cards

                        //if the card is a wild, we need to set the color to the color of the card played
                        if card.value == Value::Wild || card.value == Value::WildDrawFour {
                            played_cards.push(Card {
                                id: card.id,
                                color: card.color.clone(),
                                value: card.value.clone(),
                            });
                        } else {
                            println!("Card played: {:?}", card);
                            played_cards.push(card.clone());
                        }
                        //ultimately, we need to remove the card from the player's hand
                        player_conn.player.hand.remove(pos);
                        println!("Player {} played card: {:?}", player_id, card);
                    } else {
                        println!("Card not in hand");
                        return Err("Card not in hand");
                    }
                }
            } else {
                return Err("Player not found");
            }
        }

        //if the player has no cards left, they win the round
        if self.game_player_pool.get_player_by_id(player_id).unwrap().hand.is_empty() {
            self.end_round();
            return Ok(());
        }

        //for each draw two in the played cards, we need to draw two cards for the next player
        for card in &played_cards {
            if card.value == Value::DrawTwo {
                let next_player_id = self.get_next_player_id();
                println!("passing 2 cards to next player id: {} ", &next_player_id.to_string());
                let _ = self.draw_cards(next_player_id, 2, false).await;
            }

            //if there's a wild draw four card in the played cards, we need to draw four cards for the next player
            if card.value == Value::WildDrawFour {
                let next_player_id = self.get_next_player_id();
                println!("passing 4 cards to next player id: {} ", &next_player_id.to_string());
                let _ = self.draw_cards(next_player_id, 4, false).await;
            }

            //if there's a skip card in the played cards, we need to skip the next player. This will skip the turn once more in the next_turn function

            if card.value == Value::Skip {
                //Set the player to play to the next player
                self.player_to_play = self.get_next_player_id();
            }

            //if there's a reverse card in the played cards, we need to reverse the direction of play
            if card.value == Value::Reverse {
                self.direction *= -1;
            }
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

        self.discard_pile.extend(played_cards);

        if let Some(_winner_id) = self.check_winner() {
            println!("Winner found");

            //create and broadcast a message to all players the id of the winning player
            let winner_json =
                json!({
                    "winner_id": player_id,
                }).to_string();
            let message = create_websocket_message("winner_found", &winner_json);
            self.game_player_pool.broadcast_message(message).await;
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

    pub fn shuffle_discard_into_deck(&mut self) {
        let top_card = self.discard_pile.pop().unwrap();
        self.deck.cards.extend(self.discard_pile.drain(..));
        self.deck.shuffle();
        //we need to make sure that every Wild card that has been played (with a color chosen) is reset back to Color: Wild.
        //This is because the color chosen is stored in the card itself, and we don't want to carry that over to the next round
        //find all cards with a value of Wild or WildDrawFour and set their color to Wild
        self.reset_played_wild_cards();
        self.discard_pile.push(top_card);
    }

    pub fn reset_played_wild_cards(&mut self) {
        for card in &mut self.deck.cards {
            if card.value == Value::Wild || card.value == Value::WildDrawFour {
                card.color = crate::card::Color::Wild;
            }
        }
    }
}
