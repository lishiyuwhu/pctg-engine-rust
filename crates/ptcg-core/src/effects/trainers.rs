//! Trainer card effects

use super::EffectResult;
use crate::card::{CardType, Stage};
use crate::error::Result;
use crate::state::{CardInstanceId, GameState, PlayerId, SlotRef};

/// Nest Ball - Search basic Pokemon to bench
pub fn effect_nest_ball(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    // Check bench space
    if state.players[player.0].bench_count() >= crate::MAX_BENCH_SIZE {
        return Err(crate::error::EngineError::InvalidAction(
            "Bench is full".into(),
        ));
    }

    // Search deck for basic Pokemon
    let deck_ids: Vec<_> = state.players[player.0].deck.iter().copied().collect();
    let mut found = None;
    let mut remaining = Vec::new();

    for card_id in deck_ids.iter().rev() {
        if found.is_none() {
            if let Some(card_def) = state.get_card_def(*card_id) {
                if card_def.is_pokemon() && card_def.stage == Some(Stage::Basic) {
                    found = Some(*card_id);
                    continue;
                }
            }
        }
        remaining.push(*card_id);
    }

    state.players[player.0].deck = remaining;

    if let Some(card_id) = found {
        // Add to bench
        if let Some(empty_slot) = state.players[player.0]
            .bench
            .iter_mut()
            .find(|s| s.is_none())
        {
            let mut slot = crate::state::PokemonSlot::new();
            slot.cards.push(card_id);
            slot.turn_put_in_play = state.turn.turn_number;
            *empty_slot = Some(slot);

            return Ok(
                EffectResult::new().with_event(crate::engine::Event::CardPlayed {
                    player,
                    card: card_id,
                    location: "bench".into(),
                }),
            );
        }
    }

    Ok(EffectResult::new())
}

/// Ultra Ball - Discard 2, search any Pokemon
pub fn effect_ultra_ball(
    state: &mut GameState,
    player: PlayerId,
    discard: Vec<CardInstanceId>,
) -> Result<EffectResult> {
    // Must discard 2 cards
    if discard.len() != 2 {
        return Err(crate::error::EngineError::InvalidAction(
            "Must discard exactly 2 cards".into(),
        ));
    }

    // Move discarded cards to discard pile
    let hand = &mut state.players[player.0].hand;
    let discard_pile = &mut state.players[player.0].discard;
    for card_id in &discard {
        if let Some(pos) = hand.iter().position(|&id| id == *card_id) {
            hand.remove(pos);
            discard_pile.push(*card_id);
        }
    }

    // Search deck for Pokemon
    let deck_ids: Vec<_> = state.players[player.0].deck.iter().copied().collect();
    let mut found = None;
    let mut remaining = Vec::new();

    for card_id in deck_ids.iter().rev() {
        if found.is_none() {
            if let Some(card_def) = state.get_card_def(*card_id) {
                if card_def.is_pokemon() {
                    found = Some(*card_id);
                    continue;
                }
            }
        }
        remaining.push(*card_id);
    }

    state.players[player.0].deck = remaining;

    if let Some(card_id) = found {
        state.players[player.0].hand.push(card_id);
        return Ok(
            EffectResult::new().with_event(crate::engine::Event::CardPlayed {
                player,
                card: card_id,
                location: "hand".into(),
            }),
        );
    }

    Ok(EffectResult::new())
}

