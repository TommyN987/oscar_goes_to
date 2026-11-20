//! The game's rules as types and pure functions.
//!
//! This crate performs no IO and depends on no async runtime or serialization crate (spec rule 2.1.2).

pub mod ballot;
pub mod player;
pub mod results;
pub mod season;
pub mod seed_diff;
pub mod standings;
pub mod time;

#[cfg(test)]
mod fixtures;
