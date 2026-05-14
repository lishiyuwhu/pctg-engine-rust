//! Deck strategy — heuristic opponent AI.
//!
//! Ports the core decision heuristics from PtcgDeckAgent's GDScript
//! DeckStrategyMiraidon and DeckStrategyCharizardEx.
//!
//! Four core heuristics (enough to produce meaningful games):
//! H1: KO-aware attack scoring
//! H2: Attack-gap energy routing
//! H3: Bench slot budgeting
//! H4: Retreat only with a plan

use crate::action::Action;
use crate::card::{CardDef, EnergyType, Stage};
use crate::state::{GameState, Phase, PlayerId, SlotRef};

// ── Trait ───────────────────────────────────────────────────────────

/// Scores legal actions for heuristic opponent play.
pub trait DeckStrategy: Send + Sync {
    /// Absolute score for an action. Higher = better.
    fn score_action(&self, action: &Action, state: &GameState, player: PlayerId) -> f32;

    /// Baseline heuristic estimate for cross-type normalization.
    fn heuristic_base(&self, action: &Action) -> f32;

    /// Relative score = absolute - base.
    fn score_relative(&self, action: &Action, state: &GameState, player: PlayerId) -> f32 {
        let abs = self.score_action(action, state, player);
        let base = self.heuristic_base(action);
        abs - base
    }

    /// Select the best action by relative score.
    fn select_action(&self, actions: &[Action], state: &GameState, player: PlayerId) -> Option<usize> {
        actions.iter().enumerate().map(|(i, a)| {
            (i, self.score_relative(a, state, player))
        }).max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal)).map(|(i, _)| i)
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

/// Count how many more energy a Pokemon slot needs for its cheapest attack.
fn attack_energy_gap(slot: &crate::state::PokemonSlot, card: &CardDef) -> i32 {
    let energy_count = slot.energies.len() as i32;
    let cheapest_cost = card.attacks.iter().map(|a| a.cost.len() as i32).min().unwrap_or(999);
    (cheapest_cost - energy_count).max(0)
}

/// Does the slot have a basic Lightning energy attached?
fn has_lightning_energy(slot: &crate::state::PokemonSlot, state: &GameState) -> bool {
    slot.energies.iter().any(|&eid| {
        state.get_card_def(eid).map(|d| d.energy_type == Some(EnergyType::Lightning)).unwrap_or(false)
    })
}

/// Is the opponent's active Pokemon within KO range of the given damage?
fn would_ko_opponent(state: &GameState, player: PlayerId, damage: u16) -> bool {
    let opponent = player.opponent();
    state.players[opponent.0].active.as_ref().map_or(false, |s| {
        s.top_card()
            .and_then(|id| state.get_card_def(id))
            .and_then(|d| d.hp)
            .map(|hp| s.damage + damage >= hp)
            .unwrap_or(false)
    })
}

/// Is this Pokemon an ex or V Pokemon (worth extra prizes)?
fn is_rule_box(card: &CardDef) -> bool {
    card.is_ex() || card.is_v() || card.is_vstar()
}

/// Can this bench Pokemon attack immediately (gap 0)?
fn can_attack_now(slot: &crate::state::PokemonSlot, state: &GameState) -> bool {
    slot.top_card().and_then(|id| state.get_card_def(id)).map_or(false, |card| {
        attack_energy_gap(slot, card) == 0
    })
}

// Convenience: get card def from slot
fn slot_card<'a>(slot: &crate::state::PokemonSlot, state: &'a GameState) -> Option<&'a CardDef> {
    slot.top_card().and_then(|id| state.get_card_def(id))
}

// ── Miraidon Strategy ───────────────────────────────────────────────

pub struct MiraidonStrategy;

impl MiraidonStrategy {
    /// Core Pokemon names that should be benched early.
    fn is_core_pokemon(name: &str) -> bool {
        matches!(name, "Miraidon ex" | "Iron Hands ex" | "Raikou V" | "Zapdos" | "Raichu V")
    }

    /// Lightning-type attacker names eligible for energy.
    fn is_lightning_attacker(name: &str) -> bool {
        matches!(name, "Miraidon ex" | "Iron Hands ex" | "Raikou V" | "Raichu V" | "Zapdos" | "Radiant Greninja")
    }
}

