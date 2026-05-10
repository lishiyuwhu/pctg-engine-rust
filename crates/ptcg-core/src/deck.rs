//! Deck definitions and building

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::card::{CardDefId, CardRegistry};

/// A card instance in a deck (card definition + quantity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckCard {
    pub card_id: CardDefId,
    pub count: u8,
}

/// Deck definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deck {
    pub name: String,
    pub deck_id: u32,
    pub cards: Vec<DeckCard>,
    pub total_cards: usize,
}

impl Deck {
    /// Create a new deck
    pub fn new(name: String, deck_id: u32) -> Self {
        Self {
            name,
            deck_id,
            cards: Vec::new(),
            total_cards: 0,
        }
    }

    /// Add a card to the deck
    pub fn add_card(&mut self, card_id: CardDefId, count: u8) {
        self.cards.push(DeckCard { card_id, count });
        self.total_cards += count as usize;
    }

    /// Validate the deck against a card registry
    pub fn validate(&self, registry: &CardRegistry) -> Result<(), DeckError> {
        if self.total_cards != 60 {
            return Err(DeckError::InvalidSize {
                expected: 60,
                actual: self.total_cards,
            });
        }

        let mut energy_count = 0;
        let mut basic_pokemon_count = 0;

        for deck_card in &self.cards {
            let card_def = registry
                .get(&deck_card.card_id)
                .ok_or(DeckError::UnknownCard(deck_card.card_id.clone()))?;

            if card_def.is_basic_energy() {
                energy_count += deck_card.count as usize;
            }

            if card_def.is_pokemon() && card_def.stage == Some(crate::card::Stage::Basic) {
                basic_pokemon_count += deck_card.count as usize;
            }
        }

        if basic_pokemon_count < 1 {
            return Err(DeckError::NoBasicPokemon);
        }

        if energy_count < 1 {
            return Err(DeckError::NoEnergy);
        }

        Ok(())
    }

    /// Expand deck into individual card instances
    pub fn expand(&self) -> Vec<CardDefId> {
        let mut cards = Vec::with_capacity(60);
        for deck_card in &self.cards {
            for _ in 0..deck_card.count {
                cards.push(deck_card.card_id.clone());
            }
        }
        cards
    }
}

impl fmt::Display for Deck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Deck: {} (ID: {})", self.name, self.deck_id)?;
        writeln!(f, "Total cards: {}", self.total_cards)?;
        for deck_card in &self.cards {
            writeln!(f, "  {} x{}", deck_card.card_id, deck_card.count)?;
        }
        Ok(())
    }
}

/// Deck validation errors
#[derive(Debug, thiserror::Error)]
pub enum DeckError {
    #[error("Invalid deck size: expected {expected}, got {actual}")]
    InvalidSize { expected: usize, actual: usize },

    #[error("Unknown card: {0}")]
    UnknownCard(CardDefId),

    #[error("Deck must contain at least one Basic Pokemon")]
    NoBasicPokemon,

    #[error("Deck must contain at least one Energy card")]
    NoEnergy,

    #[error("Unsupported card in training mode: {0}")]
    UnsupportedCard(CardDefId),
}

/// Match configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchConfig {
    pub player_deck: Deck,
    pub opponent_deck: Deck,
    pub player_name: String,
    pub opponent_name: String,
    pub starting_player: StartingPlayer,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            player_deck: templates::miraidon_deck(),
            opponent_deck: templates::charizard_pidgeot_deck(),
            player_name: "Player".into(),
            opponent_name: "Opponent".into(),
            starting_player: StartingPlayer::Random,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartingPlayer {
    Random,
    Player,
    Opponent,
}

/// Pre-built deck templates
pub mod templates {
    use super::*;

    pub fn miraidon_deck() -> Deck {
        let mut deck = Deck::new("Miraidon".into(), 575720);

        // Pokemon (15)
        deck.add_card(CardDefId::new("CSV1C", "050"), 2);  // Miraidon ex
        deck.add_card(CardDefId::new("CSV6C", "051"), 2);  // Iron Hands ex
        deck.add_card(CardDefId::new("CS4DaC", "137"), 1); // Raikou V
        deck.add_card(CardDefId::new("CSV2C", "105"), 1);  // Squawkabilly ex
        deck.add_card(CardDefId::new("CSV8C", "172"), 1);  // Bloodmoon Ursaluna ex
        deck.add_card(CardDefId::new("151C", "151"), 1);   // Mew ex
        deck.add_card(CardDefId::new("CS6aC", "057"), 1);  // Zapdos
        deck.add_card(CardDefId::new("CS6.5C", "020"), 1); // Radiant Greninja
        deck.add_card(CardDefId::new("CS5aC", "019"), 1);  // Raichu V
        deck.add_card(CardDefId::new("CS5bC", "052"), 1);  // Manaphy
        deck.add_card(CardDefId::new("CSV6C", "023"), 1);  // Rotom V
        deck.add_card(CardDefId::new("CS5bC", "049"), 1);  // Lumineon V
        deck.add_card(CardDefId::new("CSV8C", "135"), 1);  // Fezandipiti ex

        // Trainers (29)
        deck.add_card(CardDefId::new("CSV1C", "107"), 4);  // Electric Generator
        deck.add_card(CardDefId::new("CSV1C", "123"), 3);  // Arven
        deck.add_card(CardDefId::new("CSV3C", "123"), 1);  // Iono
        deck.add_card(CardDefId::new("CSVH1aC", "023"), 2); // Boss's Orders
        deck.add_card(CardDefId::new("CSV6C", "125"), 1);  // Professor Turo's Scenario
        deck.add_card(CardDefId::new("CSV1C", "112"), 4);  // Ultra Ball
        deck.add_card(CardDefId::new("CSV1C", "043"), 3);  // Nest Ball
        deck.add_card(CardDefId::new("CSV7C", "177"), 4);  // Buddy-Buddy Poffin
        deck.add_card(CardDefId::new("CSV6C", "060"), 2);  // Earthen Vessel
        deck.add_card(CardDefId::new("CSV6C", "114"), 1);  // Counter Catcher
        deck.add_card(CardDefId::new("CSV1C", "109"), 1);  // Super Rod
        deck.add_card(CardDefId::new("CS5.5C", "060"), 1); // Hisuian Heavy Ball
        deck.add_card(CardDefId::new("CSV8C", "183"), 1);  // Night Stretcher
        deck.add_card(CardDefId::new("CS5DC", "116"), 1);  // Switch Cart

        // Tools (3)
        deck.add_card(CardDefId::new("CSV7C", "066"), 1);  // Forest Seal Stone
        deck.add_card(CardDefId::new("CSV1C", "118"), 1);  // Bravery Charm
        deck.add_card(CardDefId::new("CSV7C", "185"), 1);  // Rescue Board

        // Energy (13)
        deck.add_card(CardDefId::new("CSVE1C", "LIG"), 13); // Lightning Energy

        deck
    }

