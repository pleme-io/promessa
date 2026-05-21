//! `promessa` — facade re-export. Single entry point for downstream
//! crates. The actual types live in [`promessa_types`]; runtime in
//! `promessa_runtime`; transports in the per-protocol crates.
//!
//! ```
//! use promessa::TargetController;
//! ```
pub use promessa_types::*;
