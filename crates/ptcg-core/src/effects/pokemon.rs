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

/// Iron Hands ex - Double Impact: 120 main damage + 30 bench snipe.
pub fn attack_double_impact(
    state: &mut GameState,
    attacker: PlayerId,
    defender: PlayerId,
    base_damage: u16,
    choices: &crate::action::Choices,
) -> Result<super::AttackResult> {
    let main_damage = base_damage; // 120 passed in
    let mut bench_damage = Vec::new();
    let mut events = Vec::new();

    let ko = super::apply_damage(state, defender, SlotRef::Active, main_damage);
    events.push(crate::engine::Event::Damage {
        target_player: defender, target_slot: SlotRef::Active,
        damage: main_damage, ko,
    });

    if let Some(target_slot) = choices.selected_slots.first() {
        bench_damage.push((*target_slot, 30u16));
        let bench_ko = super::apply_damage(state, defender, *target_slot, 30);
        events.push(crate::engine::Event::Damage {
            target_player: defender, target_slot: *target_slot,
            damage: 30, ko: bench_ko,
        });
    }

    Ok(super::AttackResult { damage: main_damage, ko, bench_damage, events, self_lock: false })
}

/// Charizard ex - Scorching Darkness: 180 + 30 per opponent's remaining prize cards.
pub fn attack_scorching_darkness(
    state: &mut GameState,
    attacker: PlayerId,
    defender: PlayerId,
    base_damage: u16,
) -> Result<super::AttackResult> {
    let opponent_prizes = state.players[defender.0].prizes.len() as u16;
    let damage = base_damage + opponent_prizes * 30;

    let ko = super::apply_damage(state, defender, SlotRef::Active, damage);
    let events = vec![crate::engine::Event::Damage {
        target_player: defender, target_slot: SlotRef::Active, damage, ko,
    }];

    Ok(super::AttackResult { damage, ko, bench_damage: vec![], events, self_lock: false })
}

/// Miraidon ex - Photon Blaster: 220 damage, self-lock this attack next turn.
pub fn attack_photon_blaster(
    state: &mut GameState,
    defender: PlayerId,
    base_damage: u16,
) -> Result<super::AttackResult> {
    let damage = base_damage; // 220
    let ko = super::apply_damage(state, defender, SlotRef::Active, damage);
    let events = vec![crate::engine::Event::Damage {
        target_player: defender, target_slot: SlotRef::Active, damage, ko,
    }];

    Ok(super::AttackResult { damage, ko, bench_damage: vec![], events, self_lock: true })
}

/// Radiant Greninja - Moonlight Shuriken: 90 to active + 90 to up to 2 opponent Pokemon.
pub fn attack_moonlight_shuriken(
    state: &mut GameState,
    attacker: PlayerId,
    defender: PlayerId,
    base_damage: u16,
    choices: &crate::action::Choices,
) -> Result<super::AttackResult> {
    use crate::damage::DamageCalculator;
    let calc = DamageCalculator::new();
    let main_damage = base_damage; // 90
    let mut bench_damage = Vec::new();
    let mut events = Vec::new();

    // Hit active
    let ko = super::apply_damage(state, defender, SlotRef::Active, main_damage);
    events.push(crate::engine::Event::Damage {
        target_player: defender, target_slot: SlotRef::Active, damage: main_damage, ko,
    });

    // Hit up to 2 bench targets
    for (i, &target) in choices.selected_slots.iter().take(2).enumerate() {
        let bench_dmg = calc.calculate_bench_damage(
            state, attacker, SlotRef::Active, defender, target, 90,
        );
        bench_damage.push((target, bench_dmg));
        let bench_ko = super::apply_damage(state, defender, target, bench_dmg);
        events.push(crate::engine::Event::Damage {
            target_player: defender, target_slot: target, damage: bench_dmg, ko: bench_ko,
        });
    }

    Ok(super::AttackResult { damage: main_damage, ko, bench_damage, events, self_lock: false })
}

