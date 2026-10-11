//! GitHub's API, for the status of the pull requests and issues linked to todos.

pub mod client;
pub mod query;
pub mod snapshot;
#[cfg(test)]
pub(crate) mod testing;

pub use client::{GitHub, GitHubError, TokenSource};
