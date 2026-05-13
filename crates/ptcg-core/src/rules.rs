//! Rule validation and legality checks

use crate::card::{CardDef, CardType, EnergyType, Stage};
use crate::state::{GameState, PlayerId, PokemonSlot, SlotRef};
use crate::action::{Action, Choices};
use crate::error::{EngineError, Result};
use crate::MAX_BENCH_SIZE;

/// Rule validator for checking action legality
#[derive(Debug, Clone)]
pub struct RuleValidator;

impl RuleValidator {
    pub fn new() -> Self {
        Self
    }

    /// Check if an action is legal for the given player
    pub fn is_legal(&self, state: &GameState, player: PlayerId, action: &Action) -> Result<()> {
        match action {
            Action::SetupChooseActive { card } => {
                self.validate_setup_choose_active(state, player, *card)
            }
            Action::SetupBenchBasics { cards } => {
                self.validate_setup_bench_basics(state, player, cards)
            }
            Action::MulliganDraw { draw: _ } => {
                self.validate_mulligan(state, player)
            }
            Action::PlayBasicToBench { card } => {
                self.validate_play_basic(state, player, *card)
            }
            Action::Evolve { card, target } => {
                self.validate_evolve(state, player, *card, *target)
            }
            Action::AttachEnergy { card, target } => {
                self.validate_attach_energy(state, player, *card, *target)
            }
            Action::AttachTool { card, target } => {
                self.validate_attach_tool(state, player, *card, *target)
            }
            Action::PlayTrainer { card, choices } => {
                self.validate_play_trainer(state, player, *card, choices)
            }
            Action::PlayStadium { card, choices } => {
                self.validate_play_stadium(state, player, *card, choices)
            }
            Action::UseAbility { source, ability_index, choices } => {
                self.validate_use_ability(state, player, *source, *ability_index, choices)
            }
            Action::Retreat { target, discard } => {
                self.validate_retreat(state, player, *target, discard)
            }
            Action::Attack { attack_index, choices } => {
                self.validate_attack(state, player, *attack_index, choices)
            }
            Action::EndTurn => Ok(()),
            Action::Pass => Ok(()),
        }
    }

    fn validate_setup_choose_active(&self, state: &GameState, player: PlayerId, card: crate::state::CardInstanceId) -> Result<()> {
        let player_state = &state.players[player.0];
        
        // Card must be in hand
        if !player_state.hand.contains(&card) {
            return Err(EngineError::InvalidAction("Card not in hand".into()));
        }
        
        // Card must be a basic Pokemon
        let card_def = state.get_card_def(card)
            .ok_or_else(|| EngineError::InvalidCard("Unknown card".into()))?;
        
        if !card_def.is_pokemon() {
            return Err(EngineError::InvalidAction("Not a Pokemon".into()));
        }
        
        if card_def.stage != Some(Stage::Basic) {
            return Err(EngineError::InvalidAction("Not a basic Pokemon".into()));
        }
        
        // Must not have active already
        if player_state.active.is_some() {
            return Err(EngineError::InvalidAction("Already have active Pokemon".into()));
        }
        
        Ok(())
    }

    fn validate_setup_bench_basics(&self, state: &GameState, player: PlayerId, cards: &[crate::state::CardInstanceId]) -> Result<()> {
        let player_state = &state.players[player.0];
        
        // Check bench capacity
        if cards.len() > MAX_BENCH_SIZE {
            return Err(EngineError::InvalidAction("Too many bench Pokemon".into()));
        }
        
        // Each card must be a basic Pokemon in hand
        for card in cards {
            if !player_state.hand.contains(card) {
                return Err(EngineError::InvalidAction("Card not in hand".into()));
            }
            
            let card_def = state.get_card_def(*card)
                .ok_or_else(|| EngineError::InvalidCard("Unknown card".into()))?;
            
            if !card_def.is_pokemon() || card_def.stage != Some(Stage::Basic) {
                return Err(EngineError::InvalidAction("Not a basic Pokemon".into()));
            }
        }
        
        Ok(())
    }

