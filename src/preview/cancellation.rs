//! Monotonic cancellation token management for fast preview abortion.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// A lightweight, thread-safe monotonic cancellation token.
/// Incremented whenever the active selection changes to discard obsolete preview generation tasks.
#[derive(Debug, Clone, Default)]
pub struct MonotonicCancellation {
    generation: Arc<AtomicU64>,
}

impl MonotonicCancellation {
    pub fn new() -> Self {
        Self {
            generation: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Advance the generation counter, immediately invalidating any active background tasks.
    pub fn cancel_current_and_advance(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Fetch the current generation snapshot.
    pub fn current(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }

    /// Returns a guard closure checking whether this specific generation remains valid.
    pub fn is_cancelled(&self, expected_generation: u64) -> bool {
        self.generation.load(Ordering::Relaxed) != expected_generation
    }

    /// Obtain raw Arc reference to atomic counter for zero-cost sharing across worker threads.
    pub fn raw_arc(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.generation)
    }
}