impl DeckStrategy for MiraidonStrategy {
    fn score_action(&self, action: &Action, state: &GameState, player: PlayerId) -> f32 {
        let player_state = &state.players[player.0];
        let opponent = player.opponent();
        let bench_count = player_state.bench_count();
        let has_active = player_state.active.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
        let active_card = player_state.active.as_ref().and_then(|s| slot_card(s, state));

        match action {
            // ── H1: KO-aware attack scoring ──
            Action::Attack { attack_index, .. } => {
                let damage = active_card
                    .and_then(|c| c.attacks.get(*attack_index as usize))
                    .map(|a| a.damage)
                    .unwrap_or(0);
                let would_ko = would_ko_opponent(state, player, damage);
                let opp_active = state.players[opponent.0].active.as_ref();
                let is_rule_box_target = opp_active
                    .and_then(|s| s.top_card())
                    .and_then(|id| state.get_card_def(id))
                    .map(|d| is_rule_box(d))
                    .unwrap_or(false);

                if would_ko {
                    if is_rule_box_target { 1200.0 }
                    else { 1000.0 }
                } else if damage >= 120 {
                    400.0 + damage as f32
                } else {
                    300.0 + damage as f32
                }
            }

            // ── H2: Energy gap routing ──
            Action::AttachEnergy { card: _, target } => {
                let slot = player_state.get_slot(*target);
                let card = slot.and_then(|s| slot_card(s, state));
                match card {
                    Some(c) if Self::is_lightning_attacker(&c.name) => {
                        let gap = attack_energy_gap(slot.unwrap(), c);
                        match gap {
                            1 => 550.0,   // One away from attacking
                            0 => 80.0,    // Already ready
                            2 => 350.0,   // Needs charging
                            _ => 260.0,
                        }
                    }
                    Some(c) if c.name == "Miraidon ex" => {
                        // Only attach to Miraidon if it's active and needs to attack/retreat
                        if matches!(*target, SlotRef::Active) {
                            let gap = attack_energy_gap(slot.unwrap(), c);
                            if gap == 1 { 380.0 } else { 100.0 }
                        } else { -50.0 } // Don't waste energy on bench Miraidon
                    }
                    _ => 40.0, // Generic fallback
                }
            }

            // ── H3: Bench slot budgeting ──
            Action::PlayBasicToBench { card: _ } => {
                if bench_count >= 5 { return 0.0; }

                // We can't easily identify the card from PlayBasicToBench action
                // (only has CardInstanceId). Score based on bench availability.
                let free_slots = 5 - bench_count;
                let essentials_needed = if !player_state.bench.iter().any(|s| {
                    s.as_ref().and_then(|s| slot_card(s, state)).map_or(false, |c| c.name == "Miraidon ex")
                }) { 1 } else { 0 }
                    + if !player_state.bench.iter().any(|s| {
                        s.as_ref().and_then(|s| slot_card(s, state)).map_or(false, |c| c.name == "Iron Hands ex")
                    }) { 1 } else { 0 };

                if free_slots <= essentials_needed {
                    350.0 // Need core pokemon
                } else if bench_count < 3 {
                    200.0 // Still building bench
                } else {
                    100.0 // Filling extra slots
                }
            }

            // ── H4: Retreat only with a plan ──
            Action::Retreat { target, .. } => {
                if !has_active { return 0.0; }
                let active_can_attack = active_card.map_or(false, |c| {
                    player_state.active.as_ref().map_or(false, |s| attack_energy_gap(s, c) == 0)
                });

                let bench_can_attack = match target {
                    SlotRef::Bench(i) => player_state.bench[*i].as_ref()
                        .map_or(false, |s| can_attack_now(s, state)),
                    _ => false,
                };

                if active_can_attack {
                    -200.0 // Don't retreat an attacker!
                } else if bench_can_attack {
                    550.0 // Good: pivot to ready attacker
                } else {
                    -150.0 // No plan: let it get KO'd instead
                }
            }

            // ── Ability ──
            Action::UseAbility { .. } => 400.0,

            // ── Trainer play ──
            Action::PlayTrainer { .. } => 220.0,

            // ── Evolve ──
            Action::Evolve { .. } => 500.0,

            // ── End turn ──
            Action::EndTurn => 0.0,

            // ── Setup actions ──
            Action::SetupChooseActive { .. } => 500.0,
            Action::SetupBenchBasics { .. } => 300.0,

            // Default
            _ => 100.0,
        }
    }

