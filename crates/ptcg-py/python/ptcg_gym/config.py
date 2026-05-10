"""Match configuration types for PTCG Gym environment."""

from dataclasses import dataclass, field
from enum import Enum


class DeckPreset(Enum):
    """Available deck presets."""
    MIRAIDON = "miraidon"
    CHARIZARD_PIDGEOT = "charizard_pidgeot"


class StartingPlayer(Enum):
    """Who goes first."""
    RANDOM = "random"
    PLAYER = "player"
    OPPONENT = "opponent"


@dataclass
class MatchConfig:
    """Configuration for a PTCG match."""
    player_deck: str = "miraidon"
    opponent_deck: str = "charizard_pidgeot"
    starting_player: StartingPlayer = StartingPlayer.RANDOM
    seed: int = 42
    max_turns: int = 50
    action_space_size: int = 1024
