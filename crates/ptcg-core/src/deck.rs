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


    pub fn gouging_fire_deck() -> Deck {
        let mut deck = Deck::new("gouging_fire".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CS5aC", "006"), 1),
            (CardDefId::new("CS6.5C", "020"), 1),
            (CardDefId::new("CSV7C", "109"), 1),
            (CardDefId::new("CS5DC", "126"), 1),
            (CardDefId::new("CSV4C", "129"), 2),
            (CardDefId::new("CSVH1C", "043"), 4),
            (CardDefId::new("CSV2C", "105"), 1),
            (CardDefId::new("CS6.5C", "012"), 1),
            (CardDefId::new("CSVE1C", "FIR"), 6),
            (CardDefId::new("CSVH1aC", "008"), 3),
            (CardDefId::new("CSV7C", "051"), 2),
            (CardDefId::new("CS5.5C", "060"), 1),
            (CardDefId::new("CS4DaC", "056"), 1),
            (CardDefId::new("CSV8C", "183"), 2),
            (CardDefId::new("CSVE1C", "DAR"), 3),
            (CardDefId::new("151C", "151"), 1),
            (CardDefId::new("CSV6C", "121"), 4),
            (CardDefId::new("CSV8C", "094"), 1),
            (CardDefId::new("CS5DC", "116"), 3),
            (CardDefId::new("CSVH1aC", "023"), 2),
            (CardDefId::new("CSV6C", "096"), 1),
            (CardDefId::new("CS6.5C", "066"), 1),
            (CardDefId::new("CSV1C", "112"), 3),
            (CardDefId::new("CSV1C", "111"), 1),
            (CardDefId::new("CSV8C", "135"), 1),
            (CardDefId::new("CSV1C", "121"), 1),
            (CardDefId::new("CSV3C", "123"), 1),
            (CardDefId::new("CSV1C", "118"), 2),
            (CardDefId::new("CS5DC", "152"), 4),
            (CardDefId::new("CSV6C", "115"), 3),
            (CardDefId::new("CSV8C", "176"), 1),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn future_box_deck() -> Deck {
        let mut deck = Deck::new("future_box".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CSV6C", "114"), 1),
            (CardDefId::new("CSVH1C", "043"), 1),
            (CardDefId::new("CSV1C", "107"), 4),
            (CardDefId::new("CSV7C", "033"), 1),
            (CardDefId::new("CSVE1C", "LIG"), 13),
            (CardDefId::new("CSV6C", "042"), 1),
            (CardDefId::new("151C", "151"), 1),
            (CardDefId::new("CSV7C", "180"), 1),
            (CardDefId::new("CSV6C", "119"), 4),
            (CardDefId::new("CSV1C", "123"), 4),
            (CardDefId::new("CSVE1C", "GRA"), 3),
            (CardDefId::new("CSV2C", "113"), 1),
            (CardDefId::new("CSV7C", "188"), 2),
            (CardDefId::new("CSV6C", "116"), 4),
            (CardDefId::new("CSVH1aC", "023"), 2),
            (CardDefId::new("CS6bC", "123"), 1),
            (CardDefId::new("CSV7C", "111"), 4),
            (CardDefId::new("CSV1C", "121"), 3),
            (CardDefId::new("CSV3C", "123"), 3),
            (CardDefId::new("CSV1C", "109"), 1),
            (CardDefId::new("CSV7C", "153"), 2),
            (CardDefId::new("CSV6C", "051"), 3),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn iron_thorns_deck() -> Deck {
        let mut deck = Deck::new("iron_thorns".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CSVH1C", "051"), 3),
            (CardDefId::new("CSV4C", "119"), 1),
            (CardDefId::new("CSVH1C", "047"), 2),
            (CardDefId::new("CSVE1C", "LIG"), 7),
            (CardDefId::new("CSV7C", "180"), 1),
            (CardDefId::new("CS5DC", "117"), 1),
            (CardDefId::new("CSV6C", "119"), 4),
            (CardDefId::new("CSV1C", "123"), 4),
            (CardDefId::new("151C", "132"), 1),
            (CardDefId::new("CS5aC", "113"), 2),
            (CardDefId::new("CSV2C", "113"), 4),
            (CardDefId::new("CSV1C", "108"), 4),
            (CardDefId::new("CS6bC", "130"), 3),
            (CardDefId::new("CSV6C", "116"), 2),
            (CardDefId::new("CSVH1aC", "023"), 3),
            (CardDefId::new("CS6bC", "123"), 2),
            (CardDefId::new("CSNC", "024"), 4),
            (CardDefId::new("CSV1C", "124"), 1),
            (CardDefId::new("CSV7C", "197"), 1),
            (CardDefId::new("CSV1C", "121"), 2),
            (CardDefId::new("CSV3C", "123"), 1),
            (CardDefId::new("CSV7C", "091"), 4),
            (CardDefId::new("CSV6C", "115"), 1),
            (CardDefId::new("CSV8C", "191"), 2),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn dialga_metang_deck() -> Deck {
        let mut deck = Deck::new("dialga_metang".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CS6aC", "083"), 4),
            (CardDefId::new("CS6.5C", "020"), 1),
            (CardDefId::new("CSVH1C", "043"), 4),
            (CardDefId::new("CSV7C", "191"), 1),
            (CardDefId::new("CS6.5C", "049"), 1),
            (CardDefId::new("CSVE1C", "MET"), 15),
            (CardDefId::new("151C", "151"), 1),
            (CardDefId::new("CSV7C", "180"), 1),
            (CardDefId::new("CSNC", "008"), 3),
            (CardDefId::new("CSV2C", "113"), 1),
            (CardDefId::new("CSVH1aC", "023"), 3),
            (CardDefId::new("CSV1C", "112"), 4),
            (CardDefId::new("CSV1C", "121"), 4),
            (CardDefId::new("CSV3C", "123"), 4),
            (CardDefId::new("CSV1C", "109"), 4),
            (CardDefId::new("CSV7C", "147"), 4),
            (CardDefId::new("CS5bC", "096"), 3),
            (CardDefId::new("CSV7C", "177"), 2),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn palkia_dusknoir_deck() -> Deck {
        let mut deck = Deck::new("palkia_dusknoir".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CSV6C", "114"), 1),
            (CardDefId::new("CS6.5C", "020"), 1),
            (CardDefId::new("CSVE1C", "WAT"), 7),
            (CardDefId::new("CSV2C", "028"), 1),
            (CardDefId::new("CSVH1C", "043"), 4),
            (CardDefId::new("CSV2C", "105"), 1),
            (CardDefId::new("CSV8C", "083"), 2),
            (CardDefId::new("CS5.5C", "060"), 1),
            (CardDefId::new("CSV8C", "183"), 4),
            (CardDefId::new("151C", "151"), 1),
            (CardDefId::new("CSV7C", "180"), 1),
            (CardDefId::new("CS5DC", "138"), 2),
            (CardDefId::new("CSV7C", "123"), 1),
            (CardDefId::new("CSNC", "003"), 2),
            (CardDefId::new("CS6.5C", "063"), 3),
            (CardDefId::new("CS5bC", "051"), 2),
            (CardDefId::new("CSV1C", "113"), 1),
            (CardDefId::new("CSV8C", "199"), 1),
            (CardDefId::new("CSVH1C", "034"), 1),
            (CardDefId::new("CS6bC", "123"), 1),
            (CardDefId::new("CSV1C", "112"), 4),
            (CardDefId::new("CSV8C", "135"), 1),
            (CardDefId::new("CSV8C", "082"), 1),
            (CardDefId::new("CSV3C", "123"), 4),
            (CardDefId::new("CSV8C", "081"), 3),
            (CardDefId::new("CSVH1C", "045"), 4),
            (CardDefId::new("CSV6C", "115"), 3),
            (CardDefId::new("CSV8C", "172"), 1),
            (CardDefId::new("CSV7C", "177"), 1),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn palkia_gholdengo_deck() -> Deck {
        let mut deck = Deck::new("palkia_gholdengo".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CS5bC", "052"), 1),
            (CardDefId::new("CSV4C", "089"), 4),
            (CardDefId::new("CS6.5C", "020"), 1),
            (CardDefId::new("CSVH1C", "051"), 1),
            (CardDefId::new("CSVE1C", "WAT"), 6),
            (CardDefId::new("CSVH1C", "043"), 4),
            (CardDefId::new("CSV7C", "191"), 3),
            (CardDefId::new("CS5.5C", "060"), 1),
            (CardDefId::new("CSVE1C", "MET"), 4),
            (CardDefId::new("SVP", "105"), 1),
            (CardDefId::new("CSV7C", "180"), 1),
            (CardDefId::new("CS5DC", "138"), 2),
            (CardDefId::new("CSV7C", "202"), 2),
            (CardDefId::new("CS5bC", "111"), 1),
            (CardDefId::new("CSNC", "003"), 1),
            (CardDefId::new("CS5aC", "113"), 1),
            (CardDefId::new("CS5bC", "051"), 1),
            (CardDefId::new("CSV6C", "125"), 1),
            (CardDefId::new("CSV1C", "113"), 1),
            (CardDefId::new("CSVH1C", "034"), 1),
            (CardDefId::new("CSV4C", "063"), 3),
            (CardDefId::new("CSVH1aC", "023"), 2),
            (CardDefId::new("CS6bC", "123"), 1),
            (CardDefId::new("CSV1C", "112"), 2),
            (CardDefId::new("CSV1C", "111"), 1),
            (CardDefId::new("CSV3C", "123"), 1),
            (CardDefId::new("CSV1C", "109"), 1),
            (CardDefId::new("CS5aC", "105"), 1),
            (CardDefId::new("CSV6C", "115"), 3),
            (CardDefId::new("CSV7C", "177"), 3),
            (CardDefId::new("CSV3C", "115"), 4),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn lost_box_deck() -> Deck {
        let mut deck = Deck::new("lost_box".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CS6.5C", "020"), 1),
            (CardDefId::new("CSV7C", "185"), 1),
            (CardDefId::new("CSVE1C", "WAT"), 3),
            (CardDefId::new("CSV5C", "128"), 1),
            (CardDefId::new("CS6bC", "122"), 4),
            (CardDefId::new("CSVH1C", "043"), 4),
            (CardDefId::new("CS5.5C", "060"), 1),
            (CardDefId::new("CSV7C", "161"), 2),
            (CardDefId::new("CSVE1C", "LIG"), 2),
            (CardDefId::new("CSVE1C", "DAR"), 3),
            (CardDefId::new("CSV6C", "042"), 1),
            (CardDefId::new("CS6bC", "026"), 1),
            (CardDefId::new("CS5bC", "128"), 1),
            (CardDefId::new("CS5DC", "116"), 4),
            (CardDefId::new("CS5.5C", "065"), 1),
            (CardDefId::new("CSVH1aC", "023"), 1),
            (CardDefId::new("CS6.5C", "023"), 1),
            (CardDefId::new("CS6bC", "123"), 2),
            (CardDefId::new("CSV6C", "096"), 1),
            (CardDefId::new("CS4DaC", "137"), 1),
            (CardDefId::new("CS6bC", "052"), 4),
            (CardDefId::new("CS6bC", "125"), 4),
            (CardDefId::new("CS6.5C", "066"), 1),
            (CardDefId::new("CSV1C", "112"), 1),
            (CardDefId::new("CSV8C", "135"), 1),
            (CardDefId::new("CSV1C", "109"), 4),
            (CardDefId::new("CSV8C", "176"), 1),
            (CardDefId::new("CSV6C", "051"), 1),
            (CardDefId::new("CSV8C", "172"), 1),
            (CardDefId::new("CSV7C", "162"), 2),
            (CardDefId::new("CSV7C", "177"), 4),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn regidrago_deck() -> Deck {
        let mut deck = Deck::new("regidrago".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CSVH1C", "043"), 4),
            (CardDefId::new("CSV2C", "105"), 1),
            (CardDefId::new("CSVE1C", "FIR"), 3),
            (CardDefId::new("CSVH1aC", "008"), 4),
            (CardDefId::new("CSV8C", "183"), 1),
            (CardDefId::new("CSV8C", "028"), 3),
            (CardDefId::new("151C", "151"), 1),
            (CardDefId::new("CSV8C", "203"), 1),
            (CardDefId::new("CSV7C", "180"), 1),
            (CardDefId::new("CSV8C", "159"), 2),
            (CardDefId::new("CS5bC", "128"), 1),
            (CardDefId::new("CSVE1C", "GRA"), 7),
            (CardDefId::new("CS5aC", "113"), 1),
            (CardDefId::new("CS6.5C", "055"), 3),
            (CardDefId::new("CSV1C", "079"), 1),
            (CardDefId::new("CSV1C", "113"), 1),
            (CardDefId::new("CSVH1aC", "023"), 2),
            (CardDefId::new("CS6bC", "108"), 1),
            (CardDefId::new("CS6.5C", "054"), 3),
            (CardDefId::new("CSV1C", "112"), 4),
            (CardDefId::new("CSV8C", "135"), 1),
            (CardDefId::new("CSV1C", "121"), 4),
            (CardDefId::new("CSV3C", "123"), 2),
            (CardDefId::new("CS5.5C", "053"), 1),
            (CardDefId::new("CSV1C", "109"), 1),
            (CardDefId::new("CSV6C", "115"), 4),
            (CardDefId::new("CSV8C", "153"), 1),
            (CardDefId::new("CSV3C", "115"), 1),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn lugia_archeops_deck() -> Deck {
        let mut deck = Deck::new("lugia_archeops".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CS6aC", "127"), 1),
            (CardDefId::new("CSV4C", "129"), 4),
            (CardDefId::new("CSV8C", "067"), 1),
            (CardDefId::new("CSV3C", "116"), 2),
            (CardDefId::new("CS6bC", "117"), 2),
            (CardDefId::new("CSV8C", "203"), 1),
            (CardDefId::new("CSV8C", "121"), 1),
            (CardDefId::new("CS5bC", "049"), 2),
            (CardDefId::new("CS6aC", "113"), 4),
            (CardDefId::new("CSV8C", "207"), 1),
            (CardDefId::new("CS6aC", "120"), 4),
            (CardDefId::new("CSV8C", "199"), 1),
            (CardDefId::new("CS6.5C", "072"), 1),
            (CardDefId::new("CSVH1aC", "023"), 3),
            (CardDefId::new("CSNC", "024"), 3),
            (CardDefId::new("CSV1C", "112"), 4),
            (CardDefId::new("CSV1C", "120"), 1),
            (CardDefId::new("CSV8C", "135"), 1),
            (CardDefId::new("CSV1C", "121"), 3),
            (CardDefId::new("CSV3C", "123"), 2),
            (CardDefId::new("CSV7C", "171"), 2),
            (CardDefId::new("CS6aC", "103"), 3),
            (CardDefId::new("CS6aC", "102"), 3),
            (CardDefId::new("CS6aC", "131"), 4),
            (CardDefId::new("CSV6C", "051"), 1),
            (CardDefId::new("CSV8C", "172"), 1),
            (CardDefId::new("CSV7C", "204"), 3),
            (CardDefId::new("CS6.5C", "071"), 1),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn arceus_giratina_deck() -> Deck {
        let mut deck = Deck::new("arceus_giratina".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CSVH1C", "051"), 4),
            (CardDefId::new("CSV4C", "129"), 1),
            (CardDefId::new("CSVH1C", "043"), 4),
            (CardDefId::new("CSV7C", "189"), 1),
            (CardDefId::new("CSV7C", "033"), 1),
            (CardDefId::new("CSV1C", "116"), 1),
            (CardDefId::new("CS5aC", "046"), 1),
            (CardDefId::new("CSVE1C", "PSY"), 3),
            (CardDefId::new("CS5bC", "111"), 2),
            (CardDefId::new("CSVE1C", "GRA"), 6),
            (CardDefId::new("CS6bC", "107"), 2),
            (CardDefId::new("CSV1C", "113"), 1),
            (CardDefId::new("CS6aC", "120"), 1),
            (CardDefId::new("CS6bC", "130"), 2),
            (CardDefId::new("CSVH1aC", "023"), 4),
            (CardDefId::new("CS6bC", "123"), 2),
            (CardDefId::new("CS6bC", "108"), 2),
            (CardDefId::new("CSNC", "009"), 4),
            (CardDefId::new("CS5aC", "107"), 3),
            (CardDefId::new("CSNC", "024"), 4),
            (CardDefId::new("CSV1C", "112"), 4),
            (CardDefId::new("CSV3C", "123"), 4),
            (CardDefId::new("CS5aC", "105"), 2),
            (CardDefId::new("CSV1C", "099"), 1),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn raging_bolt_deck() -> Deck {
        let mut deck = Deck::new("raging_bolt".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CS6.5C", "020"), 1),
            (CardDefId::new("CSVH1C", "043"), 4),
            (CardDefId::new("CSV2C", "105"), 1),
            (CardDefId::new("CSV6C", "082"), 1),
            (CardDefId::new("CS5.5C", "060"), 1),
            (CardDefId::new("CSVH1C", "047"), 2),
            (CardDefId::new("CSV8C", "183"), 2),
            (CardDefId::new("CSV8C", "028"), 3),
            (CardDefId::new("CSVE1C", "LIG"), 3),
            (CardDefId::new("CSV6C", "042"), 1),
            (CardDefId::new("CSV7C", "180"), 1),
            (CardDefId::new("CS5bC", "128"), 1),
            (CardDefId::new("CSVE1C", "GRA"), 6),
            (CardDefId::new("CSV6C", "121"), 4),
            (CardDefId::new("CS6.5C", "063"), 3),
            (CardDefId::new("CSV2C", "113"), 3),
            (CardDefId::new("CS5DC", "116"), 2),
            (CardDefId::new("CSVH1C", "034"), 2),
            (CardDefId::new("CSVH1aC", "023"), 1),
            (CardDefId::new("CS6bC", "123"), 1),
            (CardDefId::new("CSVE1C", "FIG"), 3),
            (CardDefId::new("CSV1C", "111"), 1),
            (CardDefId::new("CSV8C", "135"), 1),
            (CardDefId::new("CSV3C", "123"), 2),
            (CardDefId::new("CSV1C", "118"), 2),
            (CardDefId::new("CSV6C", "115"), 4),
            (CardDefId::new("CSV7C", "154"), 4),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn gardevoir_deck() -> Deck {
        let mut deck = Deck::new("gardevoir".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CS5bC", "052"), 1),
            (CardDefId::new("CSV6C", "114"), 2),
            (CardDefId::new("CS6.5C", "020"), 1),
            (CardDefId::new("CSV7C", "109"), 1),
            (CardDefId::new("CSV6C", "065"), 1),
            (CardDefId::new("CSVH1C", "043"), 1),
            (CardDefId::new("CS5aC", "043"), 3),
            (CardDefId::new("CS5.5C", "060"), 1),
            (CardDefId::new("CSV8C", "183"), 2),
            (CardDefId::new("CSVE1C", "PSY"), 7),
            (CardDefId::new("CSV5C", "119"), 2),
            (CardDefId::new("CSVE1C", "DAR"), 3),
            (CardDefId::new("CS6.5C", "030"), 4),
            (CardDefId::new("CSV1C", "123"), 4),
            (CardDefId::new("CSV8C", "094"), 2),
            (CardDefId::new("CSV6C", "125"), 1),
            (CardDefId::new("CSVH1aC", "023"), 1),
            (CardDefId::new("CSV2C", "060"), 1),
            (CardDefId::new("CSV1C", "060"), 1),
            (CardDefId::new("CSV1C", "112"), 2),
            (CardDefId::new("CSV3C", "123"), 4),
            (CardDefId::new("CSV2C", "055"), 2),
            (CardDefId::new("CSV2C", "127"), 2),
            (CardDefId::new("CSV1C", "109"), 1),
            (CardDefId::new("CSV1C", "118"), 2),
            (CardDefId::new("CSV2C", "053"), 1),
            (CardDefId::new("CSV6C", "115"), 2),
            (CardDefId::new("CSV8C", "176"), 1),
            (CardDefId::new("CSV7C", "177"), 4),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn blissey_tank_deck() -> Deck {
        let mut deck = Deck::new("blissey_tank".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CSVH1C", "043"), 3),
            (CardDefId::new("CSV4C", "119"), 2),
            (CardDefId::new("CS5.5C", "060"), 1),
            (CardDefId::new("CSV8C", "183"), 1),
            (CardDefId::new("CSVE1C", "PSY"), 2),
            (CardDefId::new("CSV8C", "165"), 3),
            (CardDefId::new("CSVE1C", "DAR"), 3),
            (CardDefId::new("CSV8C", "121"), 2),
            (CardDefId::new("CSV1C", "123"), 4),
            (CardDefId::new("CSV8C", "094"), 3),
            (CardDefId::new("CSV2C", "113"), 3),
            (CardDefId::new("151C", "113"), 4),
            (CardDefId::new("CS5.5C", "064"), 2),
            (CardDefId::new("CSV8C", "078"), 1),
            (CardDefId::new("CSVH1aC", "023"), 2),
            (CardDefId::new("CS6bC", "123"), 1),
            (CardDefId::new("CSNC", "024"), 4),
            (CardDefId::new("CSVE1C", "FIG"), 2),
            (CardDefId::new("CSV1C", "112"), 4),
            (CardDefId::new("CSV1C", "121"), 2),
            (CardDefId::new("CSV3C", "123"), 4),
            (CardDefId::new("CSV2C", "127"), 2),
            (CardDefId::new("CSV1C", "109"), 1),
            (CardDefId::new("CSV7C", "187"), 1),
            (CardDefId::new("CSV5C", "120"), 1),
            (CardDefId::new("CSV6C", "115"), 1),
            (CardDefId::new("CSV7C", "141"), 1),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn dragapult_banette_deck() -> Deck {
        let mut deck = Deck::new("dragapult_banette".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CS5bC", "052"), 1),
            (CardDefId::new("CSV6C", "114"), 2),
            (CardDefId::new("CSV7C", "194"), 1),
            (CardDefId::new("CSV7C", "185"), 1),
            (CardDefId::new("CSVH1C", "043"), 4),
            (CardDefId::new("CSVE1C", "FIR"), 3),
            (CardDefId::new("CSV8C", "183"), 1),
            (CardDefId::new("CSV1C", "115"), 1),
            (CardDefId::new("CSVE1C", "PSY"), 4),
            (CardDefId::new("CSV8C", "158"), 4),
            (CardDefId::new("CSV8C", "159"), 2),
            (CardDefId::new("CSV1C", "127"), 1),
            (CardDefId::new("CS5bC", "049"), 1),
            (CardDefId::new("CSV1C", "123"), 4),
            (CardDefId::new("CSV8C", "094"), 1),
            (CardDefId::new("CS6bC", "028"), 1),
            (CardDefId::new("CSV1C", "113"), 1),
            (CardDefId::new("CSV1C", "053"), 2),
            (CardDefId::new("CSVH1aC", "023"), 1),
            (CardDefId::new("CS6.5C", "023"), 1),
            (CardDefId::new("CS6.5C", "066"), 1),
            (CardDefId::new("CSV1C", "112"), 3),
            (CardDefId::new("CSV8C", "135"), 1),
            (CardDefId::new("CSV3C", "123"), 2),
            (CardDefId::new("CSV6C", "128"), 1),
            (CardDefId::new("CSVH1C", "045"), 1),
            (CardDefId::new("CSV5C", "120"), 1),
            (CardDefId::new("CSV6C", "115"), 2),
            (CardDefId::new("CSV8C", "157"), 4),
            (CardDefId::new("CSV7C", "178"), 1),
            (CardDefId::new("CSV7C", "177"), 4),
            (CardDefId::new("CSV1C", "054"), 2),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn dragapult_dusknoir_deck() -> Deck {
        let mut deck = Deck::new("dragapult_dusknoir".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CSV6C", "114"), 2),
            (CardDefId::new("CSV7C", "185"), 1),
            (CardDefId::new("CSV8C", "186"), 1),
            (CardDefId::new("CSVH1C", "043"), 3),
            (CardDefId::new("CSV8C", "160"), 1),
            (CardDefId::new("CSVE1C", "FIR"), 3),
            (CardDefId::new("CSV8C", "083"), 2),
            (CardDefId::new("CSV8C", "183"), 1),
            (CardDefId::new("CSVE1C", "PSY"), 3),
            (CardDefId::new("CSV8C", "158"), 3),
            (CardDefId::new("CSV8C", "159"), 3),
            (CardDefId::new("CS5bC", "128"), 1),
            (CardDefId::new("CS5bC", "049"), 1),
            (CardDefId::new("CSV1C", "123"), 4),
            (CardDefId::new("CS6bC", "028"), 1),
            (CardDefId::new("CSV1C", "113"), 1),
            (CardDefId::new("CS5.5C", "065"), 1),
            (CardDefId::new("CSVH1aC", "023"), 1),
            (CardDefId::new("CS6.5C", "023"), 1),
            (CardDefId::new("CS6.5C", "066"), 1),
            (CardDefId::new("CSV1C", "112"), 3),
            (CardDefId::new("CSV8C", "135"), 1),
            (CardDefId::new("CSV8C", "082"), 1),
            (CardDefId::new("CSV3C", "123"), 2),
            (CardDefId::new("CSV8C", "081"), 2),
            (CardDefId::new("CSVH1C", "045"), 4),
            (CardDefId::new("CSV5C", "120"), 1),
            (CardDefId::new("CSV6C", "115"), 2),
            (CardDefId::new("CSV8C", "157"), 4),
            (CardDefId::new("CSV7C", "177"), 4),
            (CardDefId::new("CSV5C", "127"), 1),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
        deck
    }

    pub fn dragapult_charizard_deck() -> Deck {
        let mut deck = Deck::new("dragapult_charizard".into(), 0);
        let cards: Vec<(CardDefId, u8)> = vec![
            (CardDefId::new("CS5bC", "052"), 1),
            (CardDefId::new("CSV6C", "114"), 2),
            (CardDefId::new("CSV7C", "185"), 1),
            (CardDefId::new("CSVH1C", "043"), 3),
            (CardDefId::new("CSVE1C", "FIR"), 5),
            (CardDefId::new("CS6.5C", "070"), 1),
            (CardDefId::new("CSV8C", "183"), 1),
            (CardDefId::new("CSVE1C", "PSY"), 2),
            (CardDefId::new("CSV5C", "119"), 1),
            (CardDefId::new("CSV8C", "158"), 3),
            (CardDefId::new("CSV8C", "159"), 2),
            (CardDefId::new("CS5bC", "128"), 1),
            (CardDefId::new("CS5bC", "049"), 1),
            (CardDefId::new("CSV1C", "123"), 4),
            (CardDefId::new("CS6bC", "028"), 1),
            (CardDefId::new("CSV6C", "125"), 1),
            (CardDefId::new("CSV5C", "075"), 2),
            (CardDefId::new("151C", "004"), 2),
            (CardDefId::new("CSV5C", "015"), 1),
            (CardDefId::new("CSVH1aC", "023"), 2),
            (CardDefId::new("CS6.5C", "023"), 1),
            (CardDefId::new("CS6bC", "123"), 1),
            (CardDefId::new("CS6.5C", "066"), 1),
            (CardDefId::new("CSV1C", "112"), 3),
            (CardDefId::new("CSV8C", "135"), 1),
            (CardDefId::new("CSV3C", "123"), 2),
            (CardDefId::new("CSV1C", "109"), 1),
            (CardDefId::new("CSV8C", "173"), 1),
            (CardDefId::new("CSVH1C", "045"), 3),
            (CardDefId::new("CSVH1C", "035"), 1),
            (CardDefId::new("CSV8C", "157"), 4),
            (CardDefId::new("CSV7C", "177"), 4),
        ];
        for (id, count) in cards {
            deck.add_card(id, count);
        }
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