//! A fake of the GitHub CLI's tokens, for tests: see `crate::testing` for a fake GitHub.

use std::sync::atomic::{AtomicUsize, Ordering};

use super::TokenSource;
use crate::integrations::IntegrationProblem;

/// Gives `t1`, `t2`... each time it's asked.
#[derive(Default)]
pub struct FakeTokens {
    asked: AtomicUsize,
}

impl TokenSource for FakeTokens {
    async fn token(&self, _login: &str) -> Result<String, IntegrationProblem> {
        let asked = self.asked.fetch_add(1, Ordering::SeqCst) + 1;
        Ok(format!("t{asked}"))
    }
}

impl FakeTokens {
    pub fn asked(&self) -> usize {
        self.asked.load(Ordering::SeqCst)
    }
}