/// Pidgeot ex - Gale Winds: 120 damage + optional discard stadium for 120 bench snipe.
pub fn attack_gale_winds(
    state: &mut GameState,
    attacker: PlayerId,
    defender: PlayerId,
    base_damage: u16,
    choices: &crate::action::Choices,
) -> Result<super::AttackResult> {
    let main_damage = base_damage; // 120
    let mut bench_damage = Vec::new();
    let mut events = Vec::new();

    // Main damage to active
    let ko = super::apply_damage(state, defender, SlotRef::Active, main_damage);
    events.push(crate::engine::Event::Damage {
        target_player: defender, target_slot: SlotRef::Active, damage: main_damage, ko,
    });

    // Optional: discard a Stadium from hand to deal 120 to bench
    if choices.mode == Some(1) && !choices.selected_cards.is_empty() {
        let stadium_card = choices.selected_cards[0];
        let player_state = &mut state.players[attacker.0];
        if let Some(pos) = player_state.hand.iter().position(|&id| id == stadium_card) {
            player_state.hand.remove(pos);
            player_state.discard.push(stadium_card);
        }

        if let Some(&target) = choices.selected_slots.first() {
            bench_damage.push((target, 120u16));
            let bench_ko = super::apply_damage(state, defender, target, 120);
            events.push(crate::engine::Event::Damage {
                target_player: defender, target_slot: target, damage: 120, ko: bench_ko,
            });
        }
    }

    Ok(super::AttackResult { damage: main_damage, ko, bench_damage, events, self_lock: false })
}

/// Radiant Charizard - Combustion Blast: 250 damage, only usable when prizes <= 1.
pub fn attack_combustion_blast(
    state: &mut GameState,
    attacker: PlayerId,
    defender: PlayerId,
    base_damage: u16,
) -> Result<super::AttackResult> {
    let prizes_left = state.players[attacker.0].prizes.len();
    if prizes_left > 1 {
        // Attack condition not met: no effect (should be filtered by legal_actions)
        return Ok(super::AttackResult::new(0));
    }

    let damage = base_damage; // 250
    let ko = super::apply_damage(state, defender, SlotRef::Active, damage);
    let events = vec![crate::engine::Event::Damage {
        target_player: defender, target_slot: SlotRef::Active, damage, ko,
    }];

    Ok(super::AttackResult { damage, ko, bench_damage: vec![], events, self_lock: false })
}

/// Forest Seal Stone — Star Alchemy: search deck for any 1 card.
pub fn ability_star_alchemy(
    state: &mut crate::state::GameState,
    player: crate::state::PlayerId,
    _source: crate::state::SlotRef,
) -> Result<EffectResult> {
    use crate::effects::common::search_deck;

    let player_state = &state.players[player.0];
    if player_state.deck.is_empty() {
        return Ok(EffectResult::new());
    }

    // Search deck for any 1 card (simplified: take the first card)
    let card = player_state.deck.last().copied();
    if let Some(card_id) = card {
        let player_state = &mut state.players[player.0];
        player_state.deck.pop();
        player_state.hand.push(card_id);
    }

    Ok(EffectResult::new())
}

/// Radiant Greninja — Concealed Cards: discard 1 energy, draw 2 cards.
pub fn ability_concealed_cards(
    state: &mut crate::state::GameState,
    player: crate::state::PlayerId,
    source: crate::state::SlotRef,
) -> Result<EffectResult> {
    let slot = state.players[player.0].get_slot(source);
    if slot.map(|s| s.energies.is_empty()).unwrap_or(true) {
        return Ok(EffectResult::new());
    }
    if let Some(slot) = state.players[player.0].get_slot_mut(source) {
        if let Some(energy) = slot.energies.pop() {
            state.players[player.0].discard.push(energy);
            state.draw_cards(player, 2);
        }
    }
    Ok(EffectResult::new())
}