    fn validate_mulligan(&self, state: &GameState, player: PlayerId) -> Result<()> {
        // Valid during Setup or Mulligan phase
        if state.turn.phase != crate::state::Phase::Mulligan
            && state.turn.phase != crate::state::Phase::Setup
        {
            return Err(EngineError::InvalidAction("Not in setup or mulligan phase".into()));
        }
        
        // Check if player has a valid starting hand
        let player_state = &state.players[player.0];
        let has_basic = player_state.hand.iter().any(|id| {
            state.get_card_def(*id)
                .map(|def| def.is_pokemon() && def.stage == Some(Stage::Basic))
                .unwrap_or(false)
        });
        
        if has_basic {
            return Err(EngineError::InvalidAction("Hand has basic Pokemon, cannot mulligan".into()));
        }
        
        Ok(())
    }

    fn validate_play_basic(&self, state: &GameState, player: PlayerId, card: crate::state::CardInstanceId) -> Result<()> {
        let player_state = &state.players[player.0];
        
        // Card must be in hand
        if !player_state.hand.contains(&card) {
            return Err(EngineError::InvalidAction("Card not in hand".into()));
        }
        
        // Card must be a basic Pokemon
        let card_def = state.get_card_def(card)
            .ok_or_else(|| EngineError::InvalidCard("Unknown card".into()))?;
        
        if !card_def.is_pokemon() || card_def.stage != Some(Stage::Basic) {
            return Err(EngineError::InvalidAction("Not a basic Pokemon".into()));
        }
        
        // Must have bench space
        if player_state.bench_count() >= MAX_BENCH_SIZE {
            return Err(EngineError::InvalidAction("Bench is full".into()));
        }
        
        Ok(())
    }

    fn validate_evolve(&self, state: &GameState, player: PlayerId, card: crate::state::CardInstanceId, target: SlotRef) -> Result<()> {
        let player_state = &state.players[player.0];
        
        // Card must be in hand
        if !player_state.hand.contains(&card) {
            return Err(EngineError::InvalidAction("Card not in hand".into()));
        }
        
        // Target must be a Pokemon slot
        let target_slot = player_state.get_slot(target)
            .ok_or_else(|| EngineError::InvalidTarget("Invalid slot".into()))?;
        
        // Card must be an evolution of the target
        let card_def = state.get_card_def(card)
            .ok_or_else(|| EngineError::InvalidCard("Unknown card".into()))?;
        
        let target_def = target_slot.base_card()
            .and_then(|id| state.get_card_def(id))
            .ok_or_else(|| EngineError::CannotEvolve("Unknown Pokemon".into()))?;
        
        if !card_def.can_be_evolved_from(target_def) {
            return Err(EngineError::CannotEvolve("Cannot evolve from this Pokemon".into()));
        }
        
        // Cannot evolve same turn (except with Rare Candy - handled separately)
        if target_slot.turn_put_in_play == state.turn.turn_number {
            return Err(EngineError::CannotEvolve("Cannot evolve same turn as put in play".into()));
        }
        
        Ok(())
    }

    fn validate_attach_energy(&self, state: &GameState, player: PlayerId, card: crate::state::CardInstanceId, target: SlotRef) -> Result<()> {
        let player_state = &state.players[player.0];
        
        // Card must be in hand and be energy
        if !player_state.hand.contains(&card) {
            return Err(EngineError::InvalidAction("Card not in hand".into()));
        }
        
        let card_def = state.get_card_def(card)
            .ok_or_else(|| EngineError::InvalidCard("Unknown card".into()))?;
        
        if !card_def.is_basic_energy() && !card_def.is_special_energy() {
            return Err(EngineError::InvalidAction("Not an energy card".into()));
        }
        
        // Target must be a Pokemon slot owned by player
        let target_slot = player_state.get_slot(target)
            .ok_or_else(|| EngineError::InvalidTarget("Invalid slot".into()))?;
        
        if target_slot.is_empty() {
            return Err(EngineError::InvalidTarget("Slot is empty".into()));
        }
        
        Ok(())
    }