/// Rare Candy - Skip Stage 1, evolve Stage 1 to Stage 2
pub fn effect_rare_candy(
    state: &mut GameState,
    player: PlayerId,
    target: SlotRef,
) -> Result<EffectResult> {
    // Get target Pokemon info first (must be Stage 1)
    let target_slot = state.players[player.0]
        .get_slot(target)
        .ok_or_else(|| crate::error::EngineError::InvalidTarget("Invalid slot".into()))?;

    if target_slot.is_empty() {
        return Err(crate::error::EngineError::InvalidTarget(
            "Slot is empty".into(),
        ));
    }

    let target_top_card = target_slot.top_card();
    let target_turn_put_in_play = target_slot.turn_put_in_play;

    // Check it's Stage 1
    if let Some(card_id) = target_top_card {
        if let Some(card_def) = state.get_card_def(card_id) {
            if card_def.stage != Some(Stage::Stage1) {
                return Err(crate::error::EngineError::CannotEvolve(
                    "Target must be a Stage 1 Pokemon".into(),
                ));
            }
        }
    }

    // Check not evolved this turn
    if target_turn_put_in_play == state.turn.turn_number {
        return Err(crate::error::EngineError::CannotEvolve(
            "Cannot evolve Pokemon put in play this turn".into(),
        ));
    }

    // Get target's card def for evolution check
    let target_def = target_top_card.and_then(|id| state.get_card_def(id));

    // Search deck for Stage 2 evolution
    let deck_ids: Vec<_> = state.players[player.0].deck.iter().copied().collect();
    let mut found = None;
    let mut remaining = Vec::new();

    for card_id in deck_ids.iter().rev() {
        if found.is_none() {
            if let Some(card_def) = state.get_card_def(*card_id) {
                if card_def.is_pokemon() && card_def.stage == Some(Stage::Stage2) {
                    // Check if it can evolve from the target
                    if let Some(target) = &target_def {
                        if card_def.can_be_evolved_from(target) {
                            found = Some(*card_id);
                            continue;
                        }
                    }
                }
            }
        }
        remaining.push(*card_id);
    }

    state.players[player.0].deck = remaining;

    if let Some(card_id) = found {
        // Add to evolution stack
        if let Some(slot) = state.players[player.0].get_slot_mut(target) {
            slot.cards.push(card_id);
            slot.turn_evolved = Some(state.turn.turn_number);

            return Ok(
                EffectResult::new().with_event(crate::engine::Event::Evolved {
                    player,
                    slot: target,
                    card: card_id,
                }),
            );
        }
    }

    Err(crate::error::EngineError::CannotEvolve(
        "No valid Stage 2 evolution found".into(),
    ))
}

/// Arven - Search 1 Item and 1 Tool
pub fn effect_arven(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    let mut found_items = Vec::new();
    let mut found_tools = Vec::new();
    let mut remaining = Vec::new();

    let deck_ids: Vec<_> = state.players[player.0].deck.iter().copied().collect();

    for card_id in deck_ids.iter().rev() {
        if found_items.len() < 1 || found_tools.len() < 1 {
            if let Some(card_def) = state.get_card_def(*card_id) {
                if found_items.len() < 1 && card_def.card_type == CardType::Item {
                    found_items.push(*card_id);
                    continue;
                }
                if found_tools.len() < 1 && card_def.card_type == CardType::Tool {
                    found_tools.push(*card_id);
                    continue;
                }
            }
        }
        remaining.push(*card_id);
    }

    state.players[player.0].deck = remaining;

    // Add to hand
    for card_id in &found_items {
        state.players[player.0].hand.push(*card_id);
    }
    for card_id in &found_tools {
        state.players[player.0].hand.push(*card_id);
    }

    Ok(EffectResult::new())
}

/// Boss's Orders - Gust opponent's bench to active
pub fn effect_boss_orders(
    state: &mut GameState,
    player: PlayerId,
    target: SlotRef,
) -> Result<EffectResult> {
    let opponent = player.opponent();
    let opponent_state = &mut state.players[opponent.0];

    // Target must be bench slot
    let bench_index = target
        .bench_index()
        .ok_or_else(|| crate::error::EngineError::InvalidTarget("Must target bench".into()))?;

    // Swap bench with active
    let bench_pokemon = opponent_state.bench[bench_index].take();
    let active_pokemon = opponent_state.active.take();

    if let Some(pokemon) = bench_pokemon {
        opponent_state.active = Some(pokemon);
    }

    if let Some(pokemon) = active_pokemon {
        opponent_state.bench[bench_index] = Some(pokemon);
    }

    Ok(
        EffectResult::new().with_event(crate::engine::Event::Switched {
            player: opponent,
            from: target,
            to: SlotRef::Active,
        }),
    )
}

/// Iono - Both players shuffle hand and draw based on prizes
pub fn effect_iono(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    for p in [player, player.opponent()] {
        let player_state = &mut state.players[p.0];
        let prizes = player_state.prizes.len();

        // Shuffle hand into deck
        let hand_size = player_state.hand.len();
        player_state.deck.extend(player_state.hand.drain(..));

        // Shuffle deck
        // (RNG is managed by engine)

        // Draw based on prizes remaining
        let draw_count = prizes;
        for _ in 0..draw_count {
            if let Some(card) = player_state.deck.pop() {
                player_state.hand.push(card);
            }
        }
    }

    Ok(EffectResult::new())
}