/// Mew ex — Restart: put hand on bottom of deck, draw same number.
pub fn ability_restart(
    state: &mut crate::state::GameState,
    player: crate::state::PlayerId,
    _source: crate::state::SlotRef,
) -> Result<EffectResult> {
    let player_state = &mut state.players[player.0];
    let hand_size = player_state.hand.len();
    if hand_size == 0 {
        return Ok(EffectResult::new());
    }
    let hand_cards: Vec<_> = player_state.hand.drain(..).collect();
    for &card in hand_cards.iter().rev() {
        player_state.deck.insert(0, card);
    }
    for _ in 0..hand_size {
        if let Some(card) = player_state.deck.pop() {
            player_state.hand.push(card);
        }
    }
    Ok(EffectResult::new())
}

/// Entei V - Burst Roar: attach 1 Fire Energy from discard to this Pokemon.
pub fn ability_burst_roar(
    state: &mut crate::state::GameState,
    player: crate::state::PlayerId,
    source: crate::state::SlotRef,
) -> Result<EffectResult> {
    let player_state = &state.players[player.0];
    let fire_energy = player_state.discard.iter().position(|&id| {
        state.get_card_def(id).map(|d| d.energy_type == Some(EnergyType::Fire)).unwrap_or(false)
    });
    if let Some(pos) = fire_energy {
        let energy_id = player_state.discard[pos];
        let player_state = &mut state.players[player.0];
        player_state.discard.remove(pos);
        if let Some(slot) = player_state.get_slot_mut(source) {
            slot.energies.push(energy_id);
        }
    }
    Ok(EffectResult::new())
}

/// Gouging Fire ex - Magma Blast: deal damage, then discard 1 energy from self.
pub fn attack_discard_energy_from_self(
    state: &mut GameState,
    attacker: PlayerId,
    defender: PlayerId,
    base_damage: u16,
) -> Result<super::AttackResult> {
    let ko = super::apply_damage(state, defender, SlotRef::Active, base_damage);
    let events = vec![crate::engine::Event::Damage {
        target_player: defender, target_slot: SlotRef::Active, damage: base_damage, ko,
    }];
    if let Some(slot) = state.players[attacker.0].get_slot_mut(SlotRef::Active) {
        if let Some(energy) = slot.energies.pop() {
            state.players[attacker.0].discard.push(energy);
        }
    }
    Ok(super::AttackResult { damage: base_damage, ko, bench_damage: vec![], events, self_lock: false })
}

/// Roaring Moon ex - Calamity Storm: +60 if stadium in play.
pub fn attack_discard_stadium_bonus(
    state: &mut GameState,
    attacker: PlayerId,
    defender: PlayerId,
    base_damage: u16,
    _choices: &crate::action::Choices,
) -> Result<super::AttackResult> {
    let has_stadium = state.players[0].stadium.is_some() || state.players[1].stadium.is_some();
    let damage = if has_stadium { base_damage + 60 } else { base_damage };
    let ko = super::apply_damage(state, defender, SlotRef::Active, damage);
    let events = vec![crate::engine::Event::Damage {
        target_player: defender, target_slot: SlotRef::Active, damage, ko,
    }];
    Ok(super::AttackResult { damage, ko, bench_damage: vec![], events, self_lock: false })
}

// ── Tier 1: High-frequency draw/search abilities ──

/// Squawkabilly ex — Squawk and Seize: discard hand, draw 6. Turn 1 only.
pub fn ability_squawk_and_seize(
    state: &mut GameState, player: PlayerId, _source: SlotRef,
) -> Result<EffectResult> {
    let ps = &mut state.players[player.0];
    // Discard entire hand
    let hand: Vec<_> = ps.hand.drain(..).collect();
    ps.discard.extend(hand);
    // Draw 6
    state.draw_cards(player, 6);
    Ok(EffectResult::new())
}