    fn heuristic_base(&self, action: &Action) -> f32 {
        match action {
            Action::Attack { .. } => 500.0,
            Action::AttachEnergy { .. } => 220.0,
            Action::PlayTrainer { .. } => 110.0,
            Action::PlayBasicToBench { .. } => 180.0,
            Action::UseAbility { .. } => 160.0,
            Action::Retreat { .. } => 90.0,
            Action::Evolve { .. } => 300.0,
            Action::EndTurn => 0.0,
            _ => 100.0,
        }
    }
}

// ── Charizard Strategy ──────────────────────────────────────────────

pub struct CharizardStrategy;

impl CharizardStrategy {
    fn is_fire_attacker(name: &str) -> bool {
        matches!(name, "Charizard ex" | "Charmander" | "Charmeleon" | "Radiant Charizard")
    }
}

impl DeckStrategy for CharizardStrategy {
    fn score_action(&self, action: &Action, state: &GameState, player: PlayerId) -> f32 {
        let player_state = &state.players[player.0];
        let opponent = player.opponent();
        let has_active = player_state.active.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
        let active_card = player_state.active.as_ref().and_then(|s| slot_card(s, state));

        match action {
            // ── H1: Attack ──
            Action::Attack { attack_index, .. } => {
                let damage = active_card
                    .and_then(|c| c.attacks.get(*attack_index as usize))
                    .map(|a| a.damage)
                    .unwrap_or(0);
                let would_ko = would_ko_opponent(state, player, damage);
                let opp_active = state.players[opponent.0].active.as_ref();
                let is_rule_box = opp_active
                    .and_then(|s| s.top_card())
                    .and_then(|id| state.get_card_def(id))
                    .map(|d| is_rule_box(d))
                    .unwrap_or(false);

                if would_ko {
                    if is_rule_box { 1200.0 } else { 1000.0 }
                } else if damage >= 180 {
                    500.0 + damage as f32
                } else {
                    300.0 + damage as f32
                }
            }

            // ── H2: Energy (Fire type priority) ──
            Action::AttachEnergy { card: _, target } => {
                let slot = player_state.get_slot(*target);
                let card = slot.and_then(|s| slot_card(s, state));
                match card {
                    Some(c) if Self::is_fire_attacker(&c.name) => {
                        let gap = attack_energy_gap(slot.unwrap(), c);
                        match gap {
                            1 => 640.0,
                            0 => 120.0,
                            2 => 420.0,
                            _ => 300.0,
                        }
                    }
                    _ => 40.0,
                }
            }

            // ── H3: Bench ──
            Action::PlayBasicToBench { .. } => {
                if player_state.bench_count() >= 5 { return 0.0; }
                250.0
            }

            // ── Evolve scoring (very important for Charizard) ──
            Action::Evolve { card: _, target } => {
                let slot = player_state.get_slot(*target);
                let card_name = slot.and_then(|s| slot_card(s, state)).map(|c| c.name.as_str()).unwrap_or("");
                match card_name {
                    "Charizard ex" => 860.0,
                    "Pidgeot ex" => 780.0,
                    "Dusknoir" => 560.0,
                    _ => 480.0,
                }
            }

            // ── H4: Retreat ──
            Action::Retreat { target, .. } => {
                if !has_active { return 0.0; }
                let active_can_attack = active_card.map_or(false, |c| {
                    player_state.active.as_ref().map_or(false, |s| attack_energy_gap(s, c) == 0)
                });
                let bench_can_attack = match target {
                    SlotRef::Bench(i) => player_state.bench[*i].as_ref()
                        .map_or(false, |s| can_attack_now(s, state)),
                    _ => false,
                };
                if active_can_attack { -200.0 }
                else if bench_can_attack { 550.0 }
                else { -150.0 }
            }

            Action::UseAbility { .. } => 400.0,
            Action::PlayTrainer { .. } => 200.0,
            Action::EndTurn => 0.0,
            Action::SetupChooseActive { .. } => 500.0,
            Action::SetupBenchBasics { .. } => 300.0,
            _ => 100.0,
        }
    }

