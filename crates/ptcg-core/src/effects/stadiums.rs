//! Stadium card effects

use crate::card::CardDefId;
use crate::state::{GameState, PlayerId};

/// Artazon - Psychic Pokemon have 0 retreat cost
pub fn effect_artazon(state: &GameState, player: PlayerId) -> bool {
    let player_state = &state.players[player.0];

    if let Some(stadium) = &player_state.stadium {
        if let Some(card) = state.get_card(stadium.card_id) {
            if card.def_id.0 == "CSV2C_127" {
                return true;
            }
        }
    }

    false
}

/// Collapsed Stadium / Gravity Mountain - Reduce max bench size
pub fn effect_bench_limit(state: &GameState, player: PlayerId) -> usize {
    let player_state = &state.players[player.0];

    if let Some(stadium) = &player_state.stadium {
        if let Some(card) = state.get_card(stadium.card_id) {
            let stadium_id = &card.def_id.0;
            if stadium_id == "CS6.5C_071" || stadium_id == "CSV7C_201" {
                return 3; // Reduced bench size
            }
        }
    }

    crate::MAX_BENCH_SIZE
}

/// Lost City - KO'd Pokemon go to lost zone
pub fn effect_lost_city(state: &GameState, player: PlayerId) -> bool {
    let player_state = &state.players[player.0];

    if let Some(stadium) = &player_state.stadium {
        if let Some(card) = state.get_card(stadium.card_id) {
            if card.def_id.0 == "CSV6bC_130" {
                return true;
            }
        }
    }

    false
}
