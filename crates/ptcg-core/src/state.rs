//! Game state and player state definitions

use serde::{Deserialize, Serialize};
use std::fmt;
use std::collections::HashSet;

use crate::card::{CardDefId, CardRegistry, CardType, Stage};
use crate::deck::Deck;
use crate::rng::GameRng;
use crate::{MAX_BENCH_SIZE, INITIAL_HAND_SIZE, PRIZE_CARDS, MAX_DECK_SIZE};

/// Player identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub usize);

impl PlayerId {
    pub fn opponent(&self) -> Self {
        PlayerId(1 - self.0)
    }
}

impl fmt::Display for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player{}", self.0)
    }
}

/// Slot reference for targeting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotRef {
    Active,
    Bench(usize),
}

impl SlotRef {
    pub fn bench_index(&self) -> Option<usize> {
        match self {
            SlotRef::Active => None,
            SlotRef::Bench(i) => Some(*i),
        }
    }
}

/// Card instance identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CardInstanceId(pub u32);

impl CardInstanceId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl fmt::Display for CardInstanceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CardInstance({})", self.0)
    }
}

/// A card instance in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInstance {
    pub id: CardInstanceId,
    pub def_id: CardDefId,
    pub owner: PlayerId,
}

impl CardInstance {
    pub fn new(id: CardInstanceId, def_id: CardDefId, owner: PlayerId) -> Self {
        Self { id, def_id, owner }
    }
}

/// Status flags for Pokemon
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusFlags {
    pub poisoned: bool,
    pub burned: bool,
    pub paralyzed: bool,
    pub asleep: bool,
    pub confused: bool,
    pub cannot_retreat: bool,
}

impl StatusFlags {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn is_active(&self) -> bool {
        self.poisoned || self.burned || self.paralyzed || self.asleep || self.confused
    }
}

/// A Pokemon slot (active or bench position)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokemonSlot {
    pub cards: Vec<CardInstanceId>,  // Evolution stack (bottom to top)
    pub energies: Vec<CardInstanceId>,
    pub tool: Option<CardInstanceId>,
    pub damage: u16,
    pub status: StatusFlags,
    pub turn_put_in_play: u16,
    pub turn_evolved: Option<u16>,
    pub used_ability_this_turn: bool,
    pub vstar_power_used: bool,
}

impl PokemonSlot {
    pub fn new() -> Self {
        Self {
            cards: Vec::new(),
            energies: Vec::new(),
            tool: None,
            damage: 0,
            status: StatusFlags::new(),
            turn_put_in_play: 0,
            turn_evolved: None,
            used_ability_this_turn: false,
            vstar_power_used: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    pub fn top_card(&self) -> Option<CardInstanceId> {
        self.cards.last().copied()
    }

    pub fn base_card(&self) -> Option<CardInstanceId> {
        self.cards.first().copied()
    }

    pub fn hp(&self, max_hp: u16) -> i16 {
        max_hp as i16 - self.damage as i16
    }

    pub fn is_knocked_out(&self, max_hp: u16) -> bool {
        self.damage >= max_hp
    }

    pub fn clear(&mut self) {
        self.cards.clear();
        self.energies.clear();
        self.tool = None;
        self.damage = 0;
        self.status.clear();
        self.turn_evolved = None;
        self.used_ability_this_turn = false;
        self.vstar_power_used = false;
    }
}

/// Stadium card in play
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StadiumInPlay {
    pub card_id: CardInstanceId,
    pub owner: PlayerId,
}

/// Player state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub deck: Vec<CardInstanceId>,
    pub hand: Vec<CardInstanceId>,
    pub prizes: Vec<CardInstanceId>,
    pub discard: Vec<CardInstanceId>,
    pub lost_zone: Vec<CardInstanceId>,
    pub active: Option<PokemonSlot>,
    pub bench: Vec<Option<PokemonSlot>>,
    pub stadium: Option<StadiumInPlay>,
    pub turn_actions_used: HashSet<String>,
}

impl PlayerState {
    pub fn new() -> Self {
        Self {
            deck: Vec::new(),
            hand: Vec::new(),
            prizes: Vec::new(),
            discard: Vec::new(),
            lost_zone: Vec::new(),
            active: None,
            bench: vec![None; MAX_BENCH_SIZE],
            stadium: None,
            turn_actions_used: HashSet::new(),
        }
    }

    pub fn bench_count(&self) -> usize {
        self.bench.iter().filter(|s| s.is_some()).count()
    }

