//! `Decision` + `RemediationPolicy` — what a controller emits per tick.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::action::TypedAction;

/// What the controller decides this tick. Routed through
/// `RemediationPolicy` before any action is taken (VIGGY-AUTHORING §5).
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "verdict", rename_all = "kebab-case")]
pub enum Decision {
    /// No action this tick — gate is in Cosmetic+ state, no SLA breach.
    NoAction,

    /// Dispatch the contained action via the
    /// [`crate::TypedAction`] dispatcher.
    AutoCorrect(TypedAction),

    /// Action ready but human approval required (P1 phase). Renders
    /// to a PR draft on `pleme-io/k8s` that a human merges.
    RequireApproval(TypedAction),

    /// Emit an [`crate::AnomalyEmission`] but take no action. P0
    /// (shadow) policy default.
    Alert,

    /// Drop the decision after repeated failure — surface to the
    /// EscalationLadder. Trait law `repeated_failure_terminates`
    /// (VIGGY-AUTHORING §9.3).
    SuppressedAfterRepeatedFailure { failure_count: u32 },
}

/// How decisions are routed to actions. Declared per promessa via
/// `:remediation-policy` (VIGGY-AUTHORING §8.3).
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RemediationPolicy {
    /// Take no action. Used for retiring promessas (VIGGY-AUTHORING §8.2).
    NoOp,
    /// Emit anomalia, never dispatch action. P0 shadow mode.
    Alert,
    /// Dispatch the typed action without ceremony. P3 live mode.
    AutoCorrect,
    /// Render a PR draft; require human merge before dispatch. P1/P2.
    RequireApproval,
    /// Promote to the EscalationLadder (next escalation tier).
    Escalate,
    /// Sequence of actions — each step recorded in the receipt.
    Compose(Vec<RemediationPolicy>),
}
