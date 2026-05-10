//! Error types for the engine

use thiserror::Error;
use crate::state::CardInstanceId;
use crate::action::Action;

/// Engine error types
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Invalid action: {0}")]
    InvalidAction(String),
    
    #[error("Illegal action: {0}")]
    IllegalAction(String),
    
    #[error("Invalid card: {0}")]
    InvalidCard(String),
    
    #[error("Invalid target: {0}")]
    InvalidTarget(String),
    
    #[error("Not enough energy: need {needed}, have {available}")]
    NotEnoughEnergy { needed: String, available: String },
    
    #[error("Cannot evolve: {0}")]
    CannotEvolve(String),
    
    #[error("Cannot retreat: {0}")]
    CannotRetreat(String),
    
    #[error("Cannot attack: {0}")]
    CannotAttack(String),
    
    #[error("Game already over")]
    GameOver,
    
    #[error("Invalid game state: {0}")]
    InvalidState(String),
    
    #[error("Unknown card definition: {0}")]
    UnknownCard(String),
    
    #[error("Unsupported card: {0}")]
    UnsupportedCard(String),
    
    #[error("Invalid deck: {0}")]
    InvalidDeck(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for EngineError {
    fn from(err: serde_json::Error) -> Self {
        EngineError::Serialization(err.to_string())
    }
}

/// Result type alias
pub type Result<T> = std::result::Result<T, EngineError>;

/// Extension trait for action results
pub trait ActionResult<T> {
    fn illegal(self, message: impl Into<String>) -> Result<T>;
}

impl<T> ActionResult<T> for std::result::Result<T, EngineError> {
    fn illegal(self, message: impl Into<String>) -> Result<T> {
        self.map_err(|_| EngineError::IllegalAction(message.into()))
    }
}

impl<T> ActionResult<T> for Option<T> {
    fn illegal(self, message: impl Into<String>) -> Result<T> {
        self.ok_or_else(|| EngineError::IllegalAction(message.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EngineError::NotEnoughEnergy {
            needed: "RR".to_string(),
            available: "R".to_string(),
        };
        assert!(err.to_string().contains("Not enough energy"));
    }

    #[test]
    fn test_action_result() {
        let result: Result<i32> = Some(42).illegal("should not fail");
        assert_eq!(result.unwrap(), 42);
        
        let result: Result<i32> = None.illegal("should fail");
        assert!(result.is_err());
    }
}