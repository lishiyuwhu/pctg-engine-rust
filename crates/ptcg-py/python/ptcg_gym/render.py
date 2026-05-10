"""Text-based game state renderer for PTCG Gym."""

import json


def format_game_state(engine) -> str:
    """Format the current game state as a human-readable string.

    Args:
        engine: PyEngine instance from ptcg_py.

    Returns:
        Multi-line string showing turn, phase, active pokemon,
        bench, and counts for both players.
    """
    lines = []
    turn = engine.turn()
    phase = engine.phase()
    active_player = engine.active_player()

    lines.append(f"{'=' * 60}")
    lines.append(f"  Turn {turn}  |  Phase: {phase}  |  Active: Player {active_player}")
    lines.append(f"{'=' * 60}")

    for pid in [0, 1]:
        marker = " *" if pid == active_player and phase not in ("Setup", "Mulligan") else ""
        lines.append(f"\n  Player {pid}{marker}")
        lines.append(f"  {'-' * 30}")

        obs_json = engine.observe_dict(pid)
        try:
            obs = json.loads(obs_json)
        except json.JSONDecodeError:
            lines.append("    (observation unavailable)")
            continue

        lines.append(
            f"    Deck: {obs.get('player_deck_size', '?')}  "
            f"Hand: {obs.get('player_hand_size', '?')}  "
            f"Prizes: {obs.get('player_prizes', '?')}"
        )

        active = obs.get("player_active")
        if active:
            lines.append(
                f"    Active: HP={active['hp']}/{active['max_hp']}  "
                f"DMG={active['damage']}  "
                f"Energy={active['energy_count']}  "
                f"Stage={active.get('stage', '?')}"
            )
        else:
            lines.append("    Active: (none)")

        bench = obs.get("player_bench", [])
        bench_parts = []
        for b in bench:
            if b:
                bench_parts.append(
                    f"HP={b['hp']}/{b['max_hp']} DMG={b['damage']} E={b['energy_count']}"
                )
            else:
                bench_parts.append("Empty")
        lines.append(f"    Bench: [{', '.join(bench_parts)}]")

    lines.append(f"\n{'=' * 60}")
    return "\n".join(lines)
