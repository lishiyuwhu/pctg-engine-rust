//! Tool card effects

use super::EffectResult;
use crate::error::Result;
use crate::state::{GameState, PlayerId, SlotRef};

/// Rescue Board - Retreat -1, or 0 if HP <= 120
pub fn effect_rescue_board(state: &mut GameState, player: PlayerId, slot: SlotRef) -> bool {
    let player_state = &state.players[player.0];

    if let Some(pokemon) = player_state.get_slot(slot) {
        if let Some(card_id) = pokemon.top_card() {
            if let Some(card_def) = state.get_card_def(card_id) {
                if let Some(max_hp) = card_def.hp {
                    return pokemon.damage + 120 >= max_hp;
                }
            }
        }
    }

    false
}

/// Defiance Band - +40 damage when hit
pub fn effect_defiance_band() -> i32 {
    40
}

/// Maximum Belt - +50 damage vs ex Pokemon
pub fn effect_maximum_belt() -> i32 {
    50
}

/// Choice Belt - +30 damage vs V Pokemon
pub fn effect_choice_belt() -> i32 {
    30
}

/// Forest Seal Stone - VSTAR power once per game
pub fn effect_forest_seal_stone() -> bool {
    true // Effect handled by VSTAR power tracking
}

/// Heavy Baton - Transfer energy on KO
pub fn effect_heavy_baton(
    state: &mut GameState,
    player: PlayerId,
    slot: SlotRef,
) -> Vec<crate::state::CardInstanceId> {
    let player_state = &mut state.players[player.0];

    if let Some(pokemon) = player_state.get_slot_mut(slot) {
        let energies: Vec<_> = pokemon.energies.drain(..).collect();
        return energies;
    }

    Vec::new()
}
