//! Energy card effects

use super::EffectResult;
use crate::card::EnergyType;
use crate::error::Result;
use crate::state::{GameState, PlayerId, SlotRef};

/// Double Turbo Energy - Provides 2 Colorless, -20 damage
pub fn effect_double_turbo_energy() -> (Vec<EnergyType>, i32) {
    (vec![EnergyType::Colorless, EnergyType::Colorless], -20)
}

/// Jet Energy - Provides 1 Colorless, can retreat when attached
pub fn effect_jet_energy() -> Vec<EnergyType> {
    vec![EnergyType::Colorless]
}

/// Get energy provided by an attached energy card
pub fn get_provided_energy(
    state: &GameState,
    card_id: crate::state::CardInstanceId,
) -> Vec<EnergyType> {
    if let Some(card_def) = state.get_card_def(card_id) {
        if let Some(provides) = &card_def.provides_energy {
            return provides.clone();
        }
    }
    Vec::new()
}

/// Get damage modifier from an attached energy card
pub fn get_energy_damage_modifier(state: &GameState, card_id: crate::state::CardInstanceId) -> i32 {
    if let Some(card_def) = state.get_card_def(card_id) {
        return card_def.damage_modifier.unwrap_or(0);
    }
    0
}
