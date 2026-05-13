//! Common effect primitives

use crate::error::Result;
use crate::state::{CardInstanceId, GameState, PlayerId, PokemonSlot, SlotRef};

/// Effect result
#[derive(Debug, Clone)]
pub struct EffectResult {
    pub events: Vec<crate::engine::Event>,
}

/// Result of resolving an attack with effects.
#[derive(Debug, Clone)]
pub struct AttackResult {
    /// Base damage to the defending active Pokemon
    pub damage: u16,
    /// Whether the defending active Pokemon is KO'd
    pub ko: bool,
    /// Additional bench damage: (target_slot, damage_amount)
    pub bench_damage: Vec<(SlotRef, u16)>,
    /// Events generated during attack resolution
    pub events: Vec<crate::engine::Event>,
    /// Whether this attack locks itself next turn
    pub self_lock: bool,
}

impl AttackResult {
    pub fn new(damage: u16) -> Self {
        Self { damage, ko: false, bench_damage: vec![], events: vec![], self_lock: false }
    }

    pub fn with_ko(damage: u16) -> Self {
        Self { damage, ko: true, bench_damage: vec![], events: vec![], self_lock: false }
    }
}

impl EffectResult {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn with_event(mut self, event: crate::engine::Event) -> Self {
        self.events.push(event);
        self
    }
}

impl Default for EffectResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Draw cards effect
pub fn draw_cards(state: &mut GameState, player: PlayerId, count: usize) -> Vec<CardInstanceId> {
    state.draw_cards(player, count)
}

/// Search deck effect
pub fn search_deck(
    state: &mut GameState,
    player: PlayerId,
    count: usize,
    filter: impl Fn(&crate::card::CardDef) -> bool,
) -> Vec<CardInstanceId> {
    let deck_ids: Vec<_> = state.players[player.0].deck.iter().copied().collect();

    let mut found = Vec::new();
    let mut remaining = Vec::new();

    for card_id in deck_ids.iter().rev() {
        if found.len() < count {
            if let Some(card_def) = state.get_card_def(*card_id) {
                if filter(card_def) {
                    found.push(*card_id);
                    continue;
                }
            }
        }
        remaining.push(*card_id);
    }

    // Update deck and hand
    state.players[player.0].deck = remaining;
    for card_id in &found {
        state.players[player.0].hand.push(*card_id);
    }

    found
}

/// Search deck and attach energy
pub fn search_deck_attach_energy(
    state: &mut GameState,
    player: PlayerId,
    energy_filter: impl Fn(&crate::card::CardDef) -> bool,
    target: SlotRef,
) -> Result<Option<CardInstanceId>> {
    let deck_ids: Vec<_> = state.players[player.0].deck.iter().copied().collect();

    // Find energy card
    for (i, &card_id) in deck_ids.iter().enumerate().rev() {
        if let Some(card_def) = state.get_card_def(card_id) {
            if energy_filter(card_def) {
                // Remove from deck
                state.players[player.0].deck.retain(|&id| id != card_id);
                state.players[player.0].hand.push(card_id);

                // Attach to target
                if let Some(slot) = state.players[player.0].get_slot_mut(target) {
                    slot.energies.push(card_id);
                }

                return Ok(Some(card_id));
            }
        }
    }

    Ok(None)
}

/// Switch active with bench
pub fn switch_pokemon(state: &mut GameState, player: PlayerId, target: SlotRef) -> Result<()> {
    let player_state = &mut state.players[player.0];

    let bench_index = target
        .bench_index()
        .ok_or_else(|| crate::error::EngineError::InvalidAction("Must target bench slot".into()))?;

    // Swap active and bench
    let active = player_state.active.take();
    let bench_pokemon = player_state.bench[bench_index].take();

    if let Some(pokemon) = bench_pokemon {
        player_state.active = Some(pokemon);
    }

    if let Some(pokemon) = active {
        player_state.bench[bench_index] = Some(pokemon);
    }

    Ok(())
}

/// Apply damage to a Pokemon
pub fn apply_damage(
    state: &mut GameState,
    target_player: PlayerId,
    target_slot: SlotRef,
    damage: u16,
) -> bool {
    let top_card = state.players[target_player.0]
        .get_slot(target_slot)
        .and_then(|s| s.top_card());

    let max_hp = top_card
        .and_then(|id| state.get_card_def(id))
        .and_then(|def| def.hp);

    if let Some(slot) = state.players[target_player.0].get_slot_mut(target_slot) {
        slot.damage += damage;

        // Check if KO'd
        if let Some(hp) = max_hp {
            if slot.damage >= hp {
                return true; // KO'd
            }
        }
    }

    false
}

/// Move Pokemon to discard/lost zone
pub fn discard_pokemon(
    state: &mut GameState,
    player: PlayerId,
    slot: SlotRef,
    to_lost_zone: bool,
) -> Vec<CardInstanceId> {
    let player_state = &mut state.players[player.0];
    let mut discarded = Vec::new();

    if let Some(pokemon) = player_state.get_slot_mut(slot) {
        // Collect all cards
        discarded.extend(pokemon.cards.drain(..));
        discarded.extend(pokemon.energies.drain(..));
        if let Some(tool) = pokemon.tool.take() {
            discarded.push(tool);
        }

        // Move to discard or lost zone
        let destination = if to_lost_zone {
            &mut player_state.lost_zone
        } else {
            &mut player_state.discard
        };

        destination.extend(discarded.clone());
    }

    discarded
}

/// Evolve Pokemon
pub fn evolve_pokemon(
    state: &mut GameState,
    player: PlayerId,
    card: CardInstanceId,
    target: SlotRef,
) -> Result<()> {
    let player_state = &mut state.players[player.0];

    // Remove card from hand
    let hand = &mut player_state.hand;
    let pos = hand
        .iter()
        .position(|&id| id == card)
        .ok_or_else(|| crate::error::EngineError::InvalidAction("Card not in hand".into()))?;
    hand.remove(pos);

    // Add to evolution stack
    if let Some(slot) = player_state.get_slot_mut(target) {
        slot.cards.push(card);
        slot.turn_evolved = Some(state.turn.turn_number);
    }

    Ok(())
}