/// Electric Generator - Attach Lightning from top 5 to Electric Pokemon
pub fn effect_electric_generator(
    state: &mut GameState,
    player: PlayerId,
    choices: &crate::action::Choices,
) -> Result<EffectResult> {
    // Look at top 5 cards
    let top_5: Vec<_> = state.players[player.0]
        .deck
        .iter()
        .rev()
        .take(5)
        .copied()
        .collect();

    // Find Lightning energy in top 5
    let lightning_ids: Vec<_> = top_5
        .iter()
        .filter(|&&id| {
            state
                .get_card_def(id)
                .map(|def| {
                    def.is_basic_energy()
                        && def.energy_type == Some(crate::card::EnergyType::Lightning)
                })
                .unwrap_or(false)
        })
        .copied()
        .collect();

    if lightning_ids.is_empty() {
        return Ok(EffectResult::new());
    }

    // Player must select one to attach
    let selected = choices.selected_cards.first().copied().ok_or_else(|| {
        crate::error::EngineError::InvalidAction("Must select a Lightning energy".into())
    })?;

    // Validate selection is in top 5
    if !lightning_ids.contains(&selected) {
        return Err(crate::error::EngineError::InvalidAction(
            "Selected card not in top 5".into(),
        ));
    }

    // Remove from deck
    state.players[player.0].deck.retain(|&id| id != selected);

    // Attach to target Pokemon
    if let Some(target_slot) = choices.selected_slots.first() {
        if let Some(slot) = state.players[player.0].get_slot_mut(*target_slot) {
            slot.energies.push(selected);
        }
    }

    Ok(EffectResult::new())
}

/// Buddy-Buddy Poffin: search deck for up to 2 Basic Pokemon with HP <= 70, put on bench.
pub fn effect_buddy_poffin(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    use crate::card::Stage;
    let max_bench = crate::MAX_BENCH_SIZE;
    let player_state = &state.players[player.0];
    if player_state.bench_count() >= max_bench {
        return Ok(EffectResult::new());
    }
    let mut found: Vec<crate::state::CardInstanceId> = Vec::new();
    for &card_id in player_state.deck.iter().rev() {
        if found.len() >= 2 { break; }
        if let Some(def) = state.get_card_def(card_id) {
            if def.is_pokemon() && def.stage == Some(Stage::Basic) && def.hp.unwrap_or(999) <= 70 {
                found.push(card_id);
            }
        }
    }
    let player_state = &mut state.players[player.0];
    for card_id in found {
        if let Some(pos) = player_state.deck.iter().position(|&id| id == card_id) {
            player_state.deck.remove(pos);
            if let Some(slot) = player_state.bench.iter_mut().find(|s| s.is_none()) {
                let mut pokemon = crate::state::PokemonSlot::new();
                pokemon.cards.push(card_id);
                pokemon.turn_put_in_play = state.turn.turn_number;
                *slot = Some(pokemon);
            }
        }
    }
    Ok(EffectResult::new())
}

/// Super Rod: shuffle up to 3 Pokemon + Basic Energy from discard pile into deck.
pub fn effect_super_rod(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    let player_state = &state.players[player.0];
    let mut to_recover: Vec<crate::state::CardInstanceId> = Vec::new();
    for &card_id in player_state.discard.iter().rev() {
        if to_recover.len() >= 3 { break; }
        if let Some(def) = state.get_card_def(card_id) {
            if def.is_pokemon() || def.is_basic_energy() {
                to_recover.push(card_id);
            }
        }
    }
    let player_state = &mut state.players[player.0];
    for card_id in &to_recover {
        if let Some(pos) = player_state.discard.iter().position(|&id| id == *card_id) {
            player_state.discard.remove(pos);
            player_state.deck.push(*card_id);
        }
    }
    Ok(EffectResult::new())
}

/// Professor's Research: discard hand, draw 7.
pub fn effect_professors_research(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    let hand: Vec<_> = state.players[player.0].hand.drain(..).collect();
    state.players[player.0].discard.extend(hand);
    state.draw_cards(player, 7);
    Ok(EffectResult::new())
}

/// Lost Vacuum: discard a Tool or Stadium in play.
pub fn effect_lost_vacuum(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    let opp = player.opponent();
    // Discard opponent's active tool if any
    if let Some(slot) = state.players[opp.0].active.as_mut() {
        if let Some(tool) = slot.tool.take() {
            state.players[opp.0].discard.push(tool);
            return Ok(EffectResult::new());
        }
    }
    // Discard stadium
    if let Some(stadium) = state.players[opp.0].stadium.take() {
        state.players[opp.0].discard.push(stadium.card_id);
    } else if let Some(stadium) = state.players[player.0].stadium.take() {
        state.players[player.0].discard.push(stadium.card_id);
    }
    Ok(EffectResult::new())
}