    fn validate_attach_tool(&self, state: &GameState, player: PlayerId, card: crate::state::CardInstanceId, target: SlotRef) -> Result<()> {
        let player_state = &state.players[player.0];
        
        // Card must be in hand and be a tool
        if !player_state.hand.contains(&card) {
            return Err(EngineError::InvalidAction("Card not in hand".into()));
        }
        
        let card_def = state.get_card_def(card)
            .ok_or_else(|| EngineError::InvalidCard("Unknown card".into()))?;
        
        if card_def.card_type != CardType::Tool {
            return Err(EngineError::InvalidAction("Not a tool card".into()));
        }
        
        // Target must be a Pokemon slot owned by player without tool
        let target_slot = player_state.get_slot(target)
            .ok_or_else(|| EngineError::InvalidTarget("Invalid slot".into()))?;
        
        if target_slot.is_empty() {
            return Err(EngineError::InvalidTarget("Slot is empty".into()));
        }
        
        if target_slot.tool.is_some() {
            return Err(EngineError::InvalidAction("Pokemon already has a tool".into()));
        }
        
        Ok(())
    }

    fn validate_play_trainer(&self, state: &GameState, player: PlayerId, card: crate::state::CardInstanceId, _choices: &Choices) -> Result<()> {
        let player_state = &state.players[player.0];
        
        // Card must be in hand and be a trainer
        if !player_state.hand.contains(&card) {
            return Err(EngineError::InvalidAction("Card not in hand".into()));
        }
        
        let card_def = state.get_card_def(card)
            .ok_or_else(|| EngineError::InvalidCard("Unknown card".into()))?;
        
        if !card_def.is_trainer() {
            return Err(EngineError::InvalidAction("Not a trainer card".into()));
        }
        
        // Supporter check (only one per turn)
        if card_def.card_type == CardType::Supporter {
            let action_key = format!("supporter_{}", card_def.id);
            if player_state.turn_actions_used.contains(&action_key) {
                return Err(EngineError::InvalidAction("Already used a Supporter this turn".into()));
            }
        }
        
        Ok(())
    }

    fn validate_play_stadium(&self, state: &GameState, player: PlayerId, card: crate::state::CardInstanceId, _choices: &Choices) -> Result<()> {
        let player_state = &state.players[player.0];
        
        // Card must be in hand and be a stadium
        if !player_state.hand.contains(&card) {
            return Err(EngineError::InvalidAction("Card not in hand".into()));
        }
        
        let card_def = state.get_card_def(card)
            .ok_or_else(|| EngineError::InvalidCard("Unknown card".into()))?;
        
        if card_def.card_type != CardType::Stadium {
            return Err(EngineError::InvalidAction("Not a stadium card".into()));
        }
        
        // Stadium can only be played once per turn
        let action_key = "play_stadium";
        if player_state.turn_actions_used.contains(action_key) {
            return Err(EngineError::InvalidAction("Already played a stadium this turn".into()));
        }
        
        Ok(())
    }

    fn validate_use_ability(&self, state: &GameState, player: PlayerId, source: SlotRef, _ability_index: u8, _choices: &Choices) -> Result<()> {
        let player_state = &state.players[player.0];
        
        let slot = player_state.get_slot(source)
            .ok_or_else(|| EngineError::InvalidTarget("Invalid slot".into()))?;
        
        if slot.is_empty() {
            return Err(EngineError::InvalidTarget("Slot is empty".into()));
        }
        
        // Check if ability was already used this turn
        if slot.used_ability_this_turn {
            return Err(EngineError::InvalidAction("Ability already used this turn".into()));
        }
        
        Ok(())
    }

    fn validate_retreat(&self, state: &GameState, player: PlayerId, target: SlotRef, _discard: &[crate::state::CardInstanceId]) -> Result<()> {
        let player_state = &state.players[player.0];
        
        // Must have active Pokemon
        let active = player_state.active.as_ref()
            .ok_or_else(|| EngineError::CannotRetreat("No active Pokemon".into()))?;
        
        if active.is_empty() {
            return Err(EngineError::CannotRetreat("Active slot is empty".into()));
        }
        
        // Target must be a bench slot with a Pokemon
        match target {
            SlotRef::Bench(i) => {
                if i >= MAX_BENCH_SIZE {
                    return Err(EngineError::InvalidTarget("Invalid bench index".into()));
                }
                let bench_slot = player_state.bench[i].as_ref()
                    .ok_or_else(|| EngineError::CannotRetreat("Bench slot is empty".into()))?;
                if bench_slot.is_empty() {
                    return Err(EngineError::CannotRetreat("Bench slot is empty".into()));
                }
            }
            SlotRef::Active => {
                return Err(EngineError::CannotRetreat("Cannot retreat to active".into()));
            }
        }
        
        // Check retreat cost and energy (simplified - actual implementation needs energy tracking)
        // For now, retreat is allowed if player has enough energy
        
        Ok(())
    }

