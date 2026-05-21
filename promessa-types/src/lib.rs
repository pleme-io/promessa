//! `promessa-types` — typed surface for the Viggy Method.
//!
//! Source-of-truth Rust types for every typed enum, trait, and CR shape
//! declared in [pleme-io/theory/VIGGY-AUTHORING.md] §15.1 + §15.2.
//!
//! ## Reading order
//!
//! 1. [`TargetController`] — the central trait. Every per-kind controller
//!    (Security, SLA, CostBudget, Compliance, CustomerKpi) implements
//!    it. The trait IS the surface; `trait_laws_obeyed!(MyController)`
//!    cross-pins every implementor against 10 invariants
//!    (VIGGY-AUTHORING §10.1).
//!
//! 2. [`PromessaTargetKind`] — the canonical five (+ Custom escape
//!    hatch). One promessa declares exactly one kind.
//!
//! 3. [`TypedAction`] — the dispatch surface. Acting always goes
//!    through one of these variants (Compounding #1 — solve once).
//!
//! 4. [`Severity`] — three-tier monotonic classification.
//!
//! 5. [`RemediationPolicy`] — what to do on a severity transition.
//!
//! 6. [`AnomalyEmission`] — the typed emission consumed by the
//!    AnomalyController (peer crate, future M2).
//!
//! All types are `#[derive(Serialize, Deserialize, JsonSchema)]` so
//! they round-trip through YAML CRs and the gRPC/REST/GraphQL
//! transports without per-variant duplication.

pub mod action;
pub mod anomaly;
pub mod authoring;
pub mod controller;
pub mod decision;
pub mod kind;
pub mod severity;

pub use action::{ReconcilerKind, TypedAction};
pub use anomaly::{AnomalyEmission, AnomalyKind};
pub use authoring::{AuthoringDecision, DeferReason, PeerControllerKind, SubstrateRoute};
pub use controller::TargetController;
pub use decision::{Decision, RemediationPolicy};
pub use kind::PromessaTargetKind;
pub use severity::Severity;