/// Earthen Vessel: discard 1 card, search 2 basic energy from deck.
pub fn effect_earthen_vessel(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    // Discard 1 from hand
    if let Some(card) = state.players[player.0].hand.pop() {
        state.players[player.0].discard.push(card);
    }
    // Search 2 basic energy
    let mut found = Vec::new();
    for &eid in state.players[player.0].deck.iter().rev() {
        if found.len() >= 2 { break; }
        if state.get_card_def(eid).map(|d| d.is_basic_energy()).unwrap_or(false) {
            found.push(eid);
        }
    }
    for eid in &found {
        if let Some(pos) = state.players[player.0].deck.iter().position(|&id| id == *eid) {
            state.players[player.0].deck.remove(pos);
            state.players[player.0].hand.push(*eid);
        }
    }
    Ok(EffectResult::new())
}

/// Night Stretcher: recover 1 Pokemon or Basic Energy from discard to hand.
pub fn effect_night_stretcher(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    let pos = state.players[player.0].discard.iter().position(|&id| {
        state.get_card_def(id).map(|d| d.is_pokemon() || d.is_basic_energy()).unwrap_or(false)
    });
    if let Some(idx) = pos {
        let card = state.players[player.0].discard.remove(idx);
        state.players[player.0].hand.push(card);
    }
    Ok(EffectResult::new())
}

/// Dark Patch: attach 1 basic Darkness energy from discard to benched Darkness Pokemon.
pub fn effect_dark_patch(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    let pos = state.players[player.0].discard.iter().position(|&id| {
        state.get_card_def(id).map(|d| d.is_basic_energy() && d.energy_type == Some(crate::card::EnergyType::Darkness)).unwrap_or(false)
    });
    if let Some(idx) = pos {
        let energy = state.players[player.0].discard.remove(idx);
        // Attach to first benched Darkness Pokemon
        let bench_idx = state.players[player.0].bench.iter().position(|s| {
            s.as_ref().and_then(|s| s.top_card()).and_then(|id| state.get_card_def(id))
                .map(|d| d.energy_type == Some(crate::card::EnergyType::Darkness)).unwrap_or(false)
        });
        if let (Some(bi), Some(slot)) = (bench_idx, state.players[player.0].bench.get_mut(bench_idx.unwrap_or(0)).and_then(|s| s.as_mut())) {
            slot.energies.push(energy);
        } else if let Some(slot) = state.players[player.0].active.as_mut() {
            slot.energies.push(energy);
        }
    }
    Ok(EffectResult::new())
}

/// Energy Switch: move 1 basic energy from 1 Pokemon to another.
pub fn effect_energy_switch(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    // Move 1 energy from active to bench
    let energy = state.players[player.0].active.as_mut().and_then(|s| s.energies.pop());
    if let Some(eid) = energy {
        let bench = state.players[player.0].bench.iter_mut().find(|s| s.as_ref().map(|s| !s.is_empty()).unwrap_or(false));
        if let (Some(slot), _) = (bench, ()) {
            if let Some(s) = slot.as_mut() {
                s.energies.push(eid);
            }
        } else {
            state.players[player.0].active.as_mut().map(|s| s.energies.push(eid));
        }
    }
    Ok(EffectResult::new())
}

/// Pal Pad: shuffle up to 2 Supporter cards from discard into deck.
pub fn effect_pal_pad(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    let supporters: Vec<_> = {
        let ps = &state.players[player.0];
        ps.discard.iter().enumerate().filter(|(_, &id)| {
            state.get_card_def(id).map(|d| d.card_type == crate::card::CardType::Supporter).unwrap_or(false)
        }).take(2).map(|(i, &id)| (i, id)).collect()
    };
    for (pos, eid) in supporters.iter().rev() {
        let ps = &mut state.players[player.0];
        if let Some(p) = ps.discard.iter().position(|&id| id == *eid) {
            ps.discard.remove(p);
            ps.deck.push(*eid);
        }
    }
    Ok(EffectResult::new())
}

/// Hisuian Heavy Ball: look at prize cards, swap 1 with a card in hand.
pub fn effect_hisuian_heavy_ball(state: &mut GameState, player: PlayerId) -> Result<EffectResult> {
    // Simplified: draw a prize card (take top prize)
    if let Some(prize) = state.players[player.0].prizes.pop() {
        state.players[player.0].hand.push(prize);
    }
    Ok(EffectResult::new())
}
