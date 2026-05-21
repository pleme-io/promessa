//! `AnomalyEmission` — typed emission consumed by the future
//! AnomalyController (substrate M2).

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::kind::PromessaTargetKind;
use crate::severity::Severity;

/// A typed anomaly event emitted by a TargetController. Routed via
/// denshin NATS subject `pleme.anomaly.v1.<kind>` to the
/// AnomalyController, which applies the parent promessa's
/// RemediationPolicy.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct AnomalyEmission {
    pub anomaly_kind: AnomalyKind,
    pub parent_promessa: String,
    pub target_kind: PromessaTargetKind,
    pub severity: Severity,
    pub ts: DateTime<Utc>,
    pub payload: serde_json::Value,
    /// BLAKE3 hash of the canonical JSON serialization — anchors the
    /// emission in AnomalyChain per VIGGY-AUTHORING §15.4.
    pub canonical_hash: String,
}

/// What kind of anomaly — the discriminator for AnomalyController
/// routing.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum AnomalyKind {
    // Security-kind anomalias — first consumer per ASM-17571.
    CriticalCveFound,
    HighCveAged,
    AttestationMissing,
    // SLA-kind.
    AvailabilityBreach,
    LatencyBreach,
    ErrorRateBreach,
    // Compliance-kind.
    ControlFailing,
    BaselineDrift,
    // CostBudget-kind.
    SpendOverPace,
    SpendOverCeiling,
    // CustomerKpi-kind.
    KpiBelowMinimum,
    // Substrate / housekeeping.
    ManualBreakGlass,
    ControllerWedged,
    EscalationExhausted,
    BackpressureDetected,
    ScannerTimeout,
}