    fn validate_attack(&self, state: &GameState, player: PlayerId, attack_index: u8, _choices: &Choices) -> Result<()> {
        let player_state = &state.players[player.0];
        
        // Must have active Pokemon
        let active = player_state.active.as_ref()
            .ok_or_else(|| EngineError::CannotAttack("No active Pokemon".into()))?;
        
        if active.is_empty() {
            return Err(EngineError::CannotAttack("Active slot is empty".into()));
        }
        
        // Check if attack is locked
        if state.turn.attack_locked {
            return Err(EngineError::CannotAttack("Attack is locked".into()));
        }
        
        // Get Pokemon definition
        let top_card = active.top_card()
            .and_then(|id| state.get_card_def(id))
            .ok_or_else(|| EngineError::CannotAttack("Unknown Pokemon".into()))?;
        
        // Check attack index is valid
        if attack_index as usize >= top_card.attacks.len() {
            return Err(EngineError::CannotAttack("Invalid attack index".into()));
        }
        
        // Check energy cost (simplified - actual implementation needs full energy tracking)
        let attack = &top_card.attacks[attack_index as usize];
        
        // Check for status effects preventing attack
        if active.status.asleep || active.status.paralyzed {
            return Err(EngineError::CannotAttack("Pokemon is asleep or paralyzed".into()));
        }

        // First-turn attack restriction: player going first cannot attack on turn 1
        if state.turn.turn_number == 1 && state.turn.first_player == player {
            return Err(EngineError::CannotAttack("First player cannot attack on turn 1".into()));
        }

        Ok(())
    }

    /// Check if player has enough energy for an attack
    pub fn has_enough_energy(&self, state: &GameState, player: PlayerId, slot: SlotRef, cost: &[EnergyType]) -> bool {
        let player_state = &state.players[player.0];
        let pokemon_slot = match player_state.get_slot(slot) {
            Some(s) => s,
            None => return false,
        };
        
        // Count attached energy by type
        let mut energy_count: std::collections::HashMap<EnergyType, usize> = std::collections::HashMap::new();
        for energy_id in &pokemon_slot.energies {
            if let Some(card_def) = state.get_card_def(*energy_id) {
                if let Some(provides) = &card_def.provides_energy {
                    for e in provides {
                        *energy_count.entry(*e).or_insert(0) += 1;
                    }
                }
            }
        }
        
        // Check each cost requirement
        let mut remaining_cost: std::collections::HashMap<EnergyType, usize> = std::collections::HashMap::new();
        for e in cost {
            if *e == EnergyType::Colorless {
                // Colorless can be any energy
                continue;
            }
            *remaining_cost.entry(*e).or_insert(0) += 1;
        }
        
        // Try to satisfy with attached energy
        for (energy_type, &count) in &energy_count {
            // Subtract from colorless requirement
            if let Some(colorless) = remaining_cost.get_mut(&EnergyType::Colorless) {
                let used = (*colorless).min(count);
                *colorless -= used;
            }
            
            // Subtract from specific requirement
            if let Some(specific) = remaining_cost.get_mut(energy_type) {
                let used = (*specific).min(count);
                *specific -= used;
            }
        }
        
        // Check if all requirements are satisfied
        remaining_cost.values().all(|&c| c == 0)
    }
}

impl Default for RuleValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::presets::load_miraidon_charizard_cards;
    use crate::deck::templates;
    use crate::rng::GameRng;
    use crate::state::GameState;

    #[test]
    fn test_validate_setup() {
        let validator = RuleValidator::new();
        let mut state = GameState::new();
        state.card_registry = load_miraidon_charizard_cards();
        
        let miraidon = templates::miraidon_deck();
        let charizard = templates::charizard_pidgeot_deck();
        
        let mut rng = GameRng::new(42);
        state.setup_initial_state(&miraidon, &charizard, &mut rng);
        
        // Should have cards in hand
        let player = PlayerId(0);
        assert!(!state.players[player.0].hand.is_empty());
    }
}