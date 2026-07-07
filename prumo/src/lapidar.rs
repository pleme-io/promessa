//! `lapidar` — the **accept-if-improved-else-revert** self-tune core, composed
//! BY REFERENCE from the shipped lapidar contract.
//!
//! ## What this is (and what it is NOT)
//!
//! This is the SHADOW-FIRST hill-climb / coordinate-descent primitive every
//! delivery-chain axis is wrapped in: propose a knob move, measure its effect
//! **in shadow** (before any live commit), keep it iff it *strictly* improves
//! the objective past a margin, else revert. It is **monotone-non-regressing**:
//! efficiency compounds over ticks instead of oscillating, and a regressing move
//! is reverted, never shipped.
//!
//! It is **not a fork**. The shipped pure core is
//! [`sui-supercacheci::memory::evaluate_tune`] (`TuneProposal` / `TuneOutcome` /
//! `EfficiencyReading::objective` / `TUNE_MARGIN`, 3 green tests). prumo composes
//! it BY REFERENCE (the breathe-invariant idiom) rather than taking a hard Cargo
//! dep on the sui workspace: the semantics here are **identical** —
//! `Accept ⟺ after.objective() > before.objective() + margin`, `Revert`
//! otherwise — and the shipped source is cited so the mirror is auditable and the
//! contract can never silently diverge. The [`crate::catalog`] row for the
//! continuous-efficiency axis carries that same `doctrine_ref`.
//!
//! [`sui-supercacheci::memory::evaluate_tune`]: https://github.com/pleme-io/sui/blob/main/sui-supercacheci/src/memory.rs

use serde::{Deserialize, Serialize};

/// The `doctrine_ref` this module composes — the shipped lapidar core. Named as
/// data so the [`crate::catalog`] and the honesty tests can assert prumo cites
/// its source rather than claiming to own the algorithm.
pub const LAPIDAR_DOCTRINE_REF: &str =
    "sui-supercacheci::memory::evaluate_tune (accept-if-improved-else-revert, shipped pure core)";

/// A measured reading of ONE delivery-chain axis at a point in time — the
/// generalization of lapidar's `EfficiencyReading` to any axis. Integer fields
/// keep the objective deterministic (no float non-determinism). Every field has
/// a fixed direction, folded into [`objective`](AxisReading::objective).
///
/// **Cost is the dominant term** so that a tie (or near-tie) on performance
/// resolves by COST — the standing rule "cost-conflicts resolve by cost"
/// (see [`crate::rationale`]) is baked into the scalar objective, not bolted on.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct AxisReading {
    /// The axis' effective COST, milli-USD per hour (lower is better). THE
    /// dominant term — weighted so a perf tie breaks toward the cheaper option.
    pub cost_milli: u32,
    /// The axis' performance score (higher is better) — throughput, hit-rate,
    /// inverse-latency, whatever the axis maximizes. Unit-normalized by the axis.
    pub perf_score: u32,
    /// The axis' resiliency score `0..=100` (higher is better) — spot-pool depth,
    /// replica floor headroom, drain-ahead coverage.
    pub resiliency: u8,
    /// Wasted resource, MiB-equivalent (lower is better) — over-carve, cold-cache
    /// bytes, re-shipped layers. The efficiency slack the loop grinds down.
    pub waste: u32,
}

impl AxisReading {
    /// The single scalar objective — **higher is better**. Fixed integer weights
    /// (deterministic; no float drift), mirroring
    /// `sui-supercacheci::memory::EfficiencyReading::objective`. Cost is weighted
    /// heaviest (×100) so it dominates: a performance tie resolves by cost, and no
    /// perf gain that is not worth its cost is ever accepted.
    ///
    /// `objective = perf×10 + resiliency×50 − cost×100 − waste`
    #[must_use]
    pub fn objective(&self) -> i64 {
        i64::from(self.perf_score) * 10 + i64::from(self.resiliency) * 50
            - i64::from(self.cost_milli) * 100
            - i64::from(self.waste)
    }

    /// Direct cost comparison — the standing tie-break. When two candidate values
    /// tie (or conflict) on the objective, the cheaper one wins; this is the
    /// primitive [`crate::rationale::resolve_by_cost`] is built on.
    #[must_use]
    pub fn is_cheaper_than(&self, other: &Self) -> bool {
        self.cost_milli < other.cost_milli
    }
}

/// The outcome of evaluating a shadow-measured tune — accept it (promote LIVE) or
/// revert it (keep the prior posture). Mirrors `sui-supercacheci::memory::TuneOutcome`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TuneOutcome {
    /// The shadow reading strictly improves the objective past the margin —
    /// promote the knob change to LIVE (shadow-gated commit).
    Accept,
    /// No improvement past the margin — revert; keep the prior posture. Also the
    /// outcome for a marginal/noise move (anti-churn).
    Revert,
}

