//! Python bindings for PTCG Rust Engine
//!
//! Provides a Gymnasium-compatible RL interface via PyO3.

mod action_codec;

use action_codec::{ActionCodec, MAX_ACTION_SPACE};
use ptcg_core::{
    action::{Action, Choices},
    deck::{MatchConfig, StartingPlayer},
    engine::Engine,
    observe::Observation,
    state::{Phase, PlayerId, SlotRef},
};
use pyo3::prelude::*;
use std::collections::HashMap;

/// Create an engine with default Miraidon vs Charizard/Pidgeot match.
#[pyfunction]
fn create_engine(seed: u64) -> PyEngine {
    PyEngine::new(seed)
}

/// Create engine for Miraidon deck only (both sides Miraidon for mirror matches).
#[pyfunction]
fn create_engine_mirror(seed: u64) -> PyEngine {
    let config = MatchConfig {
        player_deck: ptcg_core::deck::templates::miraidon_deck(),
        opponent_deck: ptcg_core::deck::templates::miraidon_deck(),
        player_name: "Player".into(),
        opponent_name: "Opponent".into(),
        starting_player: StartingPlayer::Random,
    };
    let engine = Engine::new(config, seed);
    PyEngine {
        engine,
        codec: ActionCodec::new(MAX_ACTION_SPACE),
        obs_dim: 0,
    }
}

// ── PyEngine ──────────────────────────────────────────────────────────

#[pyclass]
struct PyEngine {
    engine: Engine,
    codec: ActionCodec,
    /// Cached observation dimension (computed once from to_vector_extended).
    obs_dim: usize,
}

impl PyEngine {
    fn new(seed: u64) -> Self {
        let config = MatchConfig {
            player_deck: ptcg_core::deck::templates::miraidon_deck(),
            opponent_deck: ptcg_core::deck::templates::charizard_pidgeot_deck(),
            player_name: "Player".into(),
            opponent_name: "Opponent".into(),
            starting_player: StartingPlayer::Random,
        };
        let mut engine = Engine::new(config, seed);
        engine.record_replay = false; // RL training mode: skip action log overhead
        let obs_dim = Observation::vector_dim_extended();
        Self {
            engine,
            codec: ActionCodec::new(MAX_ACTION_SPACE),
            obs_dim,
        }
    }
}

#[pymethods]
impl PyEngine {
    // ── Lifecycle ─────────────────────────────────────────────────

    /// Reset engine to a fresh game. Uses the same deck configuration.
    fn reset(&mut self, seed: Option<u64>) {
        let s = seed.unwrap_or(0);
        let config = MatchConfig {
            player_deck: ptcg_core::deck::templates::miraidon_deck(),
            opponent_deck: ptcg_core::deck::templates::charizard_pidgeot_deck(),
            player_name: "Player".into(),
            opponent_name: "Opponent".into(),
            starting_player: StartingPlayer::Random,
        };
        self.engine = Engine::new(config, s);
    }

    // ── Core RL interface ─────────────────────────────────────────

    /// Execute an action by flat integer index (0..max_action_space).
    /// Returns (obs_list, reward, done, winner, turn, phase, events_json)
    fn step(
        &mut self,
        player_id: usize,
        action_index: usize,
    ) -> PyResult<(Vec<f32>, f32, bool, Option<usize>, u16, String, String)> {
        let player = PlayerId(player_id);
        let legal = self.engine.legal_actions(player);

        // Handle empty legal actions (shouldn't happen during normal play)
        if legal.is_empty() {
            let obs = self.observe(player_id);
            return Ok((obs, 0.0, self.engine.state().is_done(), None, 0, "".into(), "[]".into()));
        }

        let (sorted_indices, _mask) = self.codec.encode_by_index(&legal);

        let action = match self.codec.decode_from_slice(&legal, &sorted_indices, action_index) {
            Some(a) => a,
            None => {
                let obs = self.observe(player_id);
                return Ok((
                    obs,
                    -0.1,
                    false,
                    None,
                    self.engine.state().turn.turn_number,
                    format!("{:?}", self.engine.state().turn.phase),
                    r#"[{"error": "invalid_action_index"}]"#.into(),
                ));
            }
        };

        let result = self.engine.step(player, action);

        let obs = self.observe(player_id);
        let reward = result.reward[player_id];
        let done = result.done;
        let winner = result.winner.map(|w| w.0);
        let turn = self.engine.state().turn.turn_number;
        let phase = format!("{:?}", self.engine.state().turn.phase);

        let events_json = serde_json::to_string(&result.events).unwrap_or_else(|_| "[]".into());

        Ok((obs, reward, done, winner, turn, phase, events_json))
    }

