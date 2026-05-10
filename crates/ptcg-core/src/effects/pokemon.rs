//! Pokemon ability and attack effects

use super::EffectResult;
use crate::card::{CardDef, EnergyType};
use crate::error::Result;
use crate::state::{CardInstanceId, GameState, PlayerId, PokemonSlot, SlotRef};

/// Miraidon ex - Tandem Unit ability
/// Once per turn, search up to 2 basic Lightning Pokemon from deck to bench
pub fn ability_tandem_unit(
    state: &mut GameState,
    player: PlayerId,
    source: SlotRef,
) -> Result<EffectResult> {
    // Check if already used
    if let Some(slot) = state.players[player.0].get_slot(source) {
        if slot.used_ability_this_turn {
            return Err(crate::error::EngineError::InvalidAction(
                "Ability already used this turn".into(),
            ));
        }
    }

    // Search deck for basic Lightning Pokemon
    let deck_ids: Vec<_> = state.players[player.0].deck.iter().copied().collect();
    let mut found = Vec::new();
    let mut remaining = Vec::new();

    for card_id in deck_ids.iter().rev() {
        if found.len() < 2 {
            if let Some(card_def) = state.get_card_def(*card_id) {
                if card_def.is_pokemon()
                    && card_def.stage == Some(crate::card::Stage::Basic)
                    && card_def.energy_type == Some(EnergyType::Lightning)
                {
                    found.push(*card_id);
                    continue;
                }
            }
        }
        remaining.push(*card_id);
    }

    state.players[player.0].deck = remaining;

    // Add to bench
    let mut added = 0;
    for card_id in &found {
        if state.players[player.0].bench_count() < crate::MAX_BENCH_SIZE {
            if let Some(empty_slot) = state.players[player.0]
                .bench
                .iter_mut()
                .find(|s| s.is_none())
            {
                let mut slot = PokemonSlot::new();
                slot.cards.push(*card_id);
                slot.turn_put_in_play = state.turn.turn_number;
                *empty_slot = Some(slot);
                added += 1;
            }
        }
    }

    // Mark ability as used
    if let Some(slot) = state.players[player.0].get_slot_mut(source) {
        slot.used_ability_this_turn = true;
    }

    Ok(EffectResult::new())
}

/// Charizard ex - Infernal Reign ability
/// When evolving, search up to 3 Fire energy from deck and attach
pub fn ability_infernal_reign(
    state: &mut GameState,
    player: PlayerId,
    source: SlotRef,
) -> Result<EffectResult> {
    // Search deck for Fire energy
    let deck_ids: Vec<_> = state.players[player.0].deck.iter().copied().collect();
    let mut found = Vec::new();
    let mut remaining = Vec::new();

    for card_id in deck_ids.iter().rev() {
        if found.len() < 3 {
            if let Some(card_def) = state.get_card_def(*card_id) {
                if card_def.is_basic_energy() && card_def.energy_type == Some(EnergyType::Fire) {
                    found.push(*card_id);
                    continue;
                }
            }
        }
        remaining.push(*card_id);
    }

    state.players[player.0].deck = remaining;

    // Attach to Pokemon in play (can distribute among player's Pokemon)
    for card_id in &found {
        if let Some(slot) = state.players[player.0].get_slot_mut(source) {
            slot.energies.push(*card_id);
        }
    }

    Ok(EffectResult::new())
}

/// Pidgeot ex - Wind Search ability
/// Once per turn, search any 1 card from deck to hand
pub fn ability_wind_search(
    state: &mut GameState,
    player: PlayerId,
    source: SlotRef,
) -> Result<EffectResult> {
    let player_state = &mut state.players[player.0];

    // Check if already used
    if let Some(slot) = player_state.get_slot(source) {
        if slot.used_ability_this_turn {
            return Err(crate::error::EngineError::InvalidAction(
                "Ability already used this turn".into(),
            ));
        }
    }

    // Search deck for any card (just take top card)
    if let Some(card_id) = player_state.deck.pop() {
        player_state.hand.push(card_id);
    }

    // Mark ability as used
    if let Some(slot) = player_state.get_slot_mut(source) {
        slot.used_ability_this_turn = true;
    }

    Ok(EffectResult::new())
}

/// Manaphy - Awaken ability
/// Prevent all damage to benched Pokemon
pub fn ability_awaken_is_active(state: &GameState, player: PlayerId) -> bool {
    let player_state = &state.players[player.0];

    // Check if Manaphy is in active
    if let Some(active) = &player_state.active {
        if let Some(card_def) = active.top_card().and_then(|id| state.get_card_def(id)) {
            if card_def.id.0 == "CS5bC_052" {
                return true;
            }
        }
    }

    false
}

/// Iron Hands ex - Double Impact attack
/// 120 damage, does 30 to 1 benched Pokemon
pub fn attack_double_impact(
    state: &mut GameState,
    attacker: PlayerId,
    attacker_slot: SlotRef,
    defender: PlayerId,
    defender_slot: SlotRef,
    choices: &crate::action::Choices,
) -> Result<EffectResult> {
    use crate::damage::DamageCalculator;

    let calculator = DamageCalculator::new();
    let mut events = Vec::new();

    // Calculate main damage
    let main_damage =
        calculator.calculate_damage(state, attacker, attacker_slot, defender, defender_slot, 120);

    // Apply damage to active
    let ko = super::apply_damage(state, defender, defender_slot, main_damage);
    events.push(crate::engine::Event::Damage {
        target_player: defender,
        target_slot: defender_slot,
        damage: main_damage,
        ko,
    });

    // Apply 30 damage to benched Pokemon if selected
    if let Some(target_slot) = choices.selected_slots.first() {
        let bench_damage = calculator.calculate_bench_damage(
            state,
            attacker,
            attacker_slot,
            defender,
            *target_slot,
            30,
        );
        let bench_ko = super::apply_damage(state, defender, *target_slot, bench_damage);
        events.push(crate::engine::Event::Damage {
            target_player: defender,
            target_slot: *target_slot,
            damage: bench_damage,
            ko: bench_ko,
        });
    }

    Ok(
        EffectResult::new().with_event(crate::engine::Event::Attack {
            attacker,
            defender,
            attack_index: 0,
            damage: main_damage,
        }),
    )
}

/// Charizard ex - Scorching Darkness attack
/// 180 + 30 per opponent's remaining prize cards
pub fn attack_scorching_darkness(
    state: &mut GameState,
    attacker: PlayerId,
    attacker_slot: SlotRef,
    defender: PlayerId,
    defender_slot: SlotRef,
) -> Result<EffectResult> {
    use crate::damage::DamageCalculator;

    let calculator = DamageCalculator::new();
    let opponent = state.players[defender.0].prizes.len();
    let bonus = (6 - opponent) * 30; // 6 prizes total, bonus per taken prize

    let total_damage = calculator.calculate_damage(
        state,
        attacker,
        attacker_slot,
        defender,
        defender_slot,
        180 + bonus as u16,
    );

    let ko = super::apply_damage(state, defender, defender_slot, total_damage);

    Ok(
        EffectResult::new().with_event(crate::engine::Event::Damage {
            target_player: defender,
            target_slot: defender_slot,
            damage: total_damage,
            ko,
        }),
    )
}
