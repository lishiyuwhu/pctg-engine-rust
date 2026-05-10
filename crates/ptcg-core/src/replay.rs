//! Game replay serialization

use serde::{Deserialize, Serialize};
use crate::state::{GameState, PlayerId};
use crate::action::LoggedAction;

/// Game replay for storage and playback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    pub seed: u64,
    pub player_deck_id: u32,
    pub opponent_deck_id: u32,
    pub winner: Option<PlayerId>,
    pub total_turns: u16,
    pub actions: Vec<LoggedAction>,
    pub final_state_hash: u64,
}

impl Replay {
    /// Create a replay from a finished game
    pub fn from_game(state: &GameState, seed: u64, player_deck_id: u32, opponent_deck_id: u32) -> Self {
        Self {
            seed,
            player_deck_id,
            opponent_deck_id,
            winner: state.winner,
            total_turns: state.turn.turn_number,
            actions: state.action_log.clone(),
            final_state_hash: state.state_hash(),
        }
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get summary
    pub fn summary(&self) -> String {
        format!(
            "Replay: Player {} vs {}, {} turns, winner: {:?}",
            self.player_deck_id,
            self.opponent_deck_id,
            self.total_turns,
            self.winner
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Engine;
    use crate::deck::MatchConfig;

    #[test]
    fn test_replay_creation() {
        let engine = Engine::new(MatchConfig::default(), 42);
        let replay = Replay::from_game(engine.state(), 42, 575720, 575716);
        
        assert_eq!(replay.seed, 42);
        assert_eq!(replay.player_deck_id, 575720);
    }

    #[test]
    fn test_replay_serialization() {
        let engine = Engine::new(MatchConfig::default(), 42);
        let replay = Replay::from_game(engine.state(), 42, 575720, 575716);
        
        let json = replay.to_json().unwrap();
        let loaded = Replay::from_json(&json).unwrap();
        
        assert_eq!(replay.seed, loaded.seed);
        assert_eq!(replay.winner, loaded.winner);
    }
}