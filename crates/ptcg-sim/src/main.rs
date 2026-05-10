//! PTCG Simulation Runner
//! 
//! Command-line tool for self-play and benchmarking.

use anyhow::Result;
use clap::Parser;
use ptcg_core::{
    deck::{MatchConfig, templates, StartingPlayer},
    engine::Engine,
    state::PlayerId,
};
use rayon::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
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

    /// Number of threads (0 = auto)
    #[arg(short = 't', long, default_value_t = 0)]
    threads: usize,

    /// Output file for results
    #[arg(short, long)]
    output: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(if args.verbose { Level::DEBUG } else { Level::INFO })
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    if args.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(args.threads)
            .build_global()?;
    }

    info!(
        "Starting PTCG self-play benchmark — {} games, {} threads",
        args.games,
        rayon::current_num_threads()
    );

    let start = Instant::now();

    // Parallel game execution
    let player0_wins = AtomicU64::new(0);
    let player1_wins = AtomicU64::new(0);
    let draws = AtomicU64::new(0);
    let total_turns = AtomicU64::new(0);
    let total_steps = AtomicU64::new(0);
    let max_turns = AtomicU64::new(0);
    let min_turns = AtomicU64::new(u64::MAX);

    (0..args.games).into_par_iter().for_each(|i| {
        let game_seed = args.seed.wrapping_add(i as u64);
        let result = play_game(game_seed);

        total_turns.fetch_add(result.turns as u64, Ordering::Relaxed);
        total_steps.fetch_add(result.steps as u64, Ordering::Relaxed);
        // Update max_turns (CAS loop)
        let mut current = max_turns.load(Ordering::Relaxed);
        while result.turns as u64 > current {
            match max_turns.compare_exchange_weak(current, result.turns as u64, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(v) => current = v,
            }
        }
        // Update min_turns
        current = min_turns.load(Ordering::Relaxed);
        while (result.turns as u64) < current {
            match min_turns.compare_exchange_weak(current, result.turns as u64, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(v) => current = v,
            }
        }
        match result.winner {
            Some(PlayerId(0)) => { player0_wins.fetch_add(1, Ordering::Relaxed); }
            Some(PlayerId(1)) => { player1_wins.fetch_add(1, Ordering::Relaxed); }
            _ => { draws.fetch_add(1, Ordering::Relaxed); }
        }
    });

    let elapsed = start.elapsed();
    let n = args.games as f64;
    info!("Benchmark complete in {:.2}s", elapsed.as_secs_f64());
    info!("Throughput: {:.0} games/s, {:.0} steps/s",
        n / elapsed.as_secs_f64(),
        total_steps.load(Ordering::Relaxed) as f64 / elapsed.as_secs_f64());

    let p0 = player0_wins.load(Ordering::Relaxed);
    let p1 = player1_wins.load(Ordering::Relaxed);
    let d = draws.load(Ordering::Relaxed);
    let t = total_turns.load(Ordering::Relaxed);
    let s = total_steps.load(Ordering::Relaxed);
    let mx = max_turns.load(Ordering::Relaxed);
    let mn = if min_turns.load(Ordering::Relaxed) == u64::MAX { 0 } else { min_turns.load(Ordering::Relaxed) };

    println!("Benchmark Results");
    println!("=================");
    println!("Total games: {}", args.games);
    println!("Player 0 wins: {} ({:.1}%)", p0, p0 as f64 / n * 100.0);
    println!("Player 1 wins: {} ({:.1}%)", p1, p1 as f64 / n * 100.0);
    println!("Draws: {}", d);
    println!("Average turns: {:.1}", t as f64 / n);
    println!("Turn range: {} - {}", mn, mx);
    println!("Average steps: {:.1}", s as f64 / n);
    println!("Time: {:.2}s", elapsed.as_secs_f64());
    println!("Throughput: {:.0} games/s", n / elapsed.as_secs_f64());

    if let Some(output) = &args.output {
        let stats = serde_json::json!({
            "total_games": args.games,
            "player0_wins": p0,
            "player1_wins": p1,
            "draws": d,
            "avg_turns": t as f64 / n,
            "min_turns": mn,
            "max_turns": mx,
            "elapsed_secs": elapsed.as_secs_f64(),
            "games_per_sec": n / elapsed.as_secs_f64(),
        });
        std::fs::write(output, serde_json::to_string_pretty(&stats)?)?;
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
    let max_steps = 5000;

    while !engine.state().is_done() && steps < max_steps {
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
    }
}

#[derive(Debug, Clone)]
struct GameResult {
    winner: Option<PlayerId>,
    turns: u16,
    steps: usize,
}