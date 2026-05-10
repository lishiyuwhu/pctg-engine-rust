//! Effect dispatch system
//! Routes effect_id strings to actual effect functions

use super::EffectResult;
use crate::error::Result;
use crate::state::{GameState, PlayerId, SlotRef};

pub fn dispatch_ability(
    state: &mut GameState,
    player: PlayerId,
    source: SlotRef,
    effect_id: &str,
) -> Result<EffectResult> {
    match effect_id {
        "ability_tandem_unit" => super::pokemon::ability_tandem_unit(state, player, source),
        "ability_infernal_reign" => super::pokemon::ability_infernal_reign(state, player, source),
        "ability_wind_search" => super::pokemon::ability_wind_search(state, player, source),
        "ability_awaken" => Ok(EffectResult::new()),
        _ => Err(crate::error::EngineError::InvalidAction(format!(
            "Unknown ability effect: {}",
            effect_id
        ))),
    }
}

pub fn dispatch_trainer(
    state: &mut GameState,
    player: PlayerId,
    card_id: &str,
    choices: &crate::action::Choices,
) -> Result<EffectResult> {
    match card_id {
        "Electric Generator" | "electric_generator" => {
            super::trainers::effect_electric_generator(state, player, choices)
        }
        "Nest Ball" | "nest_ball" => super::trainers::effect_nest_ball(state, player),
        "Ultra Ball" | "ultra_ball" => {
            super::trainers::effect_ultra_ball(state, player, choices.selected_cards.clone())
        }
        "Rare Candy" | "rare_candy" => {
            let target = choices.selected_slots.first().copied().ok_or_else(|| {
                crate::error::EngineError::InvalidAction("Must select target".into())
            })?;
            super::trainers::effect_rare_candy(state, player, target)
        }
        "Arven" | "arven" => super::trainers::effect_arven(state, player),
        "Boss's Orders" | "boss_orders" | "Guzzlord" => {
            let target = choices.selected_slots.first().copied().ok_or_else(|| {
                crate::error::EngineError::InvalidAction("Must select target".into())
            })?;
            super::trainers::effect_boss_orders(state, player, target)
        }
        "Iono" | "iono" => super::trainers::effect_iono(state, player),
        _ => Err(crate::error::EngineError::InvalidAction(format!(
            "Unknown trainer effect: {}",
            card_id
        ))),
    }
}

pub fn dispatch_stadium(
    state: &mut GameState,
    player: PlayerId,
    card_id: &str,
) -> Result<EffectResult> {
    match card_id {
        _ => Ok(EffectResult::new()),
    }
}