    fn heuristic_base(&self, action: &Action) -> f32 {
        match action {
            Action::Attack { .. } => 500.0,
            Action::AttachEnergy { .. } => 220.0,
            Action::PlayTrainer { .. } => 110.0,
            Action::PlayBasicToBench { .. } => 180.0,
            Action::UseAbility { .. } => 160.0,
            Action::Retreat { .. } => 90.0,
            Action::Evolve { .. } => 300.0,
            Action::EndTurn => 10.0,
            _ => 100.0,
        }
    }
}

// ── Gouging Fire Strategy ───────────────────────────────────────────

pub struct GougingFireStrategy;

impl DeckStrategy for GougingFireStrategy {
    fn score_action(&self, action: &Action, state: &GameState, player: PlayerId) -> f32 {
        let player_state = &state.players[player.0];
        let opponent = player.opponent();
        let has_active = player_state.active.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
        let active_card = player_state.active.as_ref().and_then(|s| slot_card(s, state));

        match action {
            Action::Attack { attack_index, .. } => {
                let damage = active_card.and_then(|c| c.attacks.get(*attack_index as usize)).map(|a| a.damage).unwrap_or(0);
                let would_ko = would_ko_opponent(state, player, damage);
                let opp_active = state.players[opponent.0].active.as_ref();
                let is_rule_box = opp_active.and_then(|s| s.top_card()).and_then(|id| state.get_card_def(id)).map(|d| is_rule_box(d)).unwrap_or(false);
                if would_ko { if is_rule_box { 1200.0 } else { 1000.0 } }
                else if damage >= 200 { 600.0 + damage as f32 }
                else { 400.0 + damage as f32 }
            }
            Action::AttachEnergy { card: _, target } => {
                let slot = player_state.get_slot(*target);
                let card = slot.and_then(|s| slot_card(s, state));
                match card {
                    Some(c) if c.energy_type == Some(EnergyType::Fire) => {
                        let gap = attack_energy_gap(slot.unwrap(), c);
                        match gap { 1 => 550.0, 0 => 80.0, 2 => 350.0, _ => 260.0 }
                    }
                    Some(c) if c.energy_type == Some(EnergyType::Darkness) => {
                        let gap = attack_energy_gap(slot.unwrap(), c);
                        match gap { 1 => 520.0, 0 => 70.0, 2 => 320.0, _ => 240.0 }
                    }
                    _ => 40.0,
                }
            }
            Action::PlayBasicToBench { .. } => {
                if player_state.bench_count() >= 5 { return 0.0; }
                if player_state.bench_count() < 3 { 300.0 } else { 150.0 }
            }
            Action::Retreat { target, .. } => {
                if !has_active { return 0.0; }
                let active_can_attack = active_card.map_or(false, |c| {
                    player_state.active.as_ref().map_or(false, |s| attack_energy_gap(s, c) == 0)
                });
                let bench_can_attack = match target {
                    SlotRef::Bench(i) => player_state.bench[*i].as_ref().map_or(false, |s| can_attack_now(s, state)),
                    _ => false,
                };
                if active_can_attack { -200.0 }
                else if bench_can_attack { 550.0 }
                else { -150.0 }
            }
            Action::UseAbility { .. } => 400.0,
            Action::PlayTrainer { .. } => 200.0,
            Action::Evolve { .. } => 300.0,
            Action::EndTurn => 0.0,
            Action::SetupChooseActive { .. } => 500.0,
            _ => 100.0,
        }
    }