    pub fn total_pokemon_count(&self) -> usize {
        let active = if self.active.is_some() { 1 } else { 0 };
        active + self.bench_count()
    }

    pub fn has_pokemon_in_play(&self) -> bool {
        self.active.is_some() || self.bench.iter().any(|s| s.is_some())
    }

    pub fn clear_turn_state(&mut self) {
        self.turn_actions_used.clear();
        if let Some(active) = &mut self.active {
            active.used_ability_this_turn = false;
        }
        for bench in &mut self.bench {
            if let Some(slot) = bench {
                slot.used_ability_this_turn = false;
            }
        }
    }

    pub fn get_slot(&self, slot_ref: SlotRef) -> Option<&PokemonSlot> {
        match slot_ref {
            SlotRef::Active => self.active.as_ref(),
            SlotRef::Bench(i) => self.bench.get(i).and_then(|s| s.as_ref()),
        }
    }

    pub fn get_slot_mut(&mut self, slot_ref: SlotRef) -> Option<&mut PokemonSlot> {
        match slot_ref {
            SlotRef::Active => self.active.as_mut(),
            SlotRef::Bench(i) => self.bench.get_mut(i).and_then(|s| s.as_mut()),
        }
    }
}

/// Game phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Phase {
    Setup,
    Mulligan,
    Draw,
    Energy,
    Play,
    Retreat,
    Attack,
    End,
}

impl fmt::Display for Phase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Phase::Setup => write!(f, "Setup"),
            Phase::Mulligan => write!(f, "Mulligan"),
            Phase::Draw => write!(f, "Draw"),
            Phase::Energy => write!(f, "Energy"),
            Phase::Play => write!(f, "Play"),
            Phase::Retreat => write!(f, "Retreat"),
            Phase::Attack => write!(f, "Attack"),
            Phase::End => write!(f, "End"),
        }
    }
}

/// Game turn state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnState {
    pub turn_number: u16,
    pub active_player: PlayerId,
    pub phase: Phase,
    pub sub_phase: Option<String>,
    pub mulligan_done: bool,
    pub mulligan_count: u8,
    pub prizes_taken_this_turn: u8,
    pub attack_locked: bool,
    /// The player who takes the first turn (cannot attack on turn 1).
    pub first_player: PlayerId,
}

impl TurnState {
    pub fn new() -> Self {
        Self {
            turn_number: 0,
            active_player: PlayerId(0),
            phase: Phase::Setup,
            sub_phase: None,
            mulligan_done: false,
            first_player: PlayerId(0),
            mulligan_count: 0,
            prizes_taken_this_turn: 0,
            attack_locked: false,
        }
    }
}

