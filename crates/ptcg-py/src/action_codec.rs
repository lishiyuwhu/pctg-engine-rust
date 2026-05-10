//! Action encoding/decoding for RL interface.
//!
//! Maps the engine's combinatorial Action enum to a flat integer space
//! with action masking, suitable for Gym's Discrete action space.

use ptcg_core::action::Action;
use ptcg_core::state::{CardInstanceId, SlotRef};
use std::collections::HashMap;

/// Maximum number of flat actions in the encoded space.
pub const MAX_ACTION_SPACE: usize = 1024;

/// Sorted legal actions with their flat integer encoding.
pub struct EncodedActions {
    /// Sorted list of legal actions in canonical order.
    pub actions: Vec<Action>,
    /// Action mask: mask[i] == true if index i is a legal action.
    pub mask: Vec<bool>,
}

/// Encodes/decodes between Action enum and flat integer indices.
pub struct ActionCodec {
    max_action_space: usize,
}

impl ActionCodec {
    pub fn new(max_action_space: usize) -> Self {
        Self { max_action_space }
    }

    /// Encode legal actions into sorted order with a boolean mask.
    pub fn encode(&self, legal_actions: &[Action]) -> EncodedActions {
        let mut sorted: Vec<Action> = legal_actions.to_vec();
        sorted.sort_by_key(|a| Self::action_sort_key(a));

        let mut mask = vec![false; self.max_action_space];
        for i in 0..sorted.len().min(self.max_action_space) {
            mask[i] = true;
        }

        EncodedActions {
            actions: sorted,
            mask,
        }
    }

    /// Decode a flat integer index back to an Action.
    /// Returns None if the index is out of bounds or not legal.
    pub fn decode(&self, encoded: &EncodedActions, action_index: usize) -> Option<Action> {
        if action_index >= encoded.actions.len() {
            return None;
        }
        Some(encoded.actions[action_index].clone())
    }

    /// Compute canonical sort key for an Action.
    ///
    /// Sort priority (variant ordinal):
    ///   0: EndTurn
    ///   1: Pass
    ///   2: MulliganDraw { draw: true }
    ///   3: MulliganDraw { draw: false }
    ///   4: SetupChooseActive { card }
    ///   5: SetupBenchBasics { cards }
    ///   6: PlayBasicToBench { card }
    ///   7: Evolve { card, target }
    ///   8: AttachEnergy { card, target }
    ///   9: AttachTool { card, target }
    ///  10: PlayTrainer { card, choices }
    ///  11: PlayStadium { card, choices }
    ///  12: UseAbility { source, ability_index, choices }
    ///  13: Retreat { target, discard }
    ///  14: Attack { attack_index, choices }
    fn action_sort_key(action: &Action) -> (u8, u64, u64, u64) {
        match action {
            Action::EndTurn => (0, 0, 0, 0),
            Action::Pass => (1, 0, 0, 0),
            Action::MulliganDraw { draw: true } => (2, 0, 0, 0),
            Action::MulliganDraw { draw: false } => (3, 0, 0, 0),
            Action::SetupChooseActive { card } => (4, card.0 as u64, 0, 0),
            Action::SetupBenchBasics { cards } => {
                let first = cards.first().map(|c| c.0 as u64).unwrap_or(0);
                (5, first, cards.len() as u64, 0)
            }
            Action::PlayBasicToBench { card } => (6, card.0 as u64, 0, 0),
            Action::Evolve { card, target } => {
                let target_key = Self::slot_ref_key(target);
                (7, card.0 as u64, target_key, 0)
            }
            Action::AttachEnergy { card, target } => {
                let target_key = Self::slot_ref_key(target);
                (8, card.0 as u64, target_key, 0)
            }
            Action::AttachTool { card, target } => {
                let target_key = Self::slot_ref_key(target);
                (9, card.0 as u64, target_key, 0)
            }
            Action::PlayTrainer { card, choices: _ } => (10, card.0 as u64, 0, 0),
            Action::PlayStadium { card, choices: _ } => (11, card.0 as u64, 0, 0),
            Action::UseAbility {
                source,
                ability_index,
                choices: _,
            } => {
                let source_key = Self::slot_ref_key(source);
                (12, source_key, *ability_index as u64, 0)
            }
            Action::Retreat { target, discard: _ } => {
                let target_key = Self::slot_ref_key(target);
                (13, target_key, 0, 0)
            }
            Action::Attack {
                attack_index,
                choices: _,
            } => (14, *attack_index as u64, 0, 0),
        }
    }

