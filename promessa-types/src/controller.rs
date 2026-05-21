//! `TargetController` — the central trait every per-kind controller
//! implements. Trait laws cross-pinned via the `trait_laws_obeyed!`
//! macro (VIGGY-AUTHORING §10.1).

use std::fmt::Debug;

use serde::{de::DeserializeOwned, Serialize};

use crate::action::TypedAction;
use crate::decision::Decision;
use crate::kind::PromessaTargetKind;
use crate::severity::Severity;

/// The TargetController trait. One per `PromessaTargetKind`:
/// `SecurityController` (M1, akeyless-nix-images FedRAMP SCR driver),
/// `SlaController` (M3), `CostBudgetController` (M3),
/// `ComplianceController` (M3), `CustomerKpiController` (M3).
///
/// Per VIGGY-LEGOS Part II.6, the diff/classify/decide functions are
/// **pure** — same inputs, same outputs, no side effects. This is the
/// load-bearing invariant `act_no_shell` (VIGGY-AUTHORING §5.2 trait
/// law `act_no_shell`) — pure compute on typed inputs; I/O happens at
/// the Observation and Action legs only.
pub trait TargetController: Send + Sync + 'static {
    /// The promessa's typed declaration shape.
    type Spec: Serialize + DeserializeOwned + Debug + Clone + Send + Sync + 'static;

    /// What `Observation` projects into for this controller.
    type Snapshot: Serialize + DeserializeOwned + Debug + Clone + Send + Sync + 'static;

    /// The per-kind drift type — what `diff(spec, snapshot)` returns.
    type Drift: Serialize + DeserializeOwned + Debug + Clone + Send + Sync + 'static;

    /// The canonical kind this controller serves.
    const KIND: PromessaTargetKind;

    /// Compute drift from spec + observed snapshot. Pure.
    fn diff(&self, spec: &Self::Spec, snapshot: &Self::Snapshot) -> Self::Drift;

    /// Classify a drift into a severity tier. Pure + monotonic
    /// (trait law `classify_monotonic`).
    fn classify(&self, drift: &Self::Drift) -> Severity;

    /// Decide an action from (spec, severity, drift). Pure.
    fn decide(&self, spec: &Self::Spec, severity: Severity, drift: &Self::Drift) -> Decision;

    /// Pre-built no-action decision for this controller. Default impl
    /// uses [`Decision::NoAction`].
    fn no_action() -> Decision {
        Decision::NoAction
    }

    /// Pre-built no-op action. Default impl uses [`TypedAction::Noop`].
    fn no_op_action() -> TypedAction {
        TypedAction::Noop
    }
}