/// A shadow-measured tune proposal — the `before` reading and the reading
/// measured **in shadow** after the candidate move. Shadow-first by construction:
/// `after_shadow` is observed before any live mutation, so a regression is
/// reverted before it ever ships. Mirrors `sui-supercacheci::memory::TuneProposal`
/// generalized over any axis.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct TuneProposal {
    /// The reading before the candidate move.
    pub before: AxisReading,
    /// The reading measured IN SHADOW after the candidate move (never live).
    pub after_shadow: AxisReading,
}

impl TuneProposal {
    /// The signed objective delta the move would produce (`after − before`).
    /// Positive = improvement. Carried so a decision can report *how much* it
    /// gained, not just accept/revert.
    #[must_use]
    pub fn delta(&self) -> i64 {
        self.after_shadow.objective() - self.before.objective()
    }
}

/// The objective margin a tune must clear to be accepted — dampens churn so the
/// self-tuner does not thrash on noise. This is part of the FIXED honesty core:
/// it is a `const`, never a field the optimizer can move (see [`crate::honesty`]).
/// Value matches the shipped `sui-supercacheci::memory::TUNE_MARGIN`.
pub const TUNE_MARGIN: i64 = 50;

/// Evaluate a shadow-measured tune — the lapidar accept-if-improved-else-revert
/// verdict. Pure/deterministic and **shadow-first by construction**: the decision
/// is a total function of the two readings, `after_shadow` is measured before any
/// live mutation, and a regression (or a sub-margin noise move) is reverted before
/// it ever ships. This is what makes every delivery-chain axis *continuously more
/// efficient* — it only ever keeps changes that measurably helped.
///
/// Mirrors `sui-supercacheci::memory::evaluate_tune` exactly (cited via
/// [`LAPIDAR_DOCTRINE_REF`]); the margin is the FIXED [`TUNE_MARGIN`], not a
/// tunable knob.
#[must_use]
pub fn evaluate_tune(p: &TuneProposal) -> TuneOutcome {
    if p.after_shadow.objective() > p.before.objective() + TUNE_MARGIN {
        TuneOutcome::Accept
    } else {
        TuneOutcome::Revert
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(cost: u32, perf: u32, res: u8, waste: u32) -> AxisReading {
        AxisReading { cost_milli: cost, perf_score: perf, resiliency: res, waste }
    }

    #[test]
    fn accepts_a_measured_improvement() {
        // Cheaper + faster + more resilient + less waste — clear win past margin.
        let before = reading(500, 70, 60, 500);
        let after = reading(400, 85, 80, 200);
        assert_eq!(
            evaluate_tune(&TuneProposal { before, after_shadow: after }),
            TuneOutcome::Accept
        );
    }

    #[test]
    fn reverts_a_regression() {
        let before = reading(400, 85, 80, 200);
        let worse = reading(600, 60, 40, 900);
        assert_eq!(
            evaluate_tune(&TuneProposal { before, after_shadow: worse }),
            TuneOutcome::Revert
        );
    }

    #[test]
    fn reverts_within_the_margin_no_churn() {
        // A tiny improvement under the margin must NOT churn the posture.
        let before = reading(400, 80, 50, 0);
        let marginal = reading(400, 81, 50, 0); // +10 objective, under the 50 margin
        assert_eq!(
            evaluate_tune(&TuneProposal { before, after_shadow: marginal }),
            TuneOutcome::Revert
        );
    }

    #[test]
    fn cost_dominates_the_objective_a_perf_tie_breaks_by_cost() {
        // Same perf/resiliency/waste; the cheaper reading has the higher objective.
        let cheap = reading(300, 80, 50, 0);
        let dear = reading(400, 80, 50, 0);
        assert!(cheap.objective() > dear.objective(), "cheaper wins the objective");
        assert!(cheap.is_cheaper_than(&dear));
    }

    #[test]
    fn delta_reports_the_gain() {
        let before = reading(500, 70, 60, 500);
        let after = reading(400, 85, 80, 200);
        let p = TuneProposal { before, after_shadow: after };
        assert!(p.delta() > 0, "an improving move has a positive delta");
        assert_eq!(p.delta(), after.objective() - before.objective());
    }

    #[test]
    fn evaluate_is_pure_and_deterministic() {
        let p = TuneProposal { before: reading(500, 70, 60, 500), after_shadow: reading(400, 85, 80, 200) };
        assert_eq!(evaluate_tune(&p), evaluate_tune(&p));
    }
}