/// Lumineon V — Luminous Sign: when played to bench, search deck for a Supporter.
pub fn ability_luminous_sign(
    state: &mut GameState, player: PlayerId, _source: SlotRef,
) -> Result<EffectResult> {
    // Find a Supporter card in deck (read-only pass)
    let supporter_pos = {
        let ps = &state.players[player.0];
        ps.deck.iter().rev().position(|&id| {
            state.get_card_def(id).map(|d| d.card_type == crate::card::CardType::Supporter).unwrap_or(false)
        })
    };
    if let Some(pos) = supporter_pos {
        let ps = &mut state.players[player.0];
        let idx = ps.deck.len() - 1 - pos;
        let card_id = ps.deck.remove(idx);
        ps.hand.push(card_id);
    }
    Ok(EffectResult::new())
}

/// Rotom V — Instant Charge: draw 3 cards, then end turn.
pub fn ability_instant_charge(
    state: &mut GameState, player: PlayerId, _source: SlotRef,
) -> Result<EffectResult> {
    state.draw_cards(player, 3);
    Ok(EffectResult::new())
}

/// Manaphy bench check: is Awaken active? Returns true if Manaphy protects bench.
pub fn is_manaphy_awaken_active(state: &GameState, player: PlayerId) -> bool {
    state.players[player.0].bench.iter().any(|s| {
        s.as_ref().and_then(|s| s.top_card()).and_then(|id| state.get_card_def(id))
            .map(|d| d.abilities.iter().any(|a| a.effect_id == "ability_awaken"))
            .unwrap_or(false)
    })
}

/// Dusknoir line — Curse Blast: self-KO, place N damage counters on opponent.
pub fn ability_dusknoir_curse_bomb(
    state: &mut GameState, player: PlayerId, source: SlotRef,
) -> Result<EffectResult> {
    // Read card name first (immutable borrow)
    let is_dusknoir = state.players[player.0].get_slot(source)
        .and_then(|s| s.top_card())
        .and_then(|id| state.get_card_def(id))
        .map(|d| d.name.contains("Dusknoir") || d.name.contains("黑夜魔灵"))
        .unwrap_or(false);
    let num_counters = if is_dusknoir { 13u16 } else { 5u16 };

    if num_counters == 0 { return Ok(EffectResult::new()); }

    // Self-KO (mutable borrow)
    if let Some(slot) = state.players[player.0].get_slot_mut(source) {
        slot.damage = 999; // KO self
    }
    // Place damage counters on opponent's active
    let opponent = player.opponent();
    if let Some(active) = state.players[opponent.0].active.as_mut() {
        active.damage += (num_counters * 10) as u16;
    }
    Ok(EffectResult::new())
}

/// Gardevoir ex — Psychic Embrace: attach Psychic Energy from discard to own Pokemon.
pub fn ability_psychic_embrace(
    state: &mut GameState, player: PlayerId, source: SlotRef,
) -> Result<EffectResult> {
    let psychic_pos = state.players[player.0].discard.iter().position(|&id| {
        state.get_card_def(id).map(|d| d.energy_type == Some(EnergyType::Psychic)).unwrap_or(false)
    });
    if let Some(pos) = psychic_pos {
        let card = state.players[player.0].discard.remove(pos);
        if let Some(slot) = state.players[player.0].get_slot_mut(source) {
            slot.energies.push(card);
        }
    }
    Ok(EffectResult::new())
}

/// Dragapult ex — Phantom Dive: 200 + place 6 damage counters on opponent bench.
pub fn attack_phantom_dive(
    state: &mut GameState, attacker: PlayerId, defender: PlayerId,
    base_damage: u16, _choices: &crate::action::Choices,
) -> Result<super::AttackResult> {
    let ko = super::apply_damage(state, defender, SlotRef::Active, base_damage);
    let mut bench_damage = Vec::new();
    let mut events = vec![crate::engine::Event::Damage {
        target_player: defender, target_slot: SlotRef::Active, damage: base_damage, ko,
    }];
    // Place 6 damage counters (60 damage) on opponent's bench Pokemon
    let counters = 60u16;
    for i in 0..5 {
        if let Some(slot) = state.players[defender.0].bench[i].as_mut() {
            if !slot.is_empty() {
                slot.damage += counters;
                bench_damage.push((SlotRef::Bench(i), counters));
                events.push(crate::engine::Event::Damage {
                    target_player: defender, target_slot: SlotRef::Bench(i), damage: counters, ko: false,
                });
            }
        }
    }
    Ok(super::AttackResult { damage: base_damage, ko, bench_damage, events, self_lock: false })
}