    /// Execute an action given as a JSON string.
    fn step_json(
        &mut self,
        player_id: usize,
        action_json: &str,
    ) -> PyResult<(Vec<f32>, f32, bool, Option<usize>, u16, String, String)> {
        let action = Self::parse_action_json(action_json);
        let player = PlayerId(player_id);
        let result = self.engine.step(player, action);

        let obs = self.observe(player_id);
        Ok((
            obs,
            result.reward[player_id],
            result.done,
            result.winner.map(|w| w.0),
            self.engine.state().turn.turn_number,
            format!("{:?}", self.engine.state().turn.phase),
            serde_json::to_string(&result.events).unwrap_or_else(|_| "[]".into()),
        ))
    }

    // ── Observation ───────────────────────────────────────────────

    /// Get extended observation vector for a player (float list).
    /// Uses direct state-to-vector path without intermediate Observation allocation.
    fn observe(&self, player_id: usize) -> Vec<f32> {
        ptcg_core::observe::vector_from_state(self.engine.state(), PlayerId(player_id))
    }

    /// Get observation as a structured Python dict (JSON string).
    fn observe_dict(&self, player_id: usize) -> String {
        let player = PlayerId(player_id);
        let obs = Observation::from_state(self.engine.state(), player);
        serde_json::to_string(&obs).unwrap_or_else(|_| "{}".into())
    }

    /// Dimension of the extended observation vector.
    fn observation_dim(&self) -> usize {
        self.obs_dim
    }

    // ── Action space ──────────────────────────────────────────────

    /// Get (legal_indices: Vec<usize>, action_mask: Vec<bool>).
    fn legal_actions_encoded(&self, player_id: usize) -> (Vec<usize>, Vec<bool>) {
        let player = PlayerId(player_id);
        let legal = self.engine.legal_actions(player);
        let encoded = self.codec.encode(&legal);

        let indices: Vec<usize> = (0..encoded.actions.len()).collect();
        (indices, encoded.mask)
    }

    /// Get legal actions as a list of JSON dict strings.
    fn legal_actions_dicts(&self, player_id: usize) -> Vec<String> {
        let player = PlayerId(player_id);
        let legal = self.engine.legal_actions(player);

        legal
            .iter()
            .map(|a| {
                let dict = self.codec.action_to_dict(a);
                serde_json::to_string(&dict).unwrap_or_else(|_| "{}".into())
            })
            .collect()
    }

    /// Get action mask as boolean list.
    fn action_mask(&self, player_id: usize) -> Vec<bool> {
        let player = PlayerId(player_id);
        let legal = self.engine.legal_actions(player);
        self.codec.encode(&legal).mask
    }

    /// Number of legal actions for a player.
    fn num_legal_actions(&self, player_id: usize) -> usize {
        let player = PlayerId(player_id);
        self.engine.legal_actions(player).len()
    }

    /// Max action space size.
    fn action_space_size(&self) -> usize {
        MAX_ACTION_SPACE
    }

    // ── State queries ─────────────────────────────────────────────

    fn is_done(&self) -> bool {
        self.engine.state().is_done()
    }

    fn winner(&self) -> Option<usize> {
        self.engine.state().winner.map(|w| w.0)
    }

    fn turn(&self) -> u16 {
        self.engine.state().turn.turn_number
    }

    fn phase(&self) -> String {
        format!("{:?}", self.engine.state().turn.phase)
    }

    /// Which players can currently act? (1 or 2 during setup/mulligan).
    fn acting_players(&self) -> Vec<usize> {
        if self.engine.state().is_done() {
            return vec![];
        }
        let is_multiactor = matches!(
            self.engine.state().turn.phase,
            Phase::Setup | Phase::Mulligan
        );
        if is_multiactor {
            vec![0, 1]
        } else {
            vec![self.engine.state().turn.active_player.0]
        }
    }

    fn active_player(&self) -> usize {
        self.engine.state().turn.active_player.0
    }

    fn is_setup_phase(&self) -> bool {
        matches!(
            self.engine.state().turn.phase,
            Phase::Setup | Phase::Mulligan
        )
    }

    // ── Rendering ─────────────────────────────────────────────────

    fn render_text(&self) -> String {
        format!("{}", self.engine)
    }
}

// ── JSON Action Parser ────────────────────────────────────────────────

