//! Notion's API, for the pages linked to todos: with the personal access token of each workspace connected to Notion,
//! kept in the Keychain.

pub mod client;
pub mod rules;
pub mod snapshot;

pub use client::{Notion, KEYCHAIN_SERVICE};