/// Iron Bundle — Blower: if on bench, switch opponent active with bench.
pub fn ability_iron_bundle_blower(
    state: &mut GameState, player: PlayerId, _source: SlotRef,
) -> Result<EffectResult> {
    let opp = player.opponent();
    // Find a bench Pokemon to swap with
    if let Some(bench_idx) = state.players[opp.0].bench.iter().position(|s| s.as_ref().map(|s| !s.is_empty()).unwrap_or(false)) {
        if state.players[opp.0].active.is_some() && state.players[opp.0].bench[bench_idx].is_some() {
            let active = state.players[opp.0].active.take();
            let bench = state.players[opp.0].bench[bench_idx].take();
            state.players[opp.0].active = bench;
            state.players[opp.0].bench[bench_idx] = active;
        }
    }
    Ok(EffectResult::new())
}

/// Fezandipiti ex — Flip the Script: if own Pokemon was KO'd last opponent's turn, draw 3.
pub fn ability_flip_the_script(
    state: &mut GameState, player: PlayerId, _source: SlotRef,
) -> Result<EffectResult> {
    state.draw_cards(player, 3);
    Ok(EffectResult::new())
}

// ── Remaining ability effects ──

/// Munkidori — Excited Brain: move up to 3 damage counters from own Pokemon to opponent active.
pub fn ability_adrena_brain(
    state: &mut GameState, player: PlayerId, _source: SlotRef,
) -> Result<EffectResult> {
    let opponent = player.opponent();
    // Move up to 3 damage counters (30 HP) from own active to opponent active
    if let Some(active) = state.players[player.0].active.as_mut() {
        let transfer = active.damage.min(30);
        if transfer > 0 {
            active.damage -= transfer;
            if let Some(opp) = state.players[opponent.0].active.as_mut() {
                opp.damage += transfer;
            }
        }
    }
    Ok(EffectResult::new())
}

/// Bloodmoon Ursaluna ex — Veteran's Technique: attack cost reduced by opponent's prizes taken.
/// Simplified: already handled via card definition (mechanic-based).
pub fn ability_veterans_technique(
    _state: &mut GameState, _player: PlayerId, _source: SlotRef,
) -> Result<EffectResult> {
    Ok(EffectResult::new()) // passive — handled in energy cost check
}

/// Drakloak — Recon Directive: look at top 2 cards of deck, take 1, put the other back.
pub fn ability_recon_directive(
    state: &mut GameState, player: PlayerId, _source: SlotRef,
) -> Result<EffectResult> {
    let ps = &mut state.players[player.0];
    if ps.deck.len() >= 2 {
        let c2 = ps.deck.pop().unwrap();
        let c1 = ps.deck.pop().unwrap();
        ps.hand.push(c1); // take the top one
        ps.deck.push(c2); // put second back
    } else if let Some(c) = ps.deck.pop() {
        ps.hand.push(c);
    }
    Ok(EffectResult::new())
}

/// Entei V / Raikou V — Fleet Foot: draw 1 card if this Pokemon is active.
pub fn ability_fleet_foot(
    state: &mut GameState, player: PlayerId, source: SlotRef,
) -> Result<EffectResult> {
    if matches!(source, SlotRef::Active) {
        state.draw_cards(player, 1);
    }
    Ok(EffectResult::new())
}

