//! Search request cancellation tokens.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct SearchCancellation {
    token: Arc<AtomicU64>,
}

impl SearchCancellation {
    pub fn new() -> Self {
        Self {
            token: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn advance(&self) -> u64 {
        self.token.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn current(&self) -> u64 {
        self.token.load(Ordering::Relaxed)
    }

    pub fn is_cancelled(&self, expected: u64) -> bool {
        self.token.load(Ordering::Relaxed) != expected
    }

    pub fn raw_arc(&self) -> Arc<AtomicU64> {
        Arc::clone(&self.token)
    }
}