    fn heuristic_base(&self, action: &Action) -> f32 {
        match action {
            Action::Attack { .. } => 500.0,
            Action::AttachEnergy { .. } => 220.0,
            Action::PlayTrainer { .. } => 110.0,
            Action::PlayBasicToBench { .. } => 180.0,
            Action::UseAbility { .. } => 160.0,
            Action::Retreat { .. } => 90.0,
            Action::Evolve { .. } => 200.0,
            Action::EndTurn => 0.0,
            _ => 100.0,
        }
    }
}

// ── Future Box Strategy ─────────────────────────────────────────────

pub struct FutureBoxStrategy;

impl DeckStrategy for FutureBoxStrategy {
    fn score_action(&self, action: &Action, state: &GameState, player: PlayerId) -> f32 {
        let player_state = &state.players[player.0];
        let opponent = player.opponent();
        let has_active = player_state.active.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
        let active_card = player_state.active.as_ref().and_then(|s| slot_card(s, state));

        match action {
            Action::Attack { attack_index, .. } => {
                let damage = active_card.and_then(|c| c.attacks.get(*attack_index as usize)).map(|a| a.damage).unwrap_or(0);
                let would_ko = would_ko_opponent(state, player, damage);
                if would_ko { 1200.0 } else if damage >= 160 { 500.0 + damage as f32 } else { 300.0 + damage as f32 }
            }
            Action::AttachEnergy { card: _, target } => {
                let slot = player_state.get_slot(*target);
                let card = slot.and_then(|s| slot_card(s, state));
                match card {
                    Some(c) if c.energy_type == Some(EnergyType::Lightning) || c.energy_type == Some(EnergyType::Grass) => {
                        let gap = attack_energy_gap(slot.unwrap(), c);
                        match gap { 1 => 550.0, 0 => 80.0, _ => 300.0 }
                    }
                    _ => 40.0,
                }
            }
            Action::PlayBasicToBench { .. } => {
                if player_state.bench_count() >= 5 { 0.0 } else if player_state.bench_count() < 3 { 300.0 } else { 150.0 }
            }
            Action::Retreat { target, .. } => {
                if !has_active { return 0.0; }
                let bench_can_attack = match target {
                    SlotRef::Bench(i) => player_state.bench[*i].as_ref().map_or(false, |s| can_attack_now(s, state)),
                    _ => false,
                };
                let active_can = active_card.map_or(false, |c| player_state.active.as_ref().map_or(false, |s| attack_energy_gap(s, c) == 0));
                if active_can { -200.0 } else if bench_can_attack { 550.0 } else { -100.0 }
            }
            Action::UseAbility { .. } => 350.0,
            Action::PlayTrainer { .. } => 200.0,
            Action::Evolve { .. } => 200.0,
            Action::EndTurn => 0.0,
            Action::SetupChooseActive { .. } => 500.0,
            _ => 100.0,
        }
    }

    fn heuristic_base(&self, action: &Action) -> f32 {
        match action {
            Action::Attack { .. } => 500.0,
            Action::AttachEnergy { .. } => 220.0,
            Action::PlayTrainer { .. } => 110.0,
            Action::PlayBasicToBench { .. } => 180.0,
            Action::UseAbility { .. } => 160.0,
            Action::Retreat { .. } => 90.0,
            Action::Evolve { .. } => 200.0,
            Action::EndTurn => 0.0,
            _ => 100.0,
        }
    }
}

// ── Generic strategy for any deck ────────────────────────────────────

