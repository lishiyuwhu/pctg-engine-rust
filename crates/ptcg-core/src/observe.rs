//! Observation encoding for RL

use serde::{Deserialize, Serialize};
use crate::state::{GameState, PlayerId, SlotRef};
use crate::card::CardDef;
use crate::MAX_BENCH_SIZE;

/// Observation for a player
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub player_id: PlayerId,
    pub turn: u16,
    pub phase: String,
    
    // Deck sizes (not contents)
    pub player_deck_size: usize,
    pub opponent_deck_size: usize,
    
    // Hand sizes
    pub player_hand_size: usize,
    pub opponent_hand_size: usize,
    
    // Prize counts
    pub player_prizes: usize,
    pub opponent_prizes: usize,
    
    // Active Pokemon info
    pub player_active: Option<PokemonObservation>,
    pub opponent_active: Option<PokemonObservation>,
    
    // Bench info
    pub player_bench: Vec<Option<PokemonObservation>>,
    pub opponent_bench: Vec<Option<PokemonObservation>>,
    
    // Stadium
    pub player_stadium: Option<String>,
    pub opponent_stadium: Option<String>,
    
    // Can attack this turn
    pub can_attack: bool,
    pub attack_locked: bool,
}

/// Pokemon observation (public info only)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonObservation {
    pub hp: u16,
    pub max_hp: u16,
    pub damage: u16,
    pub energy_count: usize,
    pub has_tool: bool,
    pub is_ex: bool,
    pub is_v: bool,
    pub is_vstar: bool,
    pub stage: String,
    pub energy_type: String,
    pub attack_count: usize,
}

impl Observation {
    /// Create observation for a player (hides private information)
    pub fn from_state(state: &GameState, player: PlayerId) -> Self {
        let player_state = &state.players[player.0];
        let opponent_state = &state.players[player.opponent().0];
        
        Self {
            player_id: player,
            turn: state.turn.turn_number,
            phase: format!("{:?}", state.turn.phase),
            
            player_deck_size: player_state.deck.len(),
            opponent_deck_size: opponent_state.deck.len(),
            
            player_hand_size: player_state.hand.len(),
            opponent_hand_size: opponent_state.hand.len(), // Only size, not contents
            
            player_prizes: player_state.prizes.len(),
            opponent_prizes: opponent_state.prizes.len(),
            
            player_active: player_state.active.as_ref()
                .and_then(|s| Self::pokemon_to_observation(state, s)),
            
            opponent_active: opponent_state.active.as_ref()
                .and_then(|s| Self::pokemon_to_observation(state, s)),
            
            player_bench: player_state.bench.iter()
                .map(|s| s.as_ref().and_then(|slot| Self::pokemon_to_observation(state, slot)))
                .collect(),
            
            opponent_bench: opponent_state.bench.iter()
                .map(|s| s.as_ref().and_then(|slot| Self::pokemon_to_observation(state, slot)))
                .collect(),
            
            player_stadium: player_state.stadium.as_ref()
                .and_then(|s| state.get_card(s.card_id).map(|c| c.def_id.0.clone())),
            
            opponent_stadium: opponent_state.stadium.as_ref()
                .and_then(|s| state.get_card(s.card_id).map(|c| c.def_id.0.clone())),
            
            can_attack: state.turn.active_player == player && state.turn.phase == crate::state::Phase::Play,
            attack_locked: state.turn.attack_locked,
        }
    }

    fn pokemon_to_observation(state: &GameState, slot: &crate::state::PokemonSlot) -> Option<PokemonObservation> {
        let top_card = slot.top_card()?;
        let card_def = state.get_card_def(top_card)?;
        
        Some(PokemonObservation {
            hp: card_def.hp.unwrap_or(0).saturating_sub(slot.damage),
            max_hp: card_def.hp.unwrap_or(0),
            damage: slot.damage,
            energy_count: slot.energies.len(),
            has_tool: slot.tool.is_some(),
            is_ex: card_def.is_ex(),
            is_v: card_def.is_v(),
            is_vstar: card_def.is_vstar(),
            stage: format!("{:?}", card_def.stage),
            energy_type: card_def.energy_type.map(|e| format!("{:?}", e)).unwrap_or_default(),
            attack_count: card_def.attacks.len(),
        })
    }

