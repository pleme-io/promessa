//! `PromessaTargetKind` — the canonical five (+ Custom escape).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The five canonical TargetController kinds plus a Custom escape
/// hatch. Per VIGGY-AUTHORING §1.1 Q5, every promessa classifies into
/// exactly one of these. Adding a sixth kind requires a substrate-
/// improvement ticket — three Custom uses force extraction of a new
/// canonical kind (★★ macros-everywhere three-times rule).
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum PromessaTargetKind {
    /// Availability / latency / error-rate over a window.
    Sla,
    /// Spend / billing over a period.
    CostBudget,
    /// Regulatory posture against a baseline.
    Compliance,
    /// Customer-facing metric (NPS / CSAT / retention / activation).
    CustomerKpi,
    /// CVE age / banned packages / runtime posture / supply-chain.
    Security,
    /// Escape hatch — adopt one of the five by analogy if at all
    /// possible; Custom requires a substrate-improvement ticket.
    Custom,
}

impl PromessaTargetKind {
    /// Canonical kebab-case slug — the bytes the `(defpromessa …)`
    /// Lisp surface emits on `:kind <slug>`.
    #[must_use]
    pub const fn kebab(self) -> &'static str {
        match self {
            Self::Sla => "sla",
            Self::CostBudget => "cost-budget",
            Self::Compliance => "compliance",
            Self::CustomerKpi => "customer-kpi",
            Self::Security => "security",
            Self::Custom => "custom",
        }
    }
}
