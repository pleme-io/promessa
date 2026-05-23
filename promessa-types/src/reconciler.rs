//! `Reconciler` — the trait for the universal Reconciler engine per
//! VIGGY-LEGOS Part IV + CONVERGENCE-SUBSTRATE §III.2.
//!
//! Every [`crate::action::ReconcilerKind`] variant in
//! [`crate::action::TypedAction::ReconcilerApply`] maps to exactly one
//! [`Reconciler`] impl that **wraps** an existing substrate primitive —
//! cartorio admit, Zot tag revoke, FluxCD git commit, skopeo Harbor
//! mirror, … The wrap-not-compete invariant (VIGGY-AUTHORING §3.3):
//! Reconcilers never reinvent. They thread typed inputs into existing
//! tools and surface the tool's response as a typed receipt.
//!
//! ## Trait laws (Reconciler-side, per VIGGY-AUTHORING §5.2 +
//! VIGGY-LEGOS Part IV.7)
//!
//! 1. `act_idempotent_on_noop` — calling [`Reconciler::act`] when the
//!    target state already holds returns
//!    [`ReconcilerOutcome::AlreadyConverged`], not a fresh `Applied`.
//! 2. `act_deterministic_for_same_input` — the same typed spec always
//!    produces the same `ReconcilerOutcome` variant. Receipt *bytes*
//!    may differ if the wrapped primitive embeds a clock, but the
//!    typed shape never does.
//! 3. `observe_receipt_typed` — `Applied` carries a typed receipt,
//!    never a free-form string.
//! 4. `flag_gated_returns_typed` — a Reconciler whose feature flag is
//!    disabled returns [`ReconcilerOutcome::FlagGated`] explicitly. It
//!    never silently no-ops.
//! 5. `repeated_failure_terminates` — N consecutive failures escalate
//!    via the EscalationLadder. This law is **engine-enforced** (the
//!    Reconciler itself is stateless across calls).
//!
//! Laws 1–4 are checked by the `define_reconciler_laws!` macro in
//! `promessa-controller-common`. Law 5 is in the engine.

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::action::ReconcilerKind;

/// One Reconciler impl per [`ReconcilerKind`] variant. Async because
/// every concrete Reconciler does I/O against an external substrate
/// primitive (cartorio HTTP, Zot registry, FluxCD git, skopeo, …).
#[async_trait]
pub trait Reconciler: Send + Sync + 'static {
    /// The kind this Reconciler serves. Engine's dispatch table is
    /// keyed on this value; the engine asserts uniqueness at boot.
    const KIND: ReconcilerKind;

    /// Typed input shape. The engine accepts `serde_json::Value` from
    /// the wire and `Spec::deserialize`s into this typed shape before
    /// calling [`Reconciler::act`].
    type Spec: Serialize + DeserializeOwned + Send + Sync + 'static;

    /// Typed receipt body. Goes into [`ReconcilerOutcome::Applied`].
    /// Per law `observe_receipt_typed` — never a free-form string.
    type Receipt: Serialize + DeserializeOwned + Send + Sync + 'static;

    /// Execute the action wrapping the substrate primitive. The only
    /// side effect is the I/O against the wrapped primitive; trait
    /// laws 1–4 above hold.
    async fn act(&self, spec: Self::Spec) -> ReconcilerOutcome<Self::Receipt>;
}

/// What a Reconciler returns after acting. Typed so the engine never
/// has to parse free-form output.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "outcome", rename_all = "kebab-case")]
pub enum ReconcilerOutcome<R> {
    /// The action ran and produced a typed receipt. The receipt is
    /// what gets written to the parent CR's `.status.reconcilerOutcomes[]`
    /// + appended to the OutcomeChain receipt for the promessa.
    Applied { receipt: R },

    /// The spec describes a state that already holds — idempotent no-op
    /// per trait law `act_idempotent_on_noop`. Engine treats this as
    /// success.
    AlreadyConverged,

    /// A feature flag (named by `flag`) gates this action; the
    /// Reconciler ships dormant-but-functional. Flipping the flag
    /// re-runs through the normal path on the next dispatch.
    ///
    /// This is the canonical mechanism for `gameWardenForwarding` and
    /// any other "ship the code, don't enable it yet" Reconciler.
    /// `flag` is `String` (not `&'static str`) so the outcome
    /// deserializes cleanly from a CR `.status.reconcilerOutcomes[]`
    /// entry.
    FlagGated { flag: String },