    /// Dimension of the extended observation vector.
    pub fn vector_dim_extended() -> usize {
        // Turn/phase: 1 + 8 + 1 = 10
        // Deck/hand/prizes: 6
        // Active x2: 8 + 8 = 16
        // Bench x2: 5*6 + 5*6 = 60
        // Energy active x2: 10 + 10 = 20 (energy type counts per active)
        // Energy bench x2: 2 + 2 = 4 (total bench energy count, avg per slot)
        // Hand composition: 8
        // Stadium: 3
        // Action flags: 6
        // Discard/lost: 2 + 2 = 4
        // Padding: 3
        // Total: 10+6+16+60+20+4+8+3+6+4+3 = 140
        140
    }

    /// Extended observation vector (~140 features) for RL training.
    pub fn to_vector_extended(&self) -> Vec<f32> {
        let mut v = Vec::with_capacity(Self::vector_dim_extended());

        // ── Turn/phase info (10) ──
        v.push(self.turn as f32 / 20.0);
        // Phase one-hot (8 phases)
        let phases = ["Setup", "Mulligan", "Draw", "Energy", "Play", "Retreat", "Attack", "End"];
        for p in &phases {
            v.push(if self.phase == *p { 1.0 } else { 0.0 });
        }
        v.push(if self.can_attack { 1.0 } else { 0.0 }); // is_active_player

        // ── Deck/hand/prize sizes (6) ──
        v.push(self.player_deck_size as f32 / 60.0);
        v.push(self.opponent_deck_size as f32 / 60.0);
        v.push(self.player_hand_size as f32 / 10.0);
        v.push(self.opponent_hand_size as f32 / 10.0);
        v.push(self.player_prizes as f32 / 6.0);
        v.push(self.opponent_prizes as f32 / 6.0);

        // ── Active Pokemon: player (8) ──
        Self::push_active_features(&mut v, &self.player_active);

        // ── Active Pokemon: opponent (8) ──
        Self::push_active_features(&mut v, &self.opponent_active);

        // ── Bench: player 5 slots x 6 features (30) ──
        Self::push_bench_features(&mut v, &self.player_bench, 5);

        // ── Bench: opponent 5 slots x 6 features (30) ──
        Self::push_bench_features(&mut v, &self.opponent_bench, 5);

        // ── Energy on active Pokemon (player) (10) ──
        Self::push_energy_features(&mut v, &self.player_active);

        // ── Energy on active Pokemon (opponent) (10) ──
        Self::push_energy_features(&mut v, &self.opponent_active);

        // ── Energy on bench (player) (2) ──
        let p_bench_energy: usize = self.player_bench.iter()
            .filter_map(|p| p.as_ref())
            .map(|p| p.energy_count)
            .sum();
        v.push(p_bench_energy as f32 / 10.0);
        let p_bench_count = self.player_bench.iter().filter(|p| p.is_some()).count().max(1);
        v.push((p_bench_energy as f32 / p_bench_count as f32) / 10.0); // avg per slot

        // ── Energy on bench (opponent) (2) ──
        let o_bench_energy: usize = self.opponent_bench.iter()
            .filter_map(|p| p.as_ref())
            .map(|p| p.energy_count)
            .sum();
        v.push(o_bench_energy as f32 / 10.0);
        let o_bench_count = self.opponent_bench.iter().filter(|p| p.is_some()).count().max(1);
        v.push((o_bench_energy as f32 / o_bench_count as f32) / 10.0);

        // ── Hand composition for player (8) ──
        // Note: detailed hand composition requires state access beyond Observation.
        // We fill with hand_size-derived features as a proxy.
        // The RL agent can infer hand composition from played cards.
        for _ in 0..8 {
            v.push(0.0); // placeholder — filled by caller if state info available
        }

        // ── Stadium info (3) ──
        v.push(if self.player_stadium.is_some() { 1.0 } else { 0.0 });
        v.push(if self.opponent_stadium.is_some() { 1.0 } else { 0.0 });
        v.push(0.0); // stadium turns remaining placeholder

        // ── Action flags (6) ──
        v.push(if self.can_attack { 1.0 } else { 0.0 });
        v.push(if self.attack_locked { 1.0 } else { 0.0 });
        v.push(0.0); // can_retreat placeholder
        v.push(if self.phase == "Play" { 1.0 } else { 0.0 }); // is_play_phase
        v.push(0.0); // energy_attached_this_turn placeholder
        v.push(0.0); // supporter_used placeholder

        // ── Discard / Lost Zone sizes (4) ──
        v.push(0.0); // player_discard placeholder
        v.push(0.0); // opponent_discard placeholder
        v.push(0.0); // player_lost_zone placeholder
        v.push(0.0); // opponent_lost_zone placeholder

        // ── Padding (3) ──
        v.push(0.0);
        v.push(0.0);
        v.push(0.0);

        debug_assert_eq!(v.len(), Self::vector_dim_extended(),
            "Vector dimension mismatch: {} != {}", v.len(), Self::vector_dim_extended());

        v
    }

