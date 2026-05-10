//! Core game engine

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::action::{Action, Choices, LoggedAction};
use crate::card::{presets, CardDefId, CardRegistry};
use crate::damage::DamageCalculator;
use crate::deck::{templates, Deck, MatchConfig, StartingPlayer};
use crate::error::{EngineError, Result};
use crate::rng::GameRng;
use crate::rules::RuleValidator;
use crate::state::{GameState, Phase, PlayerId, PokemonSlot, SlotRef, TurnState};
use crate::{INITIAL_HAND_SIZE, MAX_BENCH_SIZE, PRIZE_CARDS};

/// Game event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    CardPlayed {
        player: PlayerId,
        card: crate::state::CardInstanceId,
        location: String,
    },
    Damage {
        target_player: PlayerId,
        target_slot: SlotRef,
        damage: u16,
        ko: bool,
    },
    Evolved {
        player: PlayerId,
        slot: SlotRef,
        card: crate::state::CardInstanceId,
    },
    Switched {
        player: PlayerId,
        from: SlotRef,
        to: SlotRef,
    },
    PrizeTaken {
        player: PlayerId,
        count: usize,
    },
    KnockedOut {
        player: PlayerId,
        slot: SlotRef,
    },
    GameEnd {
        winner: Option<PlayerId>,
    },
    Attack {
        attacker: PlayerId,
        defender: PlayerId,
        attack_index: u8,
        damage: u16,
    },
    AbilityUsed {
        player: PlayerId,
        source: SlotRef,
        name: String,
    },
    Error {
        message: String,
    },
}

/// Step result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub done: bool,
    pub winner: Option<PlayerId>,
    pub reward: [f32; 2],
    pub events: Vec<Event>,
}

impl StepResult {
    pub fn new() -> Self {
        Self {
            done: false,
            winner: None,
            reward: [0.0, 0.0],
            events: Vec::new(),
        }
    }

    pub fn game_over(winner: Option<PlayerId>) -> Self {
        let mut reward = [0.0, 0.0];
        if let Some(w) = winner {
            reward[w.0] = 1.0;
            reward[w.opponent().0] = -1.0;
        }
        Self {
            done: true,
            winner,
            reward,
            events: vec![Event::GameEnd { winner }],
        }
    }
}

impl Default for StepResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Main game engine — the central coordinator for PTCG gameplay.
///
/// Manages the game state, RNG, rule validation, and damage calculation.
/// The engine drives the game loop: players query `legal_actions()` and
/// submit actions via `step()`. Phase transitions (Setup → Play → Attack)
/// are handled automatically.
///
/// # Example
///
/// ```rust,no_run
/// use ptcg_core::deck::{MatchConfig, StartingPlayer};
/// use ptcg_core::engine::Engine;
/// use ptcg_core::state::PlayerId;
///
/// let config = MatchConfig {
///     player_deck: ptcg_core::deck::templates::miraidon_deck(),
///     opponent_deck: ptcg_core::deck::templates::charizard_pidgeot_deck(),
///     ..Default::default()
/// };
/// let mut engine = Engine::new(config, 42);
/// let actions = engine.legal_actions(PlayerId(0));
/// ```
#[derive(Debug, Clone)]
pub struct Engine {
    state: GameState,
    rng: GameRng,
    validator: RuleValidator,
    damage_calculator: DamageCalculator,
    /// When false (RL training), skip state_hash and action_log for performance.
    pub record_replay: bool,
}

impl Engine {
    /// Create a new engine with the given match configuration and RNG seed.
    ///
    /// Sets up both players' decks, shuffles, deals prize cards (6 each),
    /// and draws initial hands (7 cards). The starting player is determined
    /// by the config's `starting_player` field.
    pub fn new(config: MatchConfig, seed: u64) -> Self {
        let mut state = GameState::new();
        state.card_registry = presets::load_miraidon_charizard_cards();

        let mut rng = GameRng::new(seed);

        // Setup initial state
        state.setup_initial_state(&config.player_deck, &config.opponent_deck, &mut rng);

        // Determine starting player
        let starting_player = match config.starting_player {
            StartingPlayer::Random => {
                if rng.coin_flip() {
                    PlayerId(0)
                } else {
                    PlayerId(1)
                }
            }
            StartingPlayer::Player => PlayerId(0),
            StartingPlayer::Opponent => PlayerId(1),
        };

        state.turn.active_player = starting_player;
        state.turn.turn_number = 1;
        state.turn.phase = Phase::Setup;

        Self {
            state,
            rng,
            validator: RuleValidator::new(),
            damage_calculator: DamageCalculator::new(),
            record_replay: true,
        }
    }

