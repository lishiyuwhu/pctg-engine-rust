"""PTCG Gymnasium Environment for Reinforcement Learning.

Provides a standard gym.Env wrapper around the PTCG Rust engine.
The agent plays as Player 0; the opponent is auto-played by a bot.
"""

from .env import PTCGEnv
from .config import MatchConfig, DeckPreset
from .opponent import RandomOpponent, HeuristicOpponent
from .render import format_game_state

__all__ = [
    "PTCGEnv",
    "MatchConfig",
    "DeckPreset",
    "RandomOpponent",
    "HeuristicOpponent",
    "format_game_state",
]
