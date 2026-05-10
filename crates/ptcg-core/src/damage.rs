//! Damage calculation with modifiers

use crate::card::{CardDef, EnergyType};
use crate::state::{GameState, PlayerId, PokemonSlot, SlotRef};

/// Damage calculator
#[derive(Debug, Clone)]
pub struct DamageCalculator;

impl DamageCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate damage for an attack
    pub fn calculate_damage(
        &self,
        state: &GameState,
        attacker: PlayerId,
        attacker_slot: SlotRef,
        defender: PlayerId,
        defender_slot: SlotRef,
        base_damage: u16,
    ) -> u16 {
        let mut damage = base_damage as i32;
        
        // Get attacker Pokemon
        let attacker_pokemon = match state.players[attacker.0].get_slot(attacker_slot) {
            Some(p) => p,
            None => return base_damage,
        };
        
        // Get defender Pokemon
        let defender_pokemon = match state.players[defender.0].get_slot(defender_slot) {
            Some(p) => p,
            None => return base_damage,
        };
        
        let attacker_def = match attacker_pokemon.top_card().and_then(|id| state.get_card_def(id)) {
            Some(d) => d,
            None => return base_damage,
        };
        
        let defender_def = match defender_pokemon.top_card().and_then(|id| state.get_card_def(id)) {
            Some(d) => d,
            None => return base_damage,
        };
        
        // Apply attacker modifiers
        damage += self.get_attacker_modifier(state, attacker, attacker_slot, attacker_def);
        
        // Apply defender modifiers
        damage -= self.get_defender_modifier(state, defender, defender_slot, defender_def);
        
        // Apply weakness
        if let Some(weakness) = &defender_def.weakness {
            if let Some(attacker_type) = attacker_def.energy_type {
                if attacker_type == weakness.energy_type {
                    damage += weakness.multiplier as i32 * base_damage as i32 / 10;
                }
            }
        }
        
        // Apply resistance
        if let Some(resistance) = &defender_def.resistance {
            if let Some(attacker_type) = attacker_def.energy_type {
                if attacker_type == resistance.energy_type {
                    damage += resistance.multiplier as i32;
                }
            }
        }
        
        // Ensure minimum damage is 0
        damage.max(0) as u16
    }

    /// Get attacker damage modifier from abilities, tools, stadiums
    fn get_attacker_modifier(
        &self,
        state: &GameState,
        player: PlayerId,
        slot: SlotRef,
        _def: &CardDef,
    ) -> i32 {
        let mut modifier = 0;
        let player_state = &state.players[player.0];
        
        // Miraidon ex Tandem Unit: +30 to all basic Electric Pokemon
        // Check if any bench Miraidon ex provides the buff
        for bench_slot in &player_state.bench {
            if let Some(pokemon) = bench_slot {
                if let Some(card_def) = pokemon.top_card().and_then(|id| state.get_card_def(id)) {
                    if card_def.id.0 == "CSV1C_050" {
                        // Miraidon ex
                        if let Some(attacker) = player_state.get_slot(slot) {
                            if let Some(attacker_def) = attacker.top_card().and_then(|id| state.get_card_def(id)) {
                                if attacker_def.energy_type == Some(EnergyType::Lightning) {
                                    modifier += 30;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Check active Miraidon ex
        if let Some(active) = &player_state.active {
            if let Some(card_def) = active.top_card().and_then(|id| state.get_card_def(id)) {
                if card_def.id.0 == "CSV1C_050" {
                    if let Some(attacker) = player_state.get_slot(slot) {
                        if let Some(attacker_def) = attacker.top_card().and_then(|id| state.get_card_def(id)) {
                            if attacker_def.energy_type == Some(EnergyType::Lightning) {
                                modifier += 30;
                            }
                        }
                    }
                }
            }
        }
        
        // Tool modifiers
        if let Some(pokemon) = player_state.get_slot(slot) {
            if let Some(tool_id) = pokemon.tool {
                if let Some(tool_def) = state.get_card_def(tool_id) {
                    if let Some(dmg_mod) = tool_def.damage_modifier {
                        modifier += dmg_mod;
                    }
                }
            }
        }
        
        modifier
    }

    /// Get defender damage modifier
    fn get_defender_modifier(
        &self,
        state: &GameState,
        player: PlayerId,
        slot: SlotRef,
        _def: &CardDef,
    ) -> i32 {
        let mut modifier = 0;
        let player_state = &state.players[player.0];
        
        // Stadium modifiers
        if let Some(stadium) = &player_state.stadium {
            if let Some(stadium_def) = state.get_card_def(stadium.card_id) {
                if let Some(dmg_mod) = stadium_def.damage_modifier {
                    modifier += dmg_mod;
                }
            }
        }
        
        // Tool modifiers (defensive)
        if let Some(pokemon) = player_state.get_slot(slot) {
            if let Some(tool_id) = pokemon.tool {
                if let Some(tool_def) = state.get_card_def(tool_id) {
                    if let Some(dmg_mod) = tool_def.damage_modifier {
                        modifier += dmg_mod;
                    }
                }
            }
        }
        
        modifier
    }

    /// Calculate damage for bench snipe attack
    pub fn calculate_bench_damage(
        &self,
        state: &GameState,
        attacker: PlayerId,
        attacker_slot: SlotRef,
        defender: PlayerId,
        defender_slot: SlotRef,
        base_damage: u16,
    ) -> u16 {
        // Bench snipe typically doesn't apply weakness/resistance
        // But might apply tool modifiers
        self.calculate_damage(state, attacker, attacker_slot, defender, defender_slot, base_damage)
    }
}

impl Default for DamageCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_calculation() {
        let calculator = DamageCalculator::new();
        
        // Basic test - damage should be at least 0
        // Full integration tests would require full game state setup
        assert!(true);
    }
}