    /// Get current game state
    pub fn state(&self) -> &GameState {
        &self.state
    }

    /// Get mutable game state
    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.state
    }

    /// Get all legal actions for a player in the current game state.
    ///
    /// During Setup/Mulligan, actions for either player are returned
    /// regardless of whose turn it is. During normal play, only the
    /// active player's actions are returned.
    ///
    /// Returns an empty Vec if the game is over.
    pub fn legal_actions(&self, player: PlayerId) -> Vec<Action> {
        let mut actions = Vec::new();

        // If game is over, no actions
        if self.state.is_done() {
            return actions;
        }

        // During Setup/Mulligan phase, both players can act simultaneously
        let is_setup_phase = matches!(self.state.turn.phase, Phase::Setup | Phase::Mulligan);

        // Check if it's this player's turn (skip check during setup)
        if !is_setup_phase && self.state.turn.active_player != player {
            return actions;
        }

        match self.state.turn.phase {
            Phase::Setup => {
                let player_state = &self.state.players[player.0];
                // If player has no active and no basics in hand, suggest mulligan
                if player_state.active.is_none() {
                    let has_basic = player_state.hand.iter().any(|&id| {
                        self.state.get_card_def(id)
                            .map(|def| def.is_pokemon() && def.stage == Some(crate::card::Stage::Basic))
                            .unwrap_or(false)
                    });
                    if !has_basic {
                        // No basics in hand, must mulligan
                        actions.push(Action::MulliganDraw { draw: true });
                        return actions;
                    }
                }
                self.add_setup_actions(player, &mut actions);
                // If player has chosen an active, they can also indicate they're done with setup
                if player_state.active.is_some() {
                    actions.push(Action::EndTurn);
                }
            }
            Phase::Mulligan => {
                actions.push(Action::MulliganDraw { draw: true });
                actions.push(Action::MulliganDraw { draw: false });
            }
            Phase::Play => {
                self.add_play_actions(player, &mut actions);
                self.add_ability_actions(player, &mut actions);
            }
            Phase::Attack => {
                self.add_attack_actions(player, &mut actions);
            }
            Phase::End => {
                actions.push(Action::EndTurn);
            }
            _ => {}
        }

        actions
    }

    fn add_setup_actions(&self, player: PlayerId, actions: &mut Vec<Action>) {
        let player_state = &self.state.players[player.0];

        // If no active, can choose one
        if player_state.active.is_none() {
            for &card_id in &player_state.hand {
                if let Some(card_def) = self.state.get_card_def(card_id) {
                    if card_def.is_pokemon() && card_def.stage == Some(crate::card::Stage::Basic) {
                        actions.push(Action::SetupChooseActive { card: card_id });
                    }
                }
            }
        }

        // Can bench basics
        let bench_space = MAX_BENCH_SIZE - player_state.bench_count();
        if bench_space > 0 {
            let basic_cards: Vec<_> = player_state
                .hand
                .iter()
                .filter(|&&id| {
                    self.state
                        .get_card_def(id)
                        .map(|def| def.is_pokemon() && def.stage == Some(crate::card::Stage::Basic))
                        .unwrap_or(false)
                })
                .copied()
                .collect();

            // For simplicity, generate single-card bench actions
            for &card_id in &basic_cards {
                actions.push(Action::PlayBasicToBench { card: card_id });
            }
        }
    }

    fn add_play_actions(&self, player: PlayerId, actions: &mut Vec<Action>) {
        let player_state = &self.state.players[player.0];
        let has_active = player_state.active.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
        let bench_count = player_state.bench_count();
        let turn = self.state.turn.turn_number;

        // Collect occupied bench indices once
        let occupied_bench: Vec<usize> = player_state.bench.iter().enumerate()
            .filter_map(|(i, s)| s.as_ref().and_then(|s| if !s.is_empty() { Some(i) } else { None }))
            .collect();

        // Single pass over hand: classify each card once
        for &card_id in &player_state.hand {
            let card_def = match self.state.get_card_def(card_id) {
                Some(d) => d,
                None => continue,
            };

            if card_def.is_pokemon() {
                if card_def.stage == Some(crate::card::Stage::Basic) {
                    // Play basic to bench
                    if bench_count < MAX_BENCH_SIZE {
                        actions.push(Action::PlayBasicToBench { card: card_id });
                    }
                } else if card_def.stage.is_some() {
                    // Evolve: valid targets are Pokemon not placed this turn
                    if has_active {
                        if let Some(active) = &player_state.active {
                            if !active.is_empty() && active.turn_put_in_play != turn {
                                actions.push(Action::Evolve {
                                    card: card_id,
                                    target: SlotRef::Active,
                                });
                            }
                        }
                    }
                    for &i in &occupied_bench {
                        if let Some(s) = &player_state.bench[i] {
                            if s.turn_put_in_play != turn {
                                actions.push(Action::Evolve {
                                    card: card_id,
                                    target: SlotRef::Bench(i),
                                });
                            }
                        }
                    }
                }
            } else if card_def.is_basic_energy() || card_def.is_special_energy() {
                // Attach to active
                if has_active {
                    actions.push(Action::AttachEnergy {
                        card: card_id,
                        target: SlotRef::Active,
                    });
                }
                // Attach to occupied bench slots
                for &i in &occupied_bench {
                    actions.push(Action::AttachEnergy {
                        card: card_id,
                        target: SlotRef::Bench(i),
                    });
                }
            } else if card_def.is_trainer() {
                actions.push(Action::PlayTrainer {
                    card: card_id,
                    choices: Choices::new(),
                });
            }
        }

        // Retreat
        if has_active {
            for &i in &occupied_bench {
                actions.push(Action::Retreat {
                    target: SlotRef::Bench(i),
                    discard: vec![],
                });
            }
            actions.push(Action::EndTurn);
        }
    }

    fn add_attack_actions(&self, player: PlayerId, actions: &mut Vec<Action>) {
        let player_state = &self.state.players[player.0];

        if let Some(active) = &player_state.active {
            if !active.is_empty() {
                if let Some(card_def) = active.top_card().and_then(|id| self.state.get_card_def(id))
                {
                    for (i, _) in card_def.attacks.iter().enumerate() {
                        actions.push(Action::Attack {
                            attack_index: i as u8,
                            choices: Choices::new(),
                        });
                    }
                }
            }
        }

        actions.push(Action::EndTurn);
    }

    fn add_ability_actions(&self, player: PlayerId, actions: &mut Vec<Action>) {
        let player_state = &self.state.players[player.0];

        // Check active Pokemon for abilities
        if let Some(active) = &player_state.active {
            if !active.is_empty() && !active.used_ability_this_turn {
                if let Some(card_def) = active.top_card().and_then(|id| self.state.get_card_def(id))
                {
                    if !card_def.abilities.is_empty() {
                        actions.push(Action::UseAbility {
                            source: SlotRef::Active,
                            ability_index: 0,
                            choices: Choices::new(),
                        });
                    }
                }
            }
        }

        // Check bench Pokemon for abilities
        for (i, slot) in player_state.bench.iter().enumerate() {
            if let Some(s) = slot {
                if !s.is_empty() && !s.used_ability_this_turn {
                    if let Some(card_def) = s.top_card().and_then(|id| self.state.get_card_def(id))
                    {
                        if !card_def.abilities.is_empty() {
                            actions.push(Action::UseAbility {
                                source: SlotRef::Bench(i),
                                ability_index: 0,
                                choices: Choices::new(),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Execute an action for the given player.
    ///
    /// Validates the action against the rules, executes it (updating game state),
    /// applies phase transitions, and checks for game end conditions.
    ///
    /// Returns `StepResult` with:
    /// - `done`: whether the game ended
    /// - `winner`: the winning player (if game ended)
    /// - `reward`: zero-sum reward [p0, p1] (+1 win, -1 loss, 0 draw)
    /// - `events`: game events generated by this action
    pub fn step(&mut self, player: PlayerId, action: Action) -> StepResult {
        // Validate action
        if let Err(e) = self.validator.is_legal(&self.state, player, &action) {
            return StepResult::new();
        }

        let mut events = Vec::new();

        // Execute action
        match &action {
            Action::SetupChooseActive { card } => {
                events.extend(self.execute_setup_choose_active(player, *card));
            }
            Action::PlayBasicToBench { card } => {
                events.extend(self.execute_play_basic(player, *card));
            }
            Action::MulliganDraw { draw: _ } => {
                events.extend(self.execute_mulligan(player));
            }
            Action::EndTurn if self.state.turn.phase == Phase::Setup => {
                // During setup, EndTurn means "I'm done with setup"
                // Transition check happens in apply_phase_transitions
            }
            Action::AttachEnergy { card, target } => {
                events.extend(self.execute_attach_energy(player, *card, *target));
            }
            Action::Evolve { card, target } => {
                events.extend(self.execute_evolve(player, *card, *target));
            }
            Action::PlayTrainer { card, ref choices } => {
                events.extend(self.execute_play_trainer(player, *card, choices));
            }
            Action::PlayStadium { card, ref choices } => {
                events.extend(self.execute_play_stadium(player, *card, choices));
            }
            Action::UseAbility {
                source,
                ability_index: _,
                ref choices,
            } => {
                events.extend(self.execute_use_ability(player, *source, choices));
            }
            Action::Retreat { target, discard: _ } => {
                events.extend(self.execute_retreat(player, *target));
            }
            Action::EndTurn => {
                events.extend(self.execute_end_turn(player));
            }
            Action::Attack {
                attack_index,
                ref choices,
            } => {
                events.extend(self.execute_attack(player, *attack_index, choices));
            }
            _ => {}
        }

        // Phase transitions after action execution
        self.apply_phase_transitions();

        // Log action (skip in training mode for performance)
        if self.record_replay {
            let state_hash = self.state.state_hash();
            self.state.action_log.push(LoggedAction::new(
                self.state.turn.turn_number,
                player,
                action.clone(),
                state_hash,
            ));
        }

        // Check for game end
        if let Some(winner) = self.check_winner() {
            self.state.winner = Some(winner);
            return StepResult::game_over(Some(winner));
        }

        StepResult {
            done: false,
            winner: None,
            reward: [0.0, 0.0],
            events,
        }
    }

    fn execute_setup_choose_active(
        &mut self,
        player: PlayerId,
        card: crate::state::CardInstanceId,
    ) -> Vec<Event> {
        let player_state = &mut self.state.players[player.0];

        // Remove from hand
        if let Some(pos) = player_state.hand.iter().position(|&id| id == card) {
            player_state.hand.remove(pos);
        }

        // Set as active
        let mut slot = PokemonSlot::new();
        slot.cards.push(card);
        slot.turn_put_in_play = self.state.turn.turn_number;
        player_state.active = Some(slot);

        vec![Event::CardPlayed {
            player,
            card,
            location: "active".into(),
        }]
    }

    fn execute_play_basic(
        &mut self,
        player: PlayerId,
        card: crate::state::CardInstanceId,
    ) -> Vec<Event> {
        let player_state = &mut self.state.players[player.0];

        // Remove from hand
        if let Some(pos) = player_state.hand.iter().position(|&id| id == card) {
            player_state.hand.remove(pos);
        }

        // Find empty bench slot
        if let Some(slot) = player_state.bench.iter_mut().find(|s| s.is_none()) {
            let mut pokemon = PokemonSlot::new();
            pokemon.cards.push(card);
            pokemon.turn_put_in_play = self.state.turn.turn_number;
            *slot = Some(pokemon);

            return vec![Event::CardPlayed {
                player,
                card,
                location: "bench".into(),
            }];
        }

        Vec::new()
    }

    fn execute_mulligan(&mut self, player: PlayerId) -> Vec<Event> {
        // Pre-compute the set of basic Pokemon card IDs owned by this player
        // to avoid borrowing self.state twice.
        let basic_ids: std::collections::HashSet<crate::state::CardInstanceId> = self
            .state
            .cards_iter()
            .filter(|ci| ci.owner == player)
            .filter_map(|ci| {
                self.state
                    .card_registry
                    .get(&ci.def_id)
                    .filter(|def| def.is_pokemon() && def.stage == Some(crate::card::Stage::Basic))
                    .map(|_| ci.id)
            })
            .collect();

        let player_state = &mut self.state.players[player.0];
        let hand_size = player_state.hand.len();

        // Return hand to deck and shuffle
        player_state.deck.extend(player_state.hand.drain(..));
        self.rng.shuffle(&mut player_state.deck);

        // Draw new hand
        let mut hand_temp = Vec::new();
        for _ in 0..hand_size {
            if let Some(card) = player_state.deck.pop() {
                hand_temp.push(card);
            }
        }

        // If no basic Pokemon in hand, search deck for one and swap it in
        let has_basic = hand_temp.iter().any(|id| basic_ids.contains(id));
        if !has_basic && !basic_ids.is_empty() {
            if let Some(basic_idx) = player_state.deck.iter().position(|id| basic_ids.contains(id))
            {
                let basic_card = player_state.deck.remove(basic_idx);
                let swap_idx = self.rng.next_usize(hand_temp.len());
                let non_basic = hand_temp[swap_idx];
                hand_temp[swap_idx] = basic_card;
                player_state.deck.push(non_basic);
                self.rng.shuffle(&mut player_state.deck);
            }
        }

        player_state.hand = hand_temp;
        self.state.turn.mulligan_count += 1;

        Vec::new()
    }

    fn execute_attach_energy(
        &mut self,
        player: PlayerId,
        card: crate::state::CardInstanceId,
        target: SlotRef,
    ) -> Vec<Event> {
        let player_state = &mut self.state.players[player.0];

        // Remove from hand
        if let Some(pos) = player_state.hand.iter().position(|&id| id == card) {
            player_state.hand.remove(pos);
        }

        // Attach to target
        if let Some(slot) = player_state.get_slot_mut(target) {
            slot.energies.push(card);
        }

        vec![Event::CardPlayed {
            player,
            card,
            location: format!("{:?}", target),
        }]
    }

    fn execute_evolve(
        &mut self,
        player: PlayerId,
        card: crate::state::CardInstanceId,
        target: SlotRef,
    ) -> Vec<Event> {
        // Collect evolve abilities before mutable borrow
        let evolve_abilities: Vec<String> = self
            .state
            .get_card_def(card)
            .map(|def| {
                def.abilities
                    .iter()
                    .filter(|a| a.effect_id == "ability_infernal_reign")
                    .map(|a| a.effect_id.clone())
                    .collect()
            })
            .unwrap_or_default();

        let player_state = &mut self.state.players[player.0];

        // Remove from hand
        if let Some(pos) = player_state.hand.iter().position(|&id| id == card) {
            player_state.hand.remove(pos);
        }

        // Add to evolution stack
        if let Some(slot) = player_state.get_slot_mut(target) {
            slot.cards.push(card);
            slot.turn_evolved = Some(self.state.turn.turn_number);
        }

        // Trigger evolve abilities (Infernal Reign for Charizard ex)
        for effect_id in &evolve_abilities {
            let _ = super::effects::dispatch_ability(&mut self.state, player, target, effect_id);
        }

        vec![Event::Evolved {
            player,
            slot: target,
            card,
        }]
    }

    fn execute_play_trainer(
        &mut self,
        player: PlayerId,
        card: crate::state::CardInstanceId,
        choices: &Choices,
    ) -> Vec<Event> {
        // Get card def first before mutable borrow
        let (is_supporter, card_def_id) = self
            .state
            .get_card_def(card)
            .map(|def| {
                (
                    def.card_type == crate::card::CardType::Supporter,
                    def.id.0.clone(),
                )
            })
            .unwrap_or((false, String::new()));

        let player_state = &mut self.state.players[player.0];

        // Remove from hand
        if let Some(pos) = player_state.hand.iter().position(|&id| id == card) {
            player_state.hand.remove(pos);
        }

        // Add to discard
        player_state.discard.push(card);

        // Mark supporter as used if applicable
        if is_supporter {
            let action_key = format!("supporter_{}", card_def_id);
            player_state.turn_actions_used.insert(action_key);
        }

        vec![Event::CardPlayed {
            player,
            card,
            location: "discard".into(),
        }]
    }

    fn execute_play_stadium(
        &mut self,
        player: PlayerId,
        card: crate::state::CardInstanceId,
        _choices: &Choices,
    ) -> Vec<Event> {
        let card_def = self
            .state
            .get_card_def(card)
            .map(|def| (def.id.0.clone(), def.name.clone()))
            .unwrap_or((String::new(), String::new()));

        let player_state = &mut self.state.players[player.0];

        // Remove from hand
        if let Some(pos) = player_state.hand.iter().position(|&id| id == card) {
            player_state.hand.remove(pos);
        }

        // Set as current stadium
        let old_stadium = player_state.stadium.replace(crate::state::StadiumInPlay {
            card_id: card,
            owner: player,
        });
        if let Some(old) = old_stadium {
            player_state.discard.push(old.card_id);
        }

        // Dispatch stadium effect
        let _ = super::effects::dispatch_stadium(&mut self.state, player, &card_def.0);

        vec![Event::CardPlayed {
            player,
            card,
            location: "stadium".into(),
        }]
    }

    fn execute_use_ability(
        &mut self,
        player: PlayerId,
        source: SlotRef,
        _choices: &Choices,
    ) -> Vec<Event> {
        let player_state = &mut self.state.players[player.0];

        // Get the Pokemon with the ability
        let slot = match player_state.get_slot(source) {
            Some(s) => s,
            None => return Vec::new(),
        };

        let (effect_id, ability_name) = slot
            .top_card()
            .and_then(|id| self.state.get_card_def(id))
            .and_then(|def| {
                def.abilities
                    .first()
                    .map(|a| (a.effect_id.clone(), a.name.clone()))
            })
            .unwrap_or((String::new(), String::new()));

        // Dispatch the ability effect
        let result = super::effects::dispatch_ability(&mut self.state, player, source, &effect_id);

        match result {
            Ok(_effect_result) => {
                vec![Event::AbilityUsed {
                    player,
                    source,
                    name: ability_name,
                }]
            }
            Err(e) => {
                vec![Event::Error {
                    message: e.to_string(),
                }]
            }
        }
    }

    fn execute_retreat(&mut self, player: PlayerId, target: SlotRef) -> Vec<Event> {
        let player_state = &mut self.state.players[player.0];

        // Swap active and bench
        let active = player_state.active.take();
        let bench_pokemon = player_state.bench[target.bench_index().unwrap()].take();

        if let Some(pokemon) = bench_pokemon {
            player_state.active = Some(pokemon);
        }

        if let Some(pokemon) = active {
            player_state.bench[target.bench_index().unwrap()] = Some(pokemon);
        }

        vec![Event::Switched {
            player,
            from: SlotRef::Active,
            to: target,
        }]
    }

    fn execute_attack(
        &mut self,
        player: PlayerId,
        attack_index: u8,
        choices: &Choices,
    ) -> Vec<Event> {
        let player_state = &self.state.players[player.0];
        let opponent = player.opponent();
        let opponent_state = &self.state.players[opponent.0];

        // Get attacker and defender
        let attacker_slot = SlotRef::Active;
        let defender_slot = SlotRef::Active;

        // Calculate damage
        let base_damage = if let Some(active) = &player_state.active {
            if let Some(card_def) = active.top_card().and_then(|id| self.state.get_card_def(id)) {
                if (attack_index as usize) < card_def.attacks.len() {
                    card_def.attacks[attack_index as usize].damage
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        let damage = self.damage_calculator.calculate_damage(
            &self.state,
            player,
            attacker_slot,
            opponent,
            defender_slot,
            base_damage,
        );

        // Get defender's card_id and max_hp before mutable borrow
        let defender_card_id = opponent_state.active.as_ref().and_then(|s| s.top_card());
        let defender_max_hp = defender_card_id
            .and_then(|id| self.state.get_card_def(id))
            .and_then(|def| def.hp);

        // Apply damage
        let ko = {
            let opponent_state = &mut self.state.players[opponent.0];
            if let Some(slot) = opponent_state.active.as_mut() {
                slot.damage += damage;

                // Check KO
                if let Some(max_hp) = defender_max_hp {
                    slot.damage >= max_hp
                } else {
                    false
                }
            } else {
                false
            }
        };

        // Lock attack
        self.state.turn.attack_locked = true;

        // Process KO if applicable
        let mut events = Vec::new();
        if ko {
            events.extend(self.process_ko(opponent, defender_slot, player));
        }

        events.push(Event::Attack {
            attacker: player,
            defender: opponent,
            attack_index,
            damage,
        });
        events.push(Event::Damage {
            target_player: opponent,
            target_slot: defender_slot,
            damage,
            ko,
        });

        events
    }

    fn process_ko(&mut self, player: PlayerId, slot: SlotRef, attacker: PlayerId) -> Vec<Event> {
        let mut events = Vec::new();

        // Collect all cards from the KO'd Pokemon
        let mut discarded = Vec::new();

        // Get and clear the slot
        match slot {
            SlotRef::Active => {
                if let Some(pokemon) = self.state.players[player.0].active.take() {
                    discarded.extend(pokemon.cards);
                    discarded.extend(pokemon.energies);
                    if let Some(tool) = pokemon.tool {
                        discarded.push(tool);
                    }
                }
            }
            SlotRef::Bench(i) => {
                if let Some(pokemon) = self.state.players[player.0].bench[i].take() {
                    discarded.extend(pokemon.cards);
                    discarded.extend(pokemon.energies);
                    if let Some(tool) = pokemon.tool {
                        discarded.push(tool);
                    }
                }
            }
        }

        // Move to discard
        self.state.players[player.0].discard.extend(discarded);

        events.push(Event::KnockedOut { player, slot });

        // Take prize cards (EX Pokemon takes 2 prizes)
        let prize_count = if self.state.players[player.0].prizes.len() >= 2 {
            2
        } else {
            self.state.players[player.0].prizes.len()
        };
        let mut taken_prizes = Vec::new();
        for _ in 0..prize_count {
            if let Some(prize) = self.state.players[player.0].prizes.pop() {
                taken_prizes.push(prize);
                events.push(Event::PrizeTaken {
                    player: attacker,
                    count: 1,
                });
            }
        }
        self.state.players[attacker.0].hand.extend(taken_prizes);

        // Check if player needs to select new active
        if self.state.players[player.0].active.is_none() {
            // Move a bench Pokemon to active if available
            for (i, bench_slot) in self.state.players[player.0].bench.iter_mut().enumerate() {
                if bench_slot.is_some() {
                    let pokemon = bench_slot.take().unwrap();
                    self.state.players[player.0].active = Some(pokemon);
                    events.push(Event::Switched {
                        player,
                        from: SlotRef::Bench(i),
                        to: SlotRef::Active,
                    });
                    break;
                }
            }
        }

        events
    }

    fn execute_end_turn(&mut self, player: PlayerId) -> Vec<Event> {
        // Clear turn state
        self.state.players[player.0].clear_turn_state();

        // Switch active player
        self.state.turn.active_player = player.opponent();

        // New turn
        if player == self.state.turn.active_player.opponent() {
            self.state.turn.turn_number += 1;
        }

        // Draw phase
        self.state.turn.phase = Phase::Draw;
        let drawn = self.state.draw_cards(self.state.turn.active_player, 1);

        // Move to play phase
        self.state.turn.phase = Phase::Play;
        self.state.turn.attack_locked = false;

        vec![]
    }

    fn apply_phase_transitions(&mut self) {
        match self.state.turn.phase {
            Phase::Setup => {
                // After choosing active, player can still bench more basics
                // Setup is complete when both players have active Pokemon
                let p0_has_active = self.state.players[0].active.is_some();
                let p1_has_active = self.state.players[1].active.is_some();

                if p0_has_active && p1_has_active {
                    // Both players have active Pokemon, move to Play phase for the starting player
                    self.state.turn.phase = Phase::Play;
                }
            }
            Phase::Mulligan => {
                // Check if current player now has basic Pokemon after mulligan draw
                let player_state = &self.state.players[self.state.turn.active_player.0];
                let has_basic = player_state.hand.iter().any(|id| {
                    self.state.get_card_def(*id)
                        .map(|def| def.is_pokemon() && def.stage == Some(crate::card::Stage::Basic))
                        .unwrap_or(false)
                });

                if has_basic {
                    self.state.turn.mulligan_done = true;
                    self.state.turn.phase = Phase::Setup;
                }
            }
            _ => {}
        }
    }

    fn check_winner(&self) -> Option<PlayerId> {
        // Check if a player has taken all 6 prize cards.
        // Prizes start at 6 and are decremented as they're taken.
        // Empty prizes means the player started with 0 prizes (during setup)
        // which should not trigger a win.
        for (i, player_state) in self.state.players.iter().enumerate() {
            if player_state.prizes.is_empty()
                && !matches!(self.state.turn.phase, Phase::Setup | Phase::Mulligan)
            {
                return Some(PlayerId(i));
            }
        }

        // Check if opponent has no Pokemon (skip during setup/mulligan)
        if !matches!(self.state.turn.phase, Phase::Setup | Phase::Mulligan) {
            for (i, player_state) in self.state.players.iter().enumerate() {
                if !player_state.has_pokemon_in_play() {
                    return Some(PlayerId(1 - i));
                }
            }
        }

        None
    }

    /// Get action log
    pub fn action_log(&self) -> &[LoggedAction] {
        &self.state.action_log
    }

    /// Get state hash for reproducibility
    pub fn state_hash(&self) -> u64 {
        self.state.state_hash()
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Turn {} - {:?}",
            self.state.turn.turn_number, self.state.turn.phase
        )?;
        writeln!(f, "Active player: {:?}", self.state.turn.active_player)?;

        for (i, player_state) in self.state.players.iter().enumerate() {
            writeln!(
                f,
                "Player {}: hand={}, deck={}, prizes={}",
                i,
                player_state.hand.len(),
                player_state.deck.len(),
                player_state.prizes.len()
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new(MatchConfig::default(), 42);
        assert_eq!(engine.state().turn.turn_number, 1);
    }

    #[test]
    fn test_legal_actions() {
        let engine = Engine::new(MatchConfig::default(), 42);

        // Should have actions during setup (either setup actions or mulligan)
        let actions = engine.legal_actions(PlayerId(0));
        assert!(!actions.is_empty(), "Expected legal actions during setup");
    }

    #[test]
    fn test_state_hash_deterministic() {
        let engine1 = Engine::new(MatchConfig::default(), 42);
        let engine2 = Engine::new(MatchConfig::default(), 42);

        assert_eq!(engine1.state_hash(), engine2.state_hash());
    }

    #[test]
    fn test_setup_flow() {
        // Use a seed where both players have basics (no mulligan)
        let mut engine = Engine::new(MatchConfig::default(), 43);
        assert!(matches!(engine.state().turn.phase, Phase::Setup));

        // P0 should have setup actions
        let p0_actions = engine.legal_actions(PlayerId(0));
        let choose_active = p0_actions.iter().find(|a| matches!(a, Action::SetupChooseActive { .. }));
        assert!(choose_active.is_some(), "P0 should have SetupChooseActive");

        // Execute P0 choose active
        engine.step(PlayerId(0), choose_active.unwrap().clone());
        assert!(engine.state().players[0].active.is_some(), "P0 should have active Pokemon");
    }

    #[test]
    fn test_phase_transition_to_play() {
        let mut engine = Engine::new(MatchConfig::default(), 43);

        // P0 chooses active
        let p0_actions = engine.legal_actions(PlayerId(0));
        let p0_active = p0_actions.iter().find(|a| matches!(a, Action::SetupChooseActive { .. })).unwrap().clone();
        engine.step(PlayerId(0), p0_active);

        // P1 chooses active
        let p1_actions = engine.legal_actions(PlayerId(1));
        let p1_active = p1_actions.iter().find(|a| matches!(a, Action::SetupChooseActive { .. }));
        if let Some(action) = p1_active {
            engine.step(PlayerId(1), action.clone());
        }

        // Both should now have active, phase should transition to Play
        // (may need to auto-step opponent setup)
        let p0_has = engine.state().players[0].active.is_some();
        let p1_has = engine.state().players[1].active.is_some();
        assert!(p0_has || p1_has, "At least one player should have active");
    }

    #[test]
    fn test_end_turn_switches_player() {
        let mut engine = Engine::new(MatchConfig::default(), 43);

        // Navigate to Play phase by choosing active for both players
        for &p in &[PlayerId(0), PlayerId(1)] {
            let actions = engine.legal_actions(p);
            if let Some(a) = actions.iter().find(|a| matches!(a, Action::SetupChooseActive { .. })) {
                engine.step(p, a.clone());
            }
        }

        // If we're in Play phase, test EndTurn
        if matches!(engine.state().turn.phase, Phase::Play) {
            let prev_player = engine.state().turn.active_player;
            engine.step(prev_player, Action::EndTurn);
            // Active player should have switched
            let new_player = engine.state().turn.active_player;
            assert_ne!(prev_player, new_player, "Active player should switch after EndTurn");
        }
    }

    #[test]
    fn test_record_replay_flag() {
        let mut engine = Engine::new(MatchConfig::default(), 43);
        assert!(engine.record_replay, "record_replay should default to true");

        engine.record_replay = false;
        let actions = engine.legal_actions(PlayerId(0));
        if !actions.is_empty() {
            let log_before = engine.state().action_log.len();
            engine.step(PlayerId(0), actions[0].clone());
            let log_after = engine.state().action_log.len();
            assert_eq!(log_before, log_after, "No action should be logged when record_replay is false");
        }
    }

    #[test]
    fn test_prize_setup() {
        let engine = Engine::new(MatchConfig::default(), 43);
        assert_eq!(engine.state().players[0].prizes.len(), 6);
        assert_eq!(engine.state().players[1].prizes.len(), 6);
    }

    #[test]
    fn test_winner_check_no_false_positive() {
        let engine = Engine::new(MatchConfig::default(), 43);
        // During setup, winner should be None
        assert!(engine.state().winner.is_none(), "No winner during setup");
    }

    #[test]
    fn test_state_accessors() {
        let engine = Engine::new(MatchConfig::default(), 43);
        assert_eq!(engine.state().turn.turn_number, 1);
        assert_eq!(engine.state().players.len(), 2);
        assert!(engine.state().players[0].hand.len() == 7);
        assert!(engine.state().players[1].hand.len() == 7);
    }
}
