//! PTCG Simulation Runner
//! 
//! Command-line tool for self-play and benchmarking.

use anyhow::Result;
use clap::Parser;
use ptcg_core::{
    deck::{MatchConfig, templates, StartingPlayer},
    engine::Engine,
    observe::Observation,
    replay::Replay,
    state::PlayerId,
    action::Action,
};
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

/// Self-play benchmark command
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of games to play
    #[arg(short, long, default_value_t = 100)]
    games: usize,
    
    /// Random seed
    #[arg(short, long, default_value_t = 42)]
    seed: u64,
    
    /// Output file for results
    #[arg(short, long)]
    output: Option<String>,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Setup logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(if args.verbose { Level::DEBUG } else { Level::INFO })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    
    info!("Starting PTCG self-play benchmark");
    info!("Games: {}, Seed: {}", args.games, args.seed);
    
    let mut stats = BenchmarkStats::new();
    
    for i in 0..args.games {
        let game_seed = args.seed.wrapping_add(i as u64);
        let result = play_game(game_seed);
        stats.record(result);
        
        if (i + 1) % 10 == 0 {
            info!("Played {} games, current win rate: {:.1}%", 
                i + 1, stats.win_rate() * 100.0);
        }
    }
    
    info!("Benchmark complete!");
    info!("{}", stats);
    
    if let Some(output) = &args.output {
        let json = serde_json::to_string_pretty(&stats)?;
        std::fs::write(output, json)?;
        info!("Results written to {}", output);
    }
    
    Ok(())
}

fn play_game(seed: u64) -> GameResult {
    let config = MatchConfig {
        player_deck: templates::miraidon_deck(),
        opponent_deck: templates::charizard_pidgeot_deck(),
        player_name: "Player".into(),
        opponent_name: "Opponent".into(),
        starting_player: StartingPlayer::Random,
    };

    let mut engine = Engine::new(config, seed);
    let mut steps = 0;
    let max_steps = 5000; // Prevent infinite loops

    while !engine.state().is_done() && steps < max_steps {
        // During Setup/Mulligan, both players can act
        let is_setup = matches!(engine.state().turn.phase,
            ptcg_core::state::Phase::Setup | ptcg_core::state::Phase::Mulligan);

        let players: Vec<PlayerId> = if is_setup {
            vec![PlayerId(0), PlayerId(1)]
        } else {
            vec![engine.state().turn.active_player]
        };

        for &player in &players {
            if engine.state().is_done() {
                break;
            }
            let actions = engine.legal_actions(player);

            if actions.is_empty() {
                continue;
            }

            // Random action selection
            let action_idx = (seed.wrapping_add(steps as u64) as usize) % actions.len();
            let action = actions[action_idx].clone();
            engine.step(player, action);

            steps += 1;
        }
    }

    GameResult {
        winner: engine.state().winner,
        turns: engine.state().turn.turn_number,
        steps,
        seed,
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct GameResult {
    winner: Option<PlayerId>,
    turns: u16,
    steps: usize,
    seed: u64,
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct BenchmarkStats {
    total_games: usize,
    player0_wins: usize,
    player1_wins: usize,
    draws: usize,
    total_turns: u64,
    total_steps: u64,
    max_turns: u16,
    min_turns: u16,
}

impl BenchmarkStats {
    fn new() -> Self {
        Self {
            max_turns: 0,
            min_turns: u16::MAX,
            ..Default::default()
        }
    }
    
    fn record(&mut self, result: GameResult) {
        self.total_games += 1;
        self.total_turns += result.turns as u64;
        self.total_steps += result.steps as u64;
        self.max_turns = self.max_turns.max(result.turns);
        self.min_turns = self.min_turns.min(result.turns);
        
        match result.winner {
            Some(PlayerId(0)) => self.player0_wins += 1,
            Some(PlayerId(1)) => self.player1_wins += 1,
            _ => self.draws += 1,
        }
    }
    
    fn win_rate(&self) -> f64 {
        if self.total_games == 0 {
            return 0.0;
        }
        self.player0_wins as f64 / self.total_games as f64
    }
    
    fn avg_turns(&self) -> f64 {
        if self.total_games == 0 {
            return 0.0;
        }
        self.total_turns as f64 / self.total_games as f64
    }
}

impl std::fmt::Display for BenchmarkStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Benchmark Results")?;
        writeln!(f, "=================")?;
        writeln!(f, "Total games: {}", self.total_games)?;
        writeln!(f, "Player 0 wins: {} ({:.1}%)", self.player0_wins, self.win_rate() * 100.0)?;
        writeln!(f, "Player 1 wins: {} ({:.1}%)", self.player1_wins, (1.0 - self.win_rate()) * 100.0)?;
        writeln!(f, "Draws: {}", self.draws)?;
        writeln!(f, "Average turns: {:.1}", self.avg_turns())?;
        writeln!(f, "Turn range: {} - {}", self.min_turns, self.max_turns)?;
        Ok(())
    }
}