    pub fn charizard_pidgeot_deck() -> Deck {
        let mut deck = Deck::new("Charizard Pidgeot".into(), 575716);
        
        // Pokemon (18)
        deck.add_card(CardDefId::new("CSV5C", "075"), 2);  // Charizard ex
        deck.add_card(CardDefId::new("CSV4C", "101"), 2);  // Pidgeot ex
        deck.add_card(CardDefId::new("151C", "004"), 3);   // Charmander
        deck.add_card(CardDefId::new("CSV5C", "015"), 1);  // Charmeleon
        deck.add_card(CardDefId::new("CSV4C", "099"), 2);  // Pidgey
        deck.add_card(CardDefId::new("CS5.5C", "032"), 2); // Duskull
        deck.add_card(CardDefId::new("CSV8C", "082"), 1);  // Dusclops
        deck.add_card(CardDefId::new("CSV8C", "083"), 1);  // Dusknoir
        deck.add_card(CardDefId::new("CS5bC", "049"), 1);  // Lumineon V
        deck.add_card(CardDefId::new("CS5bC", "052"), 1);  // Manaphy
        deck.add_card(CardDefId::new("CS5.5C", "007"), 1); // Radiant Charizard
        deck.add_card(CardDefId::new("CSV6C", "023"), 1);  // Rotom V
        
        // Trainers (38)
        deck.add_card(CardDefId::new("CSV1C", "123"), 4);  // Arven
        deck.add_card(CardDefId::new("CSV3C", "123"), 2);  // Iono
        deck.add_card(CardDefId::new("CSVH1aC", "023"), 2); // Boss's Orders
        deck.add_card(CardDefId::new("CSV6C", "125"), 1);  // Professor Turo's Scenario
        deck.add_card(CardDefId::new("CS6aC", "127"), 1);  // Thorton
        deck.add_card(CardDefId::new("CSV1C", "112"), 4);  // Ultra Ball
        deck.add_card(CardDefId::new("CSV1C", "043"), 3);  // Nest Ball
        deck.add_card(CardDefId::new("CSV7C", "177"), 4);  // Buddy-Buddy Poffin
        deck.add_card(CardDefId::new("CSVH1C", "045"), 4); // Rare Candy
        deck.add_card(CardDefId::new("CSV1C", "109"), 2);  // Super Rod
        deck.add_card(CardDefId::new("CSV8C", "183"), 1);  // Night Stretcher
        deck.add_card(CardDefId::new("CSV6bC", "123"), 1); // Lost Vacuum
        deck.add_card(CardDefId::new("CSV8C", "173"), 1);  // Unfair Stamp
        deck.add_card(CardDefId::new("CSV6C", "114"), 2);  // Counter Catcher
        
        // Tools (3)
        deck.add_card(CardDefId::new("CSV7C", "066"), 1);  // Forest Seal Stone
        deck.add_card(CardDefId::new("CSV1C", "117"), 1);  // Defiance Band
        deck.add_card(CardDefId::new("CS6.5C", "020"), 1); // Radiant Greninja
        
        // Stadium (1)
        deck.add_card(CardDefId::new("CS6.5C", "071"), 1); // Collapsed Stadium
        
        // Energy (6)
        deck.add_card(CardDefId::new("CSVE1C", "FIR"), 5); // Fire Energy
        deck.add_card(CardDefId::new("CSNC", "024"), 1);   // Double Turbo Energy
        
        deck
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::presets::load_miraidon_charizard_cards;

    #[test]
    fn test_miraidon_deck() {
        let deck = templates::miraidon_deck();
        assert_eq!(deck.total_cards, 60);
        
        let registry = load_miraidon_charizard_cards();
        deck.validate(&registry).expect("Deck should be valid");
    }

    #[test]
    fn test_charizard_deck() {
        let deck = templates::charizard_pidgeot_deck();
        assert_eq!(deck.total_cards, 60);
        
        let registry = load_miraidon_charizard_cards();
        deck.validate(&registry).expect("Deck should be valid");
    }

    #[test]
    fn test_deck_expand() {
        let deck = templates::miraidon_deck();
        let cards = deck.expand();
        assert_eq!(cards.len(), 60);
    }
}