impl PyEngine {
    fn parse_action_json(json: &str) -> Action {
        // Try to parse as structured JSON dict
        if let Ok(map) = serde_json::from_str::<HashMap<String, serde_json::Value>>(json) {
            let action_type = map
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("Pass");

            match action_type {
                "EndTurn" => return Action::EndTurn,
                "Pass" => return Action::Pass,
                "MulliganDraw" => {
                    let draw = map.get("draw").and_then(|v| v.as_bool()).unwrap_or(true);
                    return Action::MulliganDraw { draw };
                }
                "SetupChooseActive" => {
                    if let Some(card) = map.get("card_id").and_then(|v| v.as_u64()) {
                        return Action::SetupChooseActive {
                            card: ptcg_core::state::CardInstanceId(card as u32),
                        };
                    }
                }
                "SetupBenchBasics" => {
                    if let Some(cards_arr) = map.get("card_ids").and_then(|v| v.as_array()) {
                        let cards: Vec<ptcg_core::state::CardInstanceId> = cards_arr
                            .iter()
                            .filter_map(|v| v.as_u64().map(|n| ptcg_core::state::CardInstanceId(n as u32)))
                            .collect();
                        return Action::SetupBenchBasics { cards };
                    }
                }
                "PlayBasicToBench" => {
                    if let Some(card) = map.get("card_id").and_then(|v| v.as_u64()) {
                        return Action::PlayBasicToBench {
                            card: ptcg_core::state::CardInstanceId(card as u32),
                        };
                    }
                }
                "Evolve" => {
                    if let (Some(card), Some(target)) = (
                        map.get("card_id").and_then(|v| v.as_u64()),
                        map.get("target").and_then(|v| v.as_str()),
                    ) {
                        return Action::Evolve {
                            card: ptcg_core::state::CardInstanceId(card as u32),
                            target: parse_slot_ref(target),
                        };
                    }
                }
                "AttachEnergy" => {
                    if let (Some(card), Some(target)) = (
                        map.get("card_id").and_then(|v| v.as_u64()),
                        map.get("target").and_then(|v| v.as_str()),
                    ) {
                        return Action::AttachEnergy {
                            card: ptcg_core::state::CardInstanceId(card as u32),
                            target: parse_slot_ref(target),
                        };
                    }
                }
                "AttachTool" => {
                    if let (Some(card), Some(target)) = (
                        map.get("card_id").and_then(|v| v.as_u64()),
                        map.get("target").and_then(|v| v.as_str()),
                    ) {
                        return Action::AttachTool {
                            card: ptcg_core::state::CardInstanceId(card as u32),
                            target: parse_slot_ref(target),
                        };
                    }
                }
                "PlayTrainer" => {
                    if let Some(card) = map.get("card_id").and_then(|v| v.as_u64()) {
                        return Action::PlayTrainer {
                            card: ptcg_core::state::CardInstanceId(card as u32),
                            choices: parse_choices(&map),
                        };
                    }
                }
                "PlayStadium" => {
                    if let Some(card) = map.get("card_id").and_then(|v| v.as_u64()) {
                        return Action::PlayStadium {
                            card: ptcg_core::state::CardInstanceId(card as u32),
                            choices: parse_choices(&map),
                        };
                    }
                }
                "UseAbility" => {
                    if let (Some(source), Some(ability_index)) = (
                        map.get("source").and_then(|v| v.as_str()),
                        map.get("ability_index").and_then(|v| v.as_u64()),
                    ) {
                        return Action::UseAbility {
                            source: parse_slot_ref(source),
                            ability_index: ability_index as u8,
                            choices: parse_choices(&map),
                        };
                    }
                }
                "Retreat" => {
                    if let Some(target) = map.get("target").and_then(|v| v.as_str()) {
                        let discard = if let Some(arr) = map.get("discard").and_then(|v| v.as_array())
                        {
                            arr.iter()
                                .filter_map(|v| {
                                    v.as_u64().map(|n| ptcg_core::state::CardInstanceId(n as u32))
                                })
                                .collect()
                        } else {
                            vec![]
                        };
                        return Action::Retreat {
                            target: parse_slot_ref(target),
                            discard,
                        };
                    }
                }
                "Attack" => {
                    let attack_index = map
                        .get("attack_index")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u8;
                    return Action::Attack {
                        attack_index,
                        choices: parse_choices(&map),
                    };
                }
                _ => {}
            }
        }

        // Fallback: try as simple string (e.g. "EndTurn", "Pass")
        match json.trim_matches('"') {
            "EndTurn" => Action::EndTurn,
            "Pass" => Action::Pass,
            _ => Action::Pass,
        }
    }
}

fn parse_slot_ref(s: &str) -> SlotRef {
    if s == "active" {
        return SlotRef::Active;
    }
    if let Some(idx_str) = s.strip_prefix("bench_") {
        if let Ok(idx) = idx_str.parse::<usize>() {
            return SlotRef::Bench(idx);
        }
    }
    SlotRef::Active
}