    /// The substrate primitive returned an error. Structured so the
    /// engine can decide retry vs. EscalationLadder.
    Failed { error: ReconcilerError },
}

/// Categorized failure modes for the engine's retry policy.
///
/// `Transient` is auto-retried with backoff up to `MAX_RETRIES` (set
/// engine-side per CONVERGENCE-SUBSTRATE §III.2.5). Other variants
/// terminate immediately and surface to the EscalationLadder.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ReconcilerError {
    /// I/O hiccup — engine retries with exponential backoff.
    #[error("transient: {detail}")]
    Transient { detail: String },

    /// Typed spec failed validation. No retry — escalate.
    #[error("invalid spec: {detail}")]
    InvalidSpec { detail: String },

    /// Substrate primitive returned a clean refusal (4xx, policy
    /// denial). No retry — escalate.
    #[error("substrate refused: {detail}")]
    SubstrateRefused { detail: String },

    /// Reconciler internal bug. No retry — escalate.
    #[error("internal: {detail}")]
    Internal { detail: String },
}

impl ReconcilerError {
    /// Whether the engine should retry. Per CONVERGENCE-SUBSTRATE
    /// §III.2.5: only `Transient` retries.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(self, ReconcilerError::Transient { .. })
    }
}

impl<R> ReconcilerOutcome<R> {
    /// Engine helper: is this outcome terminal-success (Applied or
    /// AlreadyConverged or FlagGated)?
    #[must_use]
    pub fn is_terminal_success(&self) -> bool {
        matches!(
            self,
            ReconcilerOutcome::Applied { .. }
                | ReconcilerOutcome::AlreadyConverged
                | ReconcilerOutcome::FlagGated { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct DummyReceipt {
        ok: bool,
    }

    #[test]
    fn flag_gated_is_terminal_success() {
        let o: ReconcilerOutcome<DummyReceipt> =
            ReconcilerOutcome::FlagGated { flag: "gameWardenForwarding".into() };
        assert!(o.is_terminal_success());
    }

    #[test]
    fn applied_is_terminal_success() {
        let o = ReconcilerOutcome::Applied { receipt: DummyReceipt { ok: true } };
        assert!(o.is_terminal_success());
    }

    #[test]
    fn already_converged_is_terminal_success() {
        let o: ReconcilerOutcome<DummyReceipt> = ReconcilerOutcome::AlreadyConverged;
        assert!(o.is_terminal_success());
    }

    #[test]
    fn failed_is_not_terminal_success() {
        let o: ReconcilerOutcome<DummyReceipt> = ReconcilerOutcome::Failed {
            error: ReconcilerError::Transient { detail: "test".into() },
        };
        assert!(!o.is_terminal_success());
    }

    #[test]
    fn only_transient_retries() {
        assert!(ReconcilerError::Transient { detail: "".into() }.is_retryable());
        assert!(!ReconcilerError::InvalidSpec { detail: "".into() }.is_retryable());
        assert!(!ReconcilerError::SubstrateRefused { detail: "".into() }.is_retryable());
        assert!(!ReconcilerError::Internal { detail: "".into() }.is_retryable());
    }

    #[test]
    fn outcome_serde_roundtrip_applied() {
        let o = ReconcilerOutcome::Applied { receipt: DummyReceipt { ok: true } };
        let json = serde_json::to_value(&o).unwrap();
        assert_eq!(json["outcome"], "applied");
        assert_eq!(json["receipt"]["ok"], true);
    }

    #[test]
    fn outcome_serde_roundtrip_flag_gated() {
        let o: ReconcilerOutcome<DummyReceipt> =
            ReconcilerOutcome::FlagGated { flag: "gameWardenForwarding".into() };
        let json = serde_json::to_value(&o).unwrap();
        assert_eq!(json["outcome"], "flag-gated");
        assert_eq!(json["flag"], "gameWardenForwarding");
        let back: ReconcilerOutcome<DummyReceipt> = serde_json::from_value(json).unwrap();
        match back {
            ReconcilerOutcome::FlagGated { flag } => assert_eq!(flag, "gameWardenForwarding"),
            other => panic!("expected FlagGated, got {other:?}"),
        }
    }
}