/// A generic strategy that works for any deck using standard heuristics.
/// Deck-specific strategies can override energy-type or card-name priorities.
macro_rules! generic_strategy {
    ($name:ident) => {
        pub struct $name;
        impl DeckStrategy for $name {
            fn score_action(&self, action: &Action, state: &GameState, player: PlayerId) -> f32 {
                let ps = &state.players[player.0];
                match action {
                    Action::Attack { attack_index, .. } => {
                        let dmg = ps.active.as_ref().and_then(|s| slot_card(s, state))
                            .and_then(|c| c.attacks.get(*attack_index as usize)).map(|a| a.damage).unwrap_or(0);
                        if would_ko_opponent(state, player, dmg) { 1200.0 }
                        else if dmg >= 150 { 500.0 + dmg as f32 }
                        else { 300.0 + dmg as f32 }
                    }
                    Action::AttachEnergy { card: _, target } => {
                        let card = ps.get_slot(*target).and_then(|s| slot_card(s, state));
                        match card {
                            Some(c) => { let gap = attack_energy_gap(ps.get_slot(*target).unwrap(), c);
                                match gap { 1 => 550.0, 0 => 80.0, _ => 300.0 } }
                            _ => 40.0
                        }
                    }
                    Action::PlayBasicToBench { .. } =>
                        if ps.bench_count() >= 5 { 0.0 } else { 250.0 },
                    Action::Retreat { target, .. } => {
                        let bench_ok = match target { SlotRef::Bench(i) => ps.bench[*i].as_ref().map_or(false, |s| can_attack_now(s, state)), _ => false };
                        let active_ok = ps.active.as_ref().and_then(|s| slot_card(s, state)).map_or(false, |c| attack_energy_gap(ps.active.as_ref().unwrap(), c) == 0);
                        if active_ok { -200.0 } else if bench_ok { 550.0 } else { -100.0 }
                    }
                    Action::Evolve { .. } => 500.0,
                    Action::UseAbility { .. } => 400.0,
                    Action::PlayTrainer { .. } => 200.0,
                    Action::EndTurn => 0.0,
                    Action::SetupChooseActive { .. } => 500.0,
                    _ => 100.0,
                }
            }
            fn heuristic_base(&self, action: &Action) -> f32 { match action {
                Action::Attack { .. } => 500.0, Action::AttachEnergy { .. } => 220.0,
                Action::PlayTrainer { .. } => 110.0, Action::PlayBasicToBench { .. } => 180.0,
                Action::UseAbility { .. } => 160.0, Action::Retreat { .. } => 90.0,
                Action::Evolve { .. } => 300.0, Action::EndTurn => 0.0, _ => 100.0
            }}
        }
    };
}

generic_strategy!(IronThornsStrategy);
generic_strategy!(DialgaMetangStrategy);
generic_strategy!(PalkiaDusknoirStrategy);
generic_strategy!(PalkiaGholdengoStrategy);
generic_strategy!(LostBoxStrategy);
generic_strategy!(RegidragoStrategy);
generic_strategy!(LugiaArcheopsStrategy);
generic_strategy!(ArceusGiratinaStrategy);
generic_strategy!(RagingBoltOgerponStrategy);
generic_strategy!(GardevoirStrategy);
generic_strategy!(BlisseyTankStrategy);
generic_strategy!(DragapultBanetteStrategy);
generic_strategy!(DragapultDusknoirStrategy);
generic_strategy!(DragapultCharizardStrategy);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deck::MatchConfig;
    use crate::engine::Engine;

    #[test]
    fn test_miraidon_strategy_scores() {
        let engine = Engine::new(MatchConfig::default(), 43);
        let strategy = MiraidonStrategy;

        let actions = engine.legal_actions(PlayerId(0));
        assert!(!actions.is_empty());

        // Every action should have a non-negative absolute score
        for action in &actions {
            let score = strategy.score_action(action, engine.state(), PlayerId(0));
            assert!(score >= -200.0, "Score should be >= -200 for {:?}", action);
        }
    }

    #[test]
    fn test_charizard_strategy_scores() {
        let engine = Engine::new(MatchConfig::default(), 43);
        let strategy = CharizardStrategy;

        let actions = engine.legal_actions(PlayerId(0));
        assert!(!actions.is_empty());

        for action in &actions {
            let score = strategy.score_action(action, engine.state(), PlayerId(0));
            assert!(score >= -200.0, "Score should be >= -200 for {:?}", action);
        }
    }

    #[test]
    fn test_attack_energy_gap() {
        let engine = Engine::new(MatchConfig::default(), 43);
        let state = engine.state();
        let player_state = &state.players[0];

        if let Some(active) = &player_state.active {
            if let Some(card) = slot_card(active, state) {
                let gap = attack_energy_gap(active, card);
                let energy = active.energies.len();
                let cost = card.attacks.first().map(|a| a.cost.len()).unwrap_or(0);
                assert_eq!(gap, ((cost as i32) - (energy as i32)).max(0));
            }
        }
    }
}