    fn push_active_features(v: &mut Vec<f32>, active: &Option<PokemonObservation>) {
        if let Some(a) = active {
            v.push(a.hp as f32 / 400.0);
            v.push(a.max_hp as f32 / 400.0);
            v.push(a.damage as f32 / 400.0);
            v.push(a.energy_count as f32 / 10.0);
            v.push(if a.has_tool { 1.0 } else { 0.0 });
            v.push(if a.is_ex { 1.0 } else { 0.0 });
            v.push(if a.is_v { 1.0 } else { 0.0 });
            v.push(a.attack_count as f32 / 4.0);
        } else {
            v.extend_from_slice(&[0.0; 8]);
        }
    }

    fn push_bench_features(v: &mut Vec<f32>, bench: &[Option<PokemonObservation>], max_slots: usize) {
        for i in 0..max_slots {
            if let Some(p) = bench.get(i).and_then(|s| s.as_ref()) {
                v.push(1.0); // is_occupied
                v.push(p.hp as f32 / 400.0);
                v.push(p.damage as f32 / 400.0);
                v.push(p.energy_count as f32 / 10.0);
                v.push(if p.is_ex { 1.0 } else { 0.0 });
                v.push(match p.stage.as_str() {
                    "Some(Basic)" | "Basic" => 0.0,
                    "Some(Stage1)" | "Stage1" => 0.5,
                    "Some(Stage2)" | "Stage2" => 1.0,
                    _ => 0.0,
                });
            } else {
                v.extend_from_slice(&[0.0; 6]);
            }
        }
    }

    fn push_energy_features(v: &mut Vec<f32>, active: &Option<PokemonObservation>) {
        // Energy type distribution on the active Pokemon.
        // We encode 10 normalized energy type count features.
        // Without per-energy-type tracking in the observation, we use
        // energy_count and energy_type as proxy features.
        if let Some(a) = active {
            v.push(a.energy_count as f32 / 10.0);
            // Use the energy_type string to create a rough type one-hot
            let types = ["Lightning", "Fire", "Water", "Grass", "Psychic",
                         "Fighting", "Darkness", "Metal", "Colorless"];
            for t in &types {
                v.push(if a.energy_type.contains(t) { 1.0 } else { 0.0 });
            }
        } else {
            v.extend_from_slice(&[0.0; 10]);
        }
    }

