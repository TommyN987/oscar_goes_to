//! Use cases and the ports they need: clock and storage so far; identity provider and results source
//! arrive with the plan items that first call them.
//!
//! This crate depends on no HTTP or SQL crate (spec rule 2.1.2).

pub mod actor;
pub mod admin_emails;
pub mod clock;
pub mod session;
pub mod session_token;
pub mod storage;
