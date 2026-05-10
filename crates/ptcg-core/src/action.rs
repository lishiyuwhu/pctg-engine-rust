//! Action space definitions

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::state::{CardInstanceId, PlayerId, SlotRef};

/// Action types in PTCG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    // Setup phase actions
    SetupChooseActive { card: CardInstanceId },
    SetupBenchBasics { cards: Vec<CardInstanceId> },
    MulliganDraw { draw: bool },
    
    // Play phase actions
    PlayBasicToBench { card: CardInstanceId },
    Evolve { card: CardInstanceId, target: SlotRef },
    AttachEnergy { card: CardInstanceId, target: SlotRef },
    AttachTool { card: CardInstanceId, target: SlotRef },
    PlayTrainer { card: CardInstanceId, choices: Choices },
    PlayStadium { card: CardInstanceId, choices: Choices },
    UseAbility { source: SlotRef, ability_index: u8, choices: Choices },
    Retreat { target: SlotRef, discard: Vec<CardInstanceId> },
    Attack { attack_index: u8, choices: Choices },
    EndTurn,
    
    // Special actions
    Pass,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::SetupChooseActive { card } => write!(f, "SetupChooseActive({})", card),
            Action::SetupBenchBasics { cards } => write!(f, "SetupBenchBasics({} cards)", cards.len()),
            Action::MulliganDraw { draw } => write!(f, "MulliganDraw(draw={})", draw),
            Action::PlayBasicToBench { card } => write!(f, "PlayBasicToBench({})", card),
            Action::Evolve { card, target } => write!(f, "Evolve({}, {:?})", card, target),
            Action::AttachEnergy { card, target } => write!(f, "AttachEnergy({}, {:?})", card, target),
            Action::AttachTool { card, target } => write!(f, "AttachTool({}, {:?})", card, target),
            Action::PlayTrainer { card, choices } => write!(f, "PlayTrainer({}, {:?})", card, choices),
            Action::PlayStadium { card, choices: _ } => write!(f, "PlayStadium({})", card),
            Action::UseAbility { source, ability_index, choices } => {
                write!(f, "UseAbility({:?}, {}, {:?})", source, ability_index, choices)
            }
            Action::Retreat { target, discard } => write!(f, "Retreat({:?}, {} cards)", target, discard.len()),
            Action::Attack { attack_index, choices } => write!(f, "Attack({}, {:?})", attack_index, choices),
            Action::EndTurn => write!(f, "EndTurn"),
            Action::Pass => write!(f, "Pass"),
        }
    }
}

/// Choice parameters for actions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Choices {
    pub selected_cards: Vec<CardInstanceId>,
    pub selected_slots: Vec<SlotRef>,
    pub selected_count: Option<usize>,
    pub mode: Option<u8>,
}

impl Choices {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cards(mut self, cards: Vec<CardInstanceId>) -> Self {
        self.selected_cards = cards;
        self
    }

    pub fn with_slots(mut self, slots: Vec<SlotRef>) -> Self {
        self.selected_slots = slots;
        self
    }

    pub fn with_count(mut self, count: usize) -> Self {
        self.selected_count = Some(count);
        self
    }

    pub fn with_mode(mut self, mode: u8) -> Self {
        self.mode = Some(mode);
        self
    }
}

/// Logged action for replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggedAction {
    pub turn: u16,
    pub player: PlayerId,
    pub action: Action,
    pub resulting_state_hash: u64,
}

impl LoggedAction {
    pub fn new(turn: u16, player: PlayerId, action: Action, state_hash: u64) -> Self {
        Self {
            turn,
            player,
            action,
            resulting_state_hash: state_hash,
        }
    }
}

/// Attack index wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttackIndex(pub u8);

impl AttackIndex {
    pub fn new(index: u8) -> Self {
        Self(index)
    }
}

impl fmt::Display for AttackIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Attack{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choices() {
        let choices = Choices::new()
            .with_cards(vec![CardInstanceId(1), CardInstanceId(2)])
            .with_count(2);
        
        assert_eq!(choices.selected_cards.len(), 2);
        assert_eq!(choices.selected_count, Some(2));
    }

    #[test]
    fn test_action_display() {
        let action = Action::EndTurn;
        assert_eq!(format!("{}", action), "EndTurn");
    }
}