    /// Encode as compact vector for neural network (original, ~22 features)
    pub fn to_vector(&self) -> Vec<f32> {
        let mut vec = Vec::new();
        
        // Turn info
        vec.push(self.turn as f32 / 20.0); // Normalize
        
        // Deck sizes
        vec.push(self.player_deck_size as f32 / 60.0);
        vec.push(self.opponent_deck_size as f32 / 60.0);
        
        // Hand sizes
        vec.push(self.player_hand_size as f32 / 10.0);
        vec.push(self.opponent_hand_size as f32 / 10.0);
        
        // Prizes
        vec.push(self.player_prizes as f32 / 6.0);
        vec.push(self.opponent_prizes as f32 / 6.0);
        
        // Active Pokemon (player)
        if let Some(active) = &self.player_active {
            vec.push(active.hp as f32 / 400.0);
            vec.push(active.damage as f32 / 400.0);
            vec.push(active.energy_count as f32 / 10.0);
            vec.push(if active.has_tool { 1.0 } else { 0.0 });
            vec.push(if active.is_ex { 1.0 } else { 0.0 });
        } else {
            vec.extend_from_slice(&[0.0; 6]);
        }
        
        // Active Pokemon (opponent)
        if let Some(active) = &self.opponent_active {
            vec.push(active.hp as f32 / 400.0);
            vec.push(active.damage as f32 / 400.0);
            vec.push(active.energy_count as f32 / 10.0);
            vec.push(if active.has_tool { 1.0 } else { 0.0 });
            vec.push(if active.is_ex { 1.0 } else { 0.0 });
        } else {
            vec.extend_from_slice(&[0.0; 6]);
        }
        
        // Bench counts
        let player_bench_hp: f32 = self.player_bench.iter()
            .filter_map(|p| p.as_ref())
            .map(|p| p.hp as f32)
            .sum::<f32>() / 400.0;
        vec.push(player_bench_hp);
        
        let opponent_bench_hp: f32 = self.opponent_bench.iter()
            .filter_map(|p| p.as_ref())
            .map(|p| p.hp as f32)
            .sum::<f32>() / 400.0;
        vec.push(opponent_bench_hp);
        
        // Can attack
        vec.push(if self.can_attack { 1.0 } else { 0.0 });
        vec.push(if self.attack_locked { 1.0 } else { 0.0 });
        
        vec
    }
}

/// Generate observation vector directly from GameState, avoiding intermediate
/// Observation struct allocations (Strings, Vecs). This is the hot-path version
/// for RL training.
pub fn vector_from_state(state: &GameState, player: PlayerId) -> Vec<f32> {
    let mut v = Vec::with_capacity(Observation::vector_dim_extended());
    let opponent = player.opponent();
    let player_state = &state.players[player.0];
    let opponent_state = &state.players[opponent.0];

    // Turn/phase info (10)
    v.push(state.turn.turn_number as f32 / 20.0);
    let phase_idx = match state.turn.phase {
        crate::state::Phase::Setup => 0,
        crate::state::Phase::Mulligan => 1,
        crate::state::Phase::Draw => 2,
        crate::state::Phase::Energy => 3,
        crate::state::Phase::Play => 4,
        crate::state::Phase::Retreat => 5,
        crate::state::Phase::Attack => 6,
        crate::state::Phase::End => 7,
    };
    for i in 0..8 {
        v.push(if i == phase_idx { 1.0 } else { 0.0 });
    }
    let is_active = state.turn.active_player == player
        && !matches!(state.turn.phase, crate::state::Phase::Setup | crate::state::Phase::Mulligan);
    v.push(if is_active { 1.0 } else { 0.0 });

    // Deck/hand/prize sizes (6)
    v.push(player_state.deck.len() as f32 / 60.0);
    v.push(opponent_state.deck.len() as f32 / 60.0);
    v.push(player_state.hand.len() as f32 / 10.0);
    v.push(opponent_state.hand.len() as f32 / 10.0);
    v.push(player_state.prizes.len() as f32 / 6.0);
    v.push(opponent_state.prizes.len() as f32 / 6.0);

    // Active Pokemon: player + opponent (16)
    push_active_direct(&mut v, state, player_state.active.as_ref());
    push_active_direct(&mut v, state, opponent_state.active.as_ref());

    // Bench: player + opponent (60)
    push_bench_direct(&mut v, state, &player_state.bench, 5);
    push_bench_direct(&mut v, state, &opponent_state.bench, 5);

    // Energy on active (20)
    push_energy_direct(&mut v, state, player_state.active.as_ref());
    push_energy_direct(&mut v, state, opponent_state.active.as_ref());

    // Bench energy (4)
    let p_bench_energy: usize = player_state.bench.iter()
        .filter_map(|s| s.as_ref()).map(|s| s.energies.len()).sum();
    v.push(p_bench_energy as f32 / 10.0);
    let p_bench_count = player_state.bench.iter().filter(|s| s.is_some()).count().max(1);
    v.push((p_bench_energy as f32 / p_bench_count as f32) / 10.0);
    let o_bench_energy: usize = opponent_state.bench.iter()
        .filter_map(|s| s.as_ref()).map(|s| s.energies.len()).sum();
    v.push(o_bench_energy as f32 / 10.0);
    let o_bench_count = opponent_state.bench.iter().filter(|s| s.is_some()).count().max(1);
    v.push((o_bench_energy as f32 / o_bench_count as f32) / 10.0);

    // Hand composition placeholders (8)
    v.extend_from_slice(&[0.0; 8]);

    // Stadium info (3)
    v.push(if player_state.stadium.is_some() { 1.0 } else { 0.0 });
    v.push(if opponent_state.stadium.is_some() { 1.0 } else { 0.0 });
    v.push(0.0);

    // Action flags (6)
    let can_attack = state.turn.active_player == player
        && matches!(state.turn.phase, crate::state::Phase::Play | crate::state::Phase::Attack);
    v.push(if can_attack { 1.0 } else { 0.0 });
    v.push(if state.turn.attack_locked { 1.0 } else { 0.0 });
    v.push(0.0); // can_retreat placeholder
    v.push(if matches!(state.turn.phase, crate::state::Phase::Play) { 1.0 } else { 0.0 });
    v.push(0.0); // energy_attached placeholder
    v.push(0.0); // supporter_used placeholder

    // Discard/lost zone sizes (4)
    v.push(player_state.discard.len() as f32 / 60.0);
    v.push(opponent_state.discard.len() as f32 / 60.0);
    v.push(player_state.lost_zone.len() as f32 / 10.0);
    v.push(opponent_state.lost_zone.len() as f32 / 10.0);

    // Padding (3)
    v.extend_from_slice(&[0.0; 3]);

    debug_assert_eq!(v.len(), Observation::vector_dim_extended());
    v
}

fn push_active_direct(v: &mut Vec<f32>, state: &GameState, slot: Option<&crate::state::PokemonSlot>) {
    if let Some(s) = slot.and_then(|s| s.top_card().and_then(|id| state.get_card_def(id))) {
        let hp = s.hp.unwrap_or(0).saturating_sub(
            slot.unwrap().damage
        );
        v.push(hp as f32 / 400.0);
        v.push(s.hp.unwrap_or(0) as f32 / 400.0);
        v.push(slot.unwrap().damage as f32 / 400.0);
        v.push(slot.unwrap().energies.len() as f32 / 10.0);
        v.push(if slot.unwrap().tool.is_some() { 1.0 } else { 0.0 });
        v.push(if s.is_ex() { 1.0 } else { 0.0 });
        v.push(if s.is_v() { 1.0 } else { 0.0 });
        v.push(s.attacks.len() as f32 / 4.0);
    } else {
        v.extend_from_slice(&[0.0; 8]);
    }
}

fn push_bench_direct(v: &mut Vec<f32>, state: &GameState, bench: &[Option<crate::state::PokemonSlot>], max_slots: usize) {
    for i in 0..max_slots {
        if let Some(s) = bench.get(i).and_then(|s| s.as_ref()) {
            if let Some(card) = s.top_card().and_then(|id| state.get_card_def(id)) {
                let hp = card.hp.unwrap_or(0).saturating_sub(s.damage);
                v.push(1.0);
                v.push(hp as f32 / 400.0);
                v.push(s.damage as f32 / 400.0);
                v.push(s.energies.len() as f32 / 10.0);
                v.push(if card.is_ex() { 1.0 } else { 0.0 });
                let stage_val = match card.stage {
                    Some(crate::card::Stage::Basic) => 0.0,
                    Some(crate::card::Stage::Stage1) => 0.5,
                    Some(crate::card::Stage::Stage2) => 1.0,
                    _ => 0.0,
                };
                v.push(stage_val);
            } else {
                v.extend_from_slice(&[0.0; 6]);
            }
        } else {
            v.extend_from_slice(&[0.0; 6]);
        }
    }
}

