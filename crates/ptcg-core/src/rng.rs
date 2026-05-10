//! Deterministic random number generation

use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Game RNG with seed support for reproducibility
#[derive(Debug, Clone)]
pub struct GameRng {
    rng: ChaCha8Rng,
}

impl GameRng {
    /// Create a new RNG with a seed
    pub fn new(seed: u64) -> Self {
        let rng = ChaCha8Rng::seed_from_u64(seed);
        Self { rng }
    }

    /// Generate a random u64
    pub fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    /// Generate a random usize in range [0, n)
    pub fn next_usize(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.rng.next_u64() as usize) % n
    }

    /// Generate a random bool with given probability of true
    pub fn next_bool(&mut self, probability: f64) -> bool {
        self.rng.gen_bool(probability)
    }

    /// Flip a coin (50/50)
    pub fn coin_flip(&mut self) -> bool {
        self.rng.gen_bool(0.5)
    }

    /// Shuffle a slice in place
    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        slice.shuffle(&mut self.rng);
    }

    /// Pick a random element from a slice
    pub fn pick<T: Clone>(&mut self, slice: &[T]) -> Option<T> {
        if slice.is_empty() {
            return None;
        }
        Some(slice[self.next_usize(slice.len())].clone())
    }

    /// Pick multiple random elements without replacement
    pub fn pick_n<T: Clone>(&mut self, slice: &[T], n: usize) -> Vec<T> {
        let n = n.min(slice.len());
        let mut indices: Vec<usize> = (0..slice.len()).collect();
        self.shuffle(&mut indices);
        indices[..n].iter().map(|&i| slice[i].clone()).collect()
    }

    /// Get underlying RNG for external use
    pub fn inner(&mut self) -> &mut ChaCha8Rng {
        &mut self.rng
    }
}

impl Default for GameRng {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic() {
        let mut rng1 = GameRng::new(42);
        let mut rng2 = GameRng::new(42);
        
        let val1 = rng1.next_u64();
        let val2 = rng2.next_u64();
        
        assert_eq!(val1, val2);
    }

    #[test]
    fn test_shuffle() {
        let mut rng = GameRng::new(42);
        let mut vec = vec![1, 2, 3, 4, 5];
        rng.shuffle(&mut vec);
        
        // With same seed, should produce same result
        let mut rng2 = GameRng::new(42);
        let mut vec2 = vec![1, 2, 3, 4, 5];
        rng2.shuffle(&mut vec2);
        
        assert_eq!(vec, vec2);
    }

    #[test]
    fn test_pick() {
        let mut rng = GameRng::new(42);
        let slice = vec![1, 2, 3, 4, 5];
        
        let picked = rng.pick(&slice);
        assert!(picked.is_some());
        assert!(slice.contains(&picked.unwrap()));
    }

    #[test]
    fn test_pick_n() {
        let mut rng = GameRng::new(42);
        let slice = vec![1, 2, 3, 4, 5];
        
        let picked = rng.pick_n(&slice, 3);
        assert_eq!(picked.len(), 3);
        
        // Check no duplicates
        let mut unique = picked.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), picked.len());
    }
}