//! `Severity` — three-tier monotonic classification per VIGGY-AUTHORING §4.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// The three canonical severity tiers. Order is monotonic:
/// `Cosmetic < Functional < Critical`. Per VIGGY-AUTHORING §4.2, the
/// `classify_monotonic` trait law is property-tested for every
/// [`crate::TargetController`] — strictly larger drift produces
/// at-least-as-severe Severity.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    /// Within healthy operating bounds. No action required.
    Cosmetic = 0,
    /// Outside healthy bounds but within remediation SLA.
    Functional = 1,
    /// Beyond SLA or representing immediate audit/security risk.
    Critical = 2,
}

impl Severity {
    /// Does this severity warrant blocking the gate?
    #[must_use]
    pub const fn blocks_gate(self) -> bool {
        matches!(self, Self::Critical)
    }
}