fn push_energy_direct(v: &mut Vec<f32>, _state: &GameState, slot: Option<&crate::state::PokemonSlot>) {
    if let Some(s) = slot {
        v.push(s.energies.len() as f32 / 10.0);
        // Encode Pokemon's own energy type as a one-hot-like signal
        if let Some(card) = s.top_card().and_then(|id| _state.get_card_def(id)) {
            let etype_idx = card.energy_type.map(|e| energy_type_index(e)).unwrap_or(8);
            for i in 0..9 {
                v.push(if i == etype_idx { 1.0 } else { 0.0 });
            }
        } else {
            v.extend_from_slice(&[0.0; 9]);
        }
    } else {
        v.extend_from_slice(&[0.0; 10]);
    }
}

/// Map EnergyType to a fixed index (no allocation).
fn energy_type_index(et: crate::card::EnergyType) -> usize {
    match et {
        crate::card::EnergyType::Lightning => 0,
        crate::card::EnergyType::Fire => 1,
        crate::card::EnergyType::Water => 2,
        crate::card::EnergyType::Grass => 3,
        crate::card::EnergyType::Psychic => 4,
        crate::card::EnergyType::Fighting => 5,
        crate::card::EnergyType::Darkness => 6,
        crate::card::EnergyType::Metal => 7,
        crate::card::EnergyType::Dragon => 8,
        crate::card::EnergyType::Colorless => 8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Engine;
    use crate::deck::MatchConfig;

    #[test]
    fn test_observation_creation() {
        let engine = Engine::new(MatchConfig::default(), 42);
        let obs = Observation::from_state(engine.state(), PlayerId(0));
        
        assert_eq!(obs.player_id, PlayerId(0));
        assert_eq!(obs.turn, 1);
    }

    #[test]
    fn test_observation_vector() {
        let engine = Engine::new(MatchConfig::default(), 42);
        let obs = Observation::from_state(engine.state(), PlayerId(0));
        let vec = obs.to_vector();

        assert!(!vec.is_empty());
        assert!(vec.iter().all(|&v| v.is_finite()));
    }

    #[test]
    fn test_vector_from_state_dimension() {
        let engine = Engine::new(MatchConfig::default(), 43);
        let vec = vector_from_state(engine.state(), PlayerId(0));
        assert_eq!(vec.len(), Observation::vector_dim_extended());
    }

    #[test]
    fn test_vector_from_state_all_finite() {
        let engine = Engine::new(MatchConfig::default(), 43);
        let vec = vector_from_state(engine.state(), PlayerId(0));
        assert!(vec.iter().all(|&v| v.is_finite()), "All observation values should be finite");
        assert!(vec.iter().any(|&v| v != 0.0), "Observation should not be all zeros");
    }

    #[test]
    fn test_vector_from_state_has_deck_info() {
        let engine = Engine::new(MatchConfig::default(), 43);
        let vec = vector_from_state(engine.state(), PlayerId(0));
        // Deck sizes at indices 10,11 (after turn/phase info)
        assert!(vec[10] > 0.0, "Player deck size should be non-zero");
        assert!(vec[11] > 0.0, "Opponent deck size should be non-zero");
    }

    #[test]
    fn test_observation_for_both_players() {
        let engine = Engine::new(MatchConfig::default(), 43);
        let v0 = vector_from_state(engine.state(), PlayerId(0));
        let v1 = vector_from_state(engine.state(), PlayerId(1));
        // Both should produce valid vectors
        assert_eq!(v0.len(), v1.len());
        // Deck sizes should be symmetric (swapped between players)
        assert_eq!(v0[10], v1[11]); // player_deck vs opponent_deck
        assert_eq!(v0[11], v1[10]);
    }
}