/// Game state
#[derive(Debug, Clone)]
pub struct GameState {
    pub players: [PlayerState; 2],
    pub turn: TurnState,
    pub card_registry: CardRegistry,
    pub cards: std::collections::HashMap<CardInstanceId, CardInstance>,
    pub next_card_id: u32,
    pub winner: Option<PlayerId>,
    pub action_log: Vec<crate::action::LoggedAction>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            players: [PlayerState::new(), PlayerState::new()],
            turn: TurnState::new(),
            card_registry: CardRegistry::new(),
            cards: std::collections::HashMap::new(),
            next_card_id: 0,
            winner: None,
            action_log: Vec::new(),
        }
    }

    pub fn card_registry_mut(&mut self) -> &mut CardRegistry {
        &mut self.card_registry
    }

    pub fn add_card(&mut self, def_id: CardDefId, owner: PlayerId) -> CardInstanceId {
        let id = CardInstanceId(self.next_card_id);
        self.next_card_id += 1;
        let instance = CardInstance::new(id, def_id, owner);
        self.cards.insert(id, instance);
        id
    }

    pub fn get_card(&self, id: CardInstanceId) -> Option<&CardInstance> {
        self.cards.get(&id)
    }

    /// Iterate over all card instances (for cases that need full traversal).
    pub fn cards_iter(&self) -> impl Iterator<Item = &CardInstance> {
        self.cards.values()
    }

    pub fn get_card_def(&self, id: CardInstanceId) -> Option<&crate::card::CardDef> {
        self.get_card(id).and_then(|c| self.card_registry.get(&c.def_id))
    }

    pub fn player(&self, id: PlayerId) -> &PlayerState {
        &self.players[id.0]
    }

    pub fn player_mut(&mut self, id: PlayerId) -> &mut PlayerState {
        &mut self.players[id.0]
    }

    pub fn current_player(&self) -> PlayerId {
        self.turn.active_player
    }

    pub fn opponent(&self) -> PlayerId {
        self.turn.active_player.opponent()
    }

    pub fn is_done(&self) -> bool {
        self.winner.is_some()
    }

    pub fn state_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // Hash public state
        self.turn.turn_number.hash(&mut hasher);
        self.turn.active_player.hash(&mut hasher);
        self.turn.phase.hash(&mut hasher);
        
        // Hash deck sizes (not contents)
        self.players[0].deck.len().hash(&mut hasher);
        self.players[1].deck.len().hash(&mut hasher);
        
        // Hash hand sizes
        self.players[0].hand.len().hash(&mut hasher);
        self.players[1].hand.len().hash(&mut hasher);
        
        // Hash prize counts
        self.players[0].prizes.len().hash(&mut hasher);
        self.players[1].prizes.len().hash(&mut hasher);
        
        // Hash active Pokemon damage
        if let Some(active) = &self.players[0].active {
            active.damage.hash(&mut hasher);
        }
        if let Some(active) = &self.players[1].active {
            active.damage.hash(&mut hasher);
        }
        
        hasher.finish()
    }

    pub fn move_card(&mut self, card_id: CardInstanceId, from: &mut Vec<CardInstanceId>, to: &mut Vec<CardInstanceId>) {
        if let Some(pos) = from.iter().position(|&id| id == card_id) {
            from.remove(pos);
            to.push(card_id);
        }
    }

    pub fn draw_cards(&mut self, player: PlayerId, count: usize) -> Vec<CardInstanceId> {
        let deck = &mut self.players[player.0].deck;
        let hand = &mut self.players[player.0].hand;
        
        let mut drawn = Vec::new();
        for _ in 0..count {
            if let Some(card) = deck.pop() {
                drawn.push(card);
                hand.push(card);
            }
        }
        drawn
    }

    pub fn setup_initial_state(&mut self, player_deck: &Deck, opponent_deck: &Deck, rng: &mut GameRng) {
        // Add all cards to the game
        let mut all_cards: Vec<(CardInstanceId, PlayerId)> = Vec::new();
        
        for card_id in player_deck.expand() {
            let id = self.add_card(card_id, PlayerId(0));
            all_cards.push((id, PlayerId(0)));
        }
        
        for card_id in opponent_deck.expand() {
            let id = self.add_card(card_id, PlayerId(1));
            all_cards.push((id, PlayerId(1)));
        }
        
        // Shuffle and deal
        rng.shuffle(&mut all_cards);
        
        for (card_id, owner) in all_cards {
            let player = &mut self.players[owner.0];
            player.deck.push(card_id);
        }
        
        // Set aside prize cards (6 per player)
        for player_id in [PlayerId(0), PlayerId(1)] {
            let player = &mut self.players[player_id.0];
            for _ in 0..PRIZE_CARDS {
                if let Some(card) = player.deck.pop() {
                    player.prizes.push(card);
                }
            }
        }

        // Draw initial hands
        for player_id in [PlayerId(0), PlayerId(1)] {
            self.draw_cards(player_id, INITIAL_HAND_SIZE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::presets::load_miraidon_charizard_cards;
    use crate::deck::templates;

    #[test]
    fn test_player_state() {
        let mut state = PlayerState::new();
        assert_eq!(state.bench_count(), 0);
        assert!(!state.has_pokemon_in_play());
    }

    #[test]
    fn test_game_state() {
        let mut state = GameState::new();
        state.card_registry = load_miraidon_charizard_cards();
        
        let miraidon = templates::miraidon_deck();
        let charizard = templates::charizard_pidgeot_deck();
        
        let mut rng = GameRng::new(42);
        state.setup_initial_state(&miraidon, &charizard, &mut rng);
        
        assert_eq!(state.players[0].hand.len(), 7);
        assert_eq!(state.players[1].hand.len(), 7);
        assert_eq!(state.players[0].deck.len(), 47);
        assert_eq!(state.players[1].deck.len(), 47);
        assert_eq!(state.players[0].prizes.len(), 6);
        assert_eq!(state.players[1].prizes.len(), 6);
    }

    #[test]
    fn test_state_hash() {
        let mut state1 = GameState::new();
        let mut state2 = GameState::new();
        
        assert_eq!(state1.state_hash(), state2.state_hash());
    }
}