/// Radiant Alakazam — Painful Spoons: move up to 2 damage counters between opponent Pokemon.
pub fn ability_painful_spoons(
    state: &mut GameState, player: PlayerId, _source: SlotRef,
) -> Result<EffectResult> {
    let opp = player.opponent();
    // Move 2 damage counters (20 HP) from one opponent bench to opponent active
    if let Some(bench_idx) = state.players[opp.0].bench.iter().position(|s| {
        s.as_ref().map(|s| s.damage >= 20).unwrap_or(false)
    }) {
        let transfer = 20u16;
        if let Some(slot) = state.players[opp.0].bench[bench_idx].as_mut() {
            slot.damage -= transfer;
        }
        if let Some(active) = state.players[opp.0].active.as_mut() {
            active.damage += transfer;
        }
    }
    Ok(EffectResult::new())
}

/// Arceus V — Charging Star: attach up to 3 basic Energy from deck to benched Pokemon.
pub fn ability_charging_star(
    state: &mut GameState, player: PlayerId, _source: SlotRef,
) -> Result<EffectResult> {
    let ps = &state.players[player.0];
    let bench_target = ps.bench.iter().position(|s| s.as_ref().map(|s| !s.is_empty()).unwrap_or(false));
    if let Some(bi) = bench_target {
        let attached = 0;
        // Find up to 3 basic energy in deck
        let mut energy_ids = Vec::new();
        for &eid in ps.deck.iter().rev() {
            if energy_ids.len() >= 3 { break; }
            if state.get_card_def(eid).map(|d| d.is_basic_energy()).unwrap_or(false) {
                energy_ids.push(eid);
            }
        }
        let ps = &mut state.players[player.0];
        for eid in &energy_ids {
            if let Some(pos) = ps.deck.iter().position(|&id| id == *eid) {
                ps.deck.remove(pos);
                if let Some(slot) = ps.bench[bi].as_mut() {
                    slot.energies.push(*eid);
                }
            }
        }
    }
    Ok(EffectResult::new())
}

// ── Remaining attack effects ──

/// Charizard ex — Burning Darkness: 180 + 30 per opponent's remaining prize cards.
pub fn attack_burning_darkness(
    state: &mut GameState, _attacker: PlayerId, defender: PlayerId, base_damage: u16,
) -> Result<super::AttackResult> {
    let bonus = state.players[defender.0].prizes.len() as u16 * 30;
    let damage = base_damage + bonus;
    let ko = super::apply_damage(state, defender, SlotRef::Active, damage);
    Ok(super::AttackResult { damage, ko, ..Default::default() })
}

/// Arceus VSTAR — Trinity Nova: 200 + search 2 basic energy from deck, attach to bench.
pub fn attack_trinity_nova(
    state: &mut GameState, attacker: PlayerId, defender: PlayerId, base_damage: u16, _choices: &crate::action::Choices,
) -> Result<super::AttackResult> {
    let ko = super::apply_damage(state, defender, SlotRef::Active, base_damage);
    // Search 2 basic energy, attach to bench
    let ps = &state.players[attacker.0];
    let mut found = Vec::new();
    for &eid in ps.deck.iter().rev() {
        if found.len() >= 2 { break; }
        if state.get_card_def(eid).map(|d| d.is_basic_energy()).unwrap_or(false) {
            found.push(eid);
        }
    }
    let bench_target = ps.bench.iter().position(|s| s.as_ref().map(|s| !s.is_empty()).unwrap_or(false));
    let ps = &mut state.players[attacker.0];
    for eid in &found {
        if let Some(pos) = ps.deck.iter().position(|&id| id == *eid) {
            ps.deck.remove(pos);
            if let Some(slot) = bench_target.and_then(|bi| ps.bench[bi].as_mut()) {
                slot.energies.push(*eid);
            } else if let Some(active) = ps.active.as_mut() {
                active.energies.push(*eid);
            }
        }
    }
    Ok(super::AttackResult { damage: base_damage, ko, ..Default::default() })
}

/// Lugia VSTAR — Storm Dive: 220 + search up to 2 Archeops from discard, put on bench.
pub fn attack_storm_dive(
    state: &mut GameState, attacker: PlayerId, defender: PlayerId, base_damage: u16, _choices: &crate::action::Choices,
) -> Result<super::AttackResult> {
    let ko = super::apply_damage(state, defender, SlotRef::Active, base_damage);
    // Find Archeops in discard
    let archeops: Vec<_> = {
        let ps = &state.players[attacker.0];
        ps.discard.iter().enumerate().filter(|(_, &eid)| {
            state.get_card_def(eid).map(|d| d.name.contains("Archeops") || d.name.contains("始祖大鸟")).unwrap_or(false)
        }).map(|(i, &eid)| (i, eid)).take(2).collect()
    };
    for (discard_pos, eid) in archeops.iter().rev() {
        let eid = *eid;
        let ps = &mut state.players[attacker.0];
        if ps.bench_count() < crate::MAX_BENCH_SIZE {
            ps.discard.remove(*discard_pos);
            if let Some(slot) = ps.bench.iter_mut().find(|s| s.is_none()) {
                let mut pokemon = crate::state::PokemonSlot::new();
                pokemon.cards.push(eid);
                pokemon.turn_put_in_play = state.turn.turn_number;
                *slot = Some(pokemon);
            }
        }
    }
    Ok(super::AttackResult { damage: base_damage, ko, ..Default::default() })
}

/// Origin Palkia VSTAR — Subspace Swell: 60 + 20 per benched Pokemon (both players).
pub fn attack_subspace_swell(
    state: &mut GameState, _attacker: PlayerId, defender: PlayerId, base_damage: u16,
) -> Result<super::AttackResult> {
    let bench_total = state.players[0].bench_count() + state.players[1].bench_count();
    let damage = base_damage + (bench_total as u16 * 20);
    let ko = super::apply_damage(state, defender, SlotRef::Active, damage);
    Ok(super::AttackResult { damage, ko, ..Default::default() })
}

/// Giratina VSTAR — Lost Impact: 280, discard 2 energy from this Pokemon.
pub fn attack_lost_impact(
    state: &mut GameState, attacker: PlayerId, defender: PlayerId, base_damage: u16,
) -> Result<super::AttackResult> {
    let ko = super::apply_damage(state, defender, SlotRef::Active, base_damage);
    // Discard 2 energy (collect first, then push to avoid borrow conflict)
    let mut lost = Vec::new();
    if let Some(slot) = state.players[attacker.0].get_slot_mut(SlotRef::Active) {
        for _ in 0..2 { if let Some(e) = slot.energies.pop() { lost.push(e); } }
    }
    state.players[attacker.0].discard.extend(lost);
    Ok(super::AttackResult { damage: base_damage, ko, ..Default::default() })
}

/// Gholdengo ex — Coin Scramble: 50 per energy discarded from this Pokemon (up to 3).
pub fn attack_coin_scramble(
    state: &mut GameState, attacker: PlayerId, defender: PlayerId, base_damage: u16, _choices: &crate::action::Choices,
) -> Result<super::AttackResult> {
    let discard_count = state.players[attacker.0].get_slot(SlotRef::Active).map(|s| s.energies.len().min(3)).unwrap_or(0);
    let mut tossed = Vec::new();
    if let Some(slot) = state.players[attacker.0].get_slot_mut(SlotRef::Active) {
        for _ in 0..discard_count { if let Some(e) = slot.energies.pop() { tossed.push(e); } }
    }
    state.players[attacker.0].discard.extend(tossed);
    let damage = base_damage * discard_count as u16;
    let ko = if damage > 0 { super::apply_damage(state, defender, SlotRef::Active, damage) } else { false };
    Ok(super::AttackResult { damage, ko, ..Default::default() })
}

/// Banette ex — Eternal Darkness: 30 + opponent can't play Item cards next turn.
pub fn attack_eternal_darkness(
    state: &mut GameState, _attacker: PlayerId, defender: PlayerId, base_damage: u16,
) -> Result<super::AttackResult> {
    let ko = super::apply_damage(state, defender, SlotRef::Active, base_damage);
    // Item lock effect: simplified — set a flag on opponent state
    state.players[defender.0].turn_actions_used.insert("item_lock_next_turn".into());
    Ok(super::AttackResult { damage: base_damage, ko, ..Default::default() })
}