fn parse_choices(map: &HashMap<String, serde_json::Value>) -> Choices {
    let mut choices = Choices::new();

    if let Some(arr) = map.get("selected_cards").and_then(|v| v.as_array()) {
        choices.selected_cards = arr
            .iter()
            .filter_map(|v| v.as_u64().map(|n| ptcg_core::state::CardInstanceId(n as u32)))
            .collect();
    }

    if let Some(arr) = map.get("selected_slots").and_then(|v| v.as_array()) {
        choices.selected_slots = arr.iter().filter_map(|v| v.as_str().map(parse_slot_ref)).collect();
    }

    if let Some(n) = map.get("selected_count").and_then(|v| v.as_u64()) {
        choices.selected_count = Some(n as usize);
    }

    if let Some(n) = map.get("mode").and_then(|v| v.as_u64()) {
        choices.mode = Some(n as u8);
    }

    choices
}

// ── Batch Runner ────────────────────────────────────────────────────

/// Run N games in parallel with random actions, returning statistics.
#[pyfunction]
fn run_batch(n_games: usize, seed: u64, threads: Option<usize>) -> PyResult<String> {
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use ptcg_core::deck::{MatchConfig, StartingPlayer};

    if let Some(n) = threads {
        if n > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(n)
                .build_global()
                .ok();
        }
    }

    let p0_wins = AtomicU64::new(0);
    let p1_wins = AtomicU64::new(0);
    let draws = AtomicU64::new(0);
    let total_turns = AtomicU64::new(0);
    let total_steps = AtomicU64::new(0);

    (0..n_games).into_par_iter().for_each(|i| {
        let game_seed = seed.wrapping_add(i as u64);
        let config = MatchConfig {
            player_deck: ptcg_core::deck::templates::miraidon_deck(),
            opponent_deck: ptcg_core::deck::templates::charizard_pidgeot_deck(),
            player_name: "P0".into(),
            opponent_name: "P1".into(),
            starting_player: StartingPlayer::Random,
        };
        let mut engine = Engine::new(config, game_seed);
        engine.record_replay = false;
        let miraidon = ptcg_core::strategy::MiraidonStrategy;
        let charizard = ptcg_core::strategy::CharizardStrategy;
        use ptcg_core::strategy::DeckStrategy;
        let mut steps = 0usize;
        let max_steps = 2000;

        while !engine.state().is_done() && steps < max_steps {
            let is_setup = matches!(
                engine.state().turn.phase,
                ptcg_core::state::Phase::Setup | ptcg_core::state::Phase::Mulligan
            );
            let players: Vec<PlayerId> = if is_setup {
                vec![PlayerId(0), PlayerId(1)]
            } else {
                vec![engine.state().turn.active_player]
            };
            for &p in &players {
                if engine.state().is_done() { break; }
                let actions = engine.legal_actions(p);
                if actions.is_empty() { continue; }
                let idx = if p.0 == 0 {
                    miraidon.select_action(&actions, engine.state(), p).unwrap_or(0)
                } else {
                    charizard.select_action(&actions, engine.state(), p).unwrap_or(0)
                };
                engine.step(p, actions[idx].clone());
                steps += 1;
            }
        }

        total_turns.fetch_add(engine.state().turn.turn_number as u64, Ordering::Relaxed);
        total_steps.fetch_add(steps as u64, Ordering::Relaxed);
        match engine.state().winner {
            Some(PlayerId(0)) => { p0_wins.fetch_add(1, Ordering::Relaxed); }
            Some(PlayerId(1)) => { p1_wins.fetch_add(1, Ordering::Relaxed); }
            _ => { draws.fetch_add(1, Ordering::Relaxed); }
        }
    });

    let n = n_games as f64;
    let result = serde_json::json!({
        "total_games": n_games,
        "player0_wins": p0_wins.load(Ordering::Relaxed),
        "player1_wins": p1_wins.load(Ordering::Relaxed),
        "draws": draws.load(Ordering::Relaxed),
        "avg_turns": total_turns.load(Ordering::Relaxed) as f64 / n,
        "avg_steps": total_steps.load(Ordering::Relaxed) as f64 / n,
    });

    Ok(serde_json::to_string(&result).unwrap_or_else(|_| "{}".into()))
}

// ── Python Module ─────────────────────────────────────────────────────

#[pymodule]
fn ptcg_py(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(create_engine, m)?)?;
    m.add_function(wrap_pyfunction!(create_engine_mirror, m)?)?;
    m.add_function(wrap_pyfunction!(run_batch, m)?)?;
    m.add_class::<PyEngine>()?;
    Ok(())
}
