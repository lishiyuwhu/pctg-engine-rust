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
        "ability_concealed_cards" => super::pokemon::ability_concealed_cards(state, player, source),
        "ability_restart" => super::pokemon::ability_restart(state, player, source),
        "ability_burst_roar" => super::pokemon::ability_burst_roar(state, player, source),
        "tool_star_alchemy" => super::pokemon::ability_star_alchemy(state, player, source),
        "ability_quick_draw" => Ok(EffectResult::new()), // Iron Leaves — bench-in switch (simplified no-op)
        "ability_squawk_and_seize" => super::pokemon::ability_squawk_and_seize(state, player, source),
        "ability_luminous_sign" => super::pokemon::ability_luminous_sign(state, player, source),
        "ability_instant_charge" => super::pokemon::ability_instant_charge(state, player, source),
        "ability_flip_the_script" => super::pokemon::ability_flip_the_script(state, player, source),
        "ability_iron_bundle_blower" => Ok(EffectResult::new()), // Iron Bundle — gust from bench (simplified no-op)
        "ability_azure_command" => Ok(EffectResult::new()), // Iron Crown — future damage boost (passive)
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
        "Buddy-Buddy Poffin" | "buddy_poffin" | "buddy-buddy poffin" => {
            super::trainers::effect_buddy_poffin(state, player)
        }
        "Super Rod" | "super_rod" => super::trainers::effect_super_rod(state, player),
        "Counter Catcher" | "counter_catcher" => {
            let target = choices.selected_slots.first().copied().ok_or_else(|| {
                crate::error::EngineError::InvalidAction("Must select target".into())
            })?;
            super::trainers::effect_boss_orders(state, player, target) // same mechanic
        }
        _ => Err(crate::error::EngineError::InvalidAction(format!(
            "Unknown trainer effect: {}",
            card_id
        ))),
    }
}

pub fn dispatch_attack(
    state: &mut crate::state::GameState,
    attacker: crate::state::PlayerId,
    defender: crate::state::PlayerId,
    effect_id: &str,
    base_damage: u16,
    choices: &crate::action::Choices,
    rng: &mut crate::rng::GameRng,
) -> Result<super::AttackResult> {
    match effect_id {
        "attack_bench_snipe_30" => {
            super::pokemon::attack_double_impact(state, attacker, defender, base_damage, choices)
        }
        "attack_prize_count_damage" => {
            super::pokemon::attack_scorching_darkness(state, attacker, defender, base_damage)
        }
        "attack_self_lock_next_turn" => {
            super::pokemon::attack_photon_blaster(state, defender, base_damage)
        }
        "attack_bench_snipe_double_90" => {
            super::pokemon::attack_moonlight_shuriken(state, attacker, defender, base_damage, choices)
        }
        "attack_optional_discard_stadium" => {
            super::pokemon::attack_gale_winds(state, attacker, defender, base_damage, choices)
        }
        "attack_prize_condition_damage" => {
            super::pokemon::attack_combustion_blast(state, attacker, defender, base_damage)
        }
        "attack_discard_energy_from_self" => {
            super::pokemon::attack_discard_energy_from_self(state, attacker, defender, base_damage)
        }
        "attack_discard_stadium_bonus" => {
            super::pokemon::attack_discard_stadium_bonus(state, attacker, defender, base_damage, choices)
        }
        _ => Ok(super::AttackResult::new(base_damage)),
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