/// Gardevoir ex — Miracle Force: 160, heal 30 from this Pokemon.
pub fn attack_miracle_force(
    state: &mut GameState, attacker: PlayerId, defender: PlayerId, base_damage: u16,
) -> Result<super::AttackResult> {
    let ko = super::apply_damage(state, defender, SlotRef::Active, base_damage);
    if let Some(slot) = state.players[attacker.0].get_slot_mut(SlotRef::Active) {
        slot.damage = slot.damage.saturating_sub(30);
    }
    Ok(super::AttackResult { damage: base_damage, ko, ..Default::default() })
}

/// Blissey ex — Return: 180, heal 30 from 1 bench Pokemon.
pub fn attack_blissey_return(
    state: &mut GameState, attacker: PlayerId, defender: PlayerId, base_damage: u16, _choices: &crate::action::Choices,
) -> Result<super::AttackResult> {
    let ko = super::apply_damage(state, defender, SlotRef::Active, base_damage);
    // Heal 30 from first bench Pokemon
    if let Some(slot) = state.players[attacker.0].bench.iter_mut().find(|s| s.as_ref().map(|s| s.damage > 0).unwrap_or(false)) {
        if let Some(s) = slot.as_mut() {
            s.damage = s.damage.saturating_sub(30);
        }
    }
    Ok(super::AttackResult { damage: base_damage, ko, ..Default::default() })
}

/// Iron Thorns ex — Volt Cyclone: 140, discard 1 Future Booster Energy from self for bonus.
pub fn attack_volt_cyclone(
    state: &mut GameState, attacker: PlayerId, defender: PlayerId, base_damage: u16,
) -> Result<super::AttackResult> {
    let ko = super::apply_damage(state, defender, SlotRef::Active, base_damage);
    // Discard a tool if attached
    if let Some(slot) = state.players[attacker.0].get_slot_mut(SlotRef::Active) {
        if let Some(tool) = slot.tool.take() {
            state.players[attacker.0].discard.push(tool);
        }
    }
    Ok(super::AttackResult { damage: base_damage, ko, ..Default::default() })
}

/// Hisuian Samurott VSTAR — Cruel Blade: 110+, +110 if defender is damaged.
pub fn attack_cruel_blade(
    state: &mut GameState, _attacker: PlayerId, defender: PlayerId, base_damage: u16,
) -> Result<super::AttackResult> {
    let defender_damaged = state.players[defender.0].active.as_ref().map(|s| s.damage > 0).unwrap_or(false);
    let damage = if defender_damaged { base_damage + 110 } else { base_damage };
    let ko = super::apply_damage(state, defender, SlotRef::Active, damage);
    Ok(super::AttackResult { damage, ko, ..Default::default() })
}

/// Regidrago VSTAR — Dragon's Glory: copy any Dragon Pokemon's attack from discard.
/// Simplified: use the first Dragon Pokemon's attack in discard.
pub fn attack_dragons_glory(
    state: &mut GameState, attacker: PlayerId, defender: PlayerId, _base_damage: u16, _choices: &crate::action::Choices,
) -> Result<super::AttackResult> {
    let discard_dragon = {
        let ps = &state.players[attacker.0];
        ps.discard.iter().find_map(|&eid| {
            state.get_card_def(eid).filter(|d| {
                d.energy_type == Some(crate::card::EnergyType::Dragon) && !d.attacks.is_empty()
            }).map(|d| d.attacks[0].damage)
        })
    };
    let damage = discard_dragon.unwrap_or(100); // fallback to 100
    let ko = super::apply_damage(state, defender, SlotRef::Active, damage);
    Ok(super::AttackResult { damage, ko, ..Default::default() })
}
