//! PTCG Rust Engine Core
//! 
//! A minimal, high-performance Pokemon TCG game engine designed for RL training.
//! 
//! # Architecture
//! 
//! - `card`: Card definitions and static data
//! - `deck`: Deck building and validation
//! - `state`: Game state and player state
//! - `action`: Action space enumeration
//! - `engine`: Core game loop and state machine
//! - `rules`: Rule validation and legality checks
//! - `damage`: Damage calculation with modifiers
//! - `effects`: Card effect implementations
//! - `observe`: Observation encoding for RL
//! - `replay`: Game replay serialization
//! - `rng`: Deterministic random number generation

pub mod card;
pub mod deck;
pub mod state;
pub mod action;
pub mod engine;
pub mod rules;
pub mod damage;
pub mod effects;
pub mod observe;
pub mod replay;
pub mod rng;
pub mod error;

pub use card::{CardDef, CardDefId, CardType, EnergyType, Mechanic};
pub use deck::{Deck, DeckError, MatchConfig};
pub use state::{GameState, PlayerState, PokemonSlot, StatusFlags, CardInstance, CardInstanceId, PlayerId, SlotRef};
pub use action::{Action, Choices, AttackIndex};
pub use engine::{Engine, StepResult, Event};
pub use rules::RuleValidator;
pub use damage::DamageCalculator;
pub use observe::Observation;
pub use replay::Replay;
pub use rng::GameRng;
pub use error::{EngineError, Result};

/// Game constants
pub const MAX_BENCH_SIZE: usize = 5;
pub const INITIAL_HAND_SIZE: usize = 7;
pub const PRIZE_CARDS: usize = 6;
pub const MAX_DECK_SIZE: usize = 60;

/// Energy type constants
pub const ENERGY_COST_ANY: &str = "C";