    fn slot_ref_key(slot: &SlotRef) -> u64 {
        match slot {
            SlotRef::Active => 0,
            SlotRef::Bench(i) => 1 + *i as u64,
        }
    }

    /// Convert an Action to a Python-friendly HashMap.
    pub fn action_to_dict(&self, action: &Action) -> HashMap<String, String> {
        let mut d = HashMap::new();
        match action {
            Action::EndTurn => {
                d.insert("type".into(), "EndTurn".into());
            }
            Action::Pass => {
                d.insert("type".into(), "Pass".into());
            }
            Action::MulliganDraw { draw } => {
                d.insert("type".into(), "MulliganDraw".into());
                d.insert("draw".into(), draw.to_string());
            }
            Action::SetupChooseActive { card } => {
                d.insert("type".into(), "SetupChooseActive".into());
                d.insert("card_id".into(), card.0.to_string());
            }
            Action::SetupBenchBasics { cards } => {
                d.insert("type".into(), "SetupBenchBasics".into());
                d.insert(
                    "card_ids".into(),
                    format!(
                        "[{}]",
                        cards
                            .iter()
                            .map(|c| c.0.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    ),
                );
            }
            Action::PlayBasicToBench { card } => {
                d.insert("type".into(), "PlayBasicToBench".into());
                d.insert("card_id".into(), card.0.to_string());
            }
            Action::Evolve { card, target } => {
                d.insert("type".into(), "Evolve".into());
                d.insert("card_id".into(), card.0.to_string());
                d.insert("target".into(), format_slot_ref(target));
            }
            Action::AttachEnergy { card, target } => {
                d.insert("type".into(), "AttachEnergy".into());
                d.insert("card_id".into(), card.0.to_string());
                d.insert("target".into(), format_slot_ref(target));
            }
            Action::AttachTool { card, target } => {
                d.insert("type".into(), "AttachTool".into());
                d.insert("card_id".into(), card.0.to_string());
                d.insert("target".into(), format_slot_ref(target));
            }
            Action::PlayTrainer { card, choices: _ } => {
                d.insert("type".into(), "PlayTrainer".into());
                d.insert("card_id".into(), card.0.to_string());
            }
            Action::PlayStadium { card, choices: _ } => {
                d.insert("type".into(), "PlayStadium".into());
                d.insert("card_id".into(), card.0.to_string());
            }
            Action::UseAbility {
                source,
                ability_index,
                choices: _,
            } => {
                d.insert("type".into(), "UseAbility".into());
                d.insert("source".into(), format_slot_ref(source));
                d.insert("ability_index".into(), ability_index.to_string());
            }
            Action::Retreat { target, discard } => {
                d.insert("type".into(), "Retreat".into());
                d.insert("target".into(), format_slot_ref(target));
                d.insert("discard_count".into(), discard.len().to_string());
            }
            Action::Attack {
                attack_index,
                choices: _,
            } => {
                d.insert("type".into(), "Attack".into());
                d.insert("attack_index".into(), attack_index.to_string());
            }
        }
        d
    }
}

fn format_slot_ref(slot: &SlotRef) -> String {
    match slot {
        SlotRef::Active => "active".into(),
        SlotRef::Bench(i) => format!("bench_{}", i),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ptcg_core::action::Choices;

    #[test]
    fn test_encode_empty() {
        let codec = ActionCodec::new(16);
        let encoded = codec.encode(&[]);
        assert!(encoded.actions.is_empty());
        assert_eq!(encoded.mask.len(), 16);
        assert!(encoded.mask.iter().all(|&v| !v));
    }

    #[test]
    fn test_encode_sort_order() {
        let codec = ActionCodec::new(16);
        let actions = vec![
            Action::Attack {
                attack_index: 0,
                choices: Choices::new(),
            },
            Action::EndTurn,
            Action::Pass,
        ];
        let encoded = codec.encode(&actions);
        assert_eq!(encoded.actions.len(), 3);
        // Sorted: EndTurn(0), Pass(1), Attack(14)
        assert!(matches!(encoded.actions[0], Action::EndTurn));
        assert!(matches!(encoded.actions[1], Action::Pass));
        assert!(matches!(encoded.actions[2], Action::Attack { .. }));
        assert!(encoded.mask[0]);
        assert!(encoded.mask[1]);
        assert!(encoded.mask[2]);
        assert!(!encoded.mask[3]);
    }

    #[test]
    fn test_decode_roundtrip() {
        let codec = ActionCodec::new(64);
        let actions = vec![
            Action::EndTurn,
            Action::AttachEnergy {
                card: CardInstanceId(5),
                target: SlotRef::Active,
            },
            Action::Retreat {
                target: SlotRef::Bench(2),
                discard: vec![],
            },
        ];
        let encoded = codec.encode(&actions);

        for i in 0..actions.len() {
            let decoded = codec.decode(&encoded, i);
            assert!(decoded.is_some());
        }

        // Out of bounds
        assert!(codec.decode(&encoded, 5).is_none());
    }

    #[test]
    fn test_action_to_dict() {
        let codec = ActionCodec::new(64);

        let dict = codec.action_to_dict(&Action::EndTurn);
        assert_eq!(dict.get("type").unwrap(), "EndTurn");

        let dict = codec.action_to_dict(&Action::AttachEnergy {
            card: CardInstanceId(10),
            target: SlotRef::Active,
        });
        assert_eq!(dict.get("type").unwrap(), "AttachEnergy");
        assert_eq!(dict.get("card_id").unwrap(), "10");
        assert_eq!(dict.get("target").unwrap(), "active");

        let dict = codec.action_to_dict(&Action::Retreat {
            target: SlotRef::Bench(3),
            discard: vec![CardInstanceId(1), CardInstanceId(2)],
        });
        assert_eq!(dict.get("type").unwrap(), "Retreat");
        assert_eq!(dict.get("target").unwrap(), "bench_3");
        assert_eq!(dict.get("discard_count").unwrap(), "2");
    }

    #[test]
    fn test_sort_key_ordering() {
        // Verify EndTurn < Pass < Attack
        let a = ActionCodec::action_sort_key(&Action::EndTurn);
        let b = ActionCodec::action_sort_key(&Action::Pass);
        let c = ActionCodec::action_sort_key(&Action::Attack {
            attack_index: 0,
            choices: Choices::new(),
        });
        assert!(a < b);
        assert!(b < c);

        // Cards with lower IDs sort first
        let d = ActionCodec::action_sort_key(&Action::PlayBasicToBench {
            card: CardInstanceId(3),
        });
        let e = ActionCodec::action_sort_key(&Action::PlayBasicToBench {
            card: CardInstanceId(7),
        });
        assert!(d < e);
    }

    #[test]
    fn test_mask_bounds() {
        // With max=8, only first 8 legal actions get mask=true
        let codec = ActionCodec::new(8);
        let mut actions = Vec::new();
        for i in 0..20 {
            actions.push(Action::PlayBasicToBench {
                card: CardInstanceId(i),
            });
        }
        let encoded = codec.encode(&actions);
        assert_eq!(encoded.actions.len(), 20);
        assert_eq!(encoded.mask.len(), 8);
        // First 8 are true, rest are false
        let true_count = encoded.mask.iter().filter(|&&v| v).count();
        assert_eq!(true_count, 8);
    }
}
