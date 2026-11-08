//! Asynchronous completion cancellation and request token tracking.

use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct CancellationToken {
    current_token: AtomicU64,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            current_token: AtomicU64::new(0),
        }
    }

    /// Issue a new request token, superseding all prior tokens.
    pub fn next_token(&self) -> u64 {
        self.current_token.fetch_add(1, Ordering::SeqCst) + 1
    }

    /// Check if a given token is still the latest active token.
    pub fn is_valid(&self, token: u64) -> bool {
        token == self.current_token.load(Ordering::SeqCst)
    }

    /// Invalidate all in-flight requests.
    pub fn cancel_all(&self) {
        self.current_token.fetch_add(1, Ordering::SeqCst);
    }
}
