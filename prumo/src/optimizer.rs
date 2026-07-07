//! `optimizer` — **the tick loop**: lapidar (accept-if-improved-else-revert) +
//! the camelot-posture seven-beat, composed into ONE loop over every axis.
//!
//! ## The one loop (map §6)
//!
//! Each tick, for each axis, the optimizer:
//! 1. reads the current cost/perf ([`AxisReading`]),
//! 2. takes the sealed-algorithm optimum (an [`AxisProposal`] — computed by the
//!    composed home crate, or by [`crate::seal`] for the worked examples),
//! 3. routes by the **knob-vs-structure line** (control-theory, load-bearing):
//!    - a **KNOB** move (R2) is handed to lapidar's [`evaluate_tune`] — accept iff
//!      it strictly improves past the fixed margin, else revert (shadow-first);
//!    - a **STRUCTURE** move (R3: a step-function block — BestEffort QoS, an RBAC
//!      edge, a dropped `spot_allocation_strategy`) is **escalated UP to the
//!      posture seven-beat, never knob-chased**. The instant thrash rises, the
//!      loop routes UP — it does not accelerate the knob.
//! 4. resolves cost-conflicts by COST (the [`AxisProposal::rationale`] carries the
//!    inline justification, loud where the intuitive choice loses),
//! 5. is applied **shadow-first** (`shadow_gated == true` on every decision:
//!    observe the would-apply, then commit — breathe `ShadowWouldApply` / afinar
//!    `dry_run`).
//!
//! ## Why it is ONE optimizer, not five (map §6)
//!
//! Every link's actuator is shadow-gated and every result is content-addressed,
//! so the same five-tuple `Controller` drives all five links: prumo Observes all
//! their signals in one beat, Decides across all their knobs in one beat (routing
//! each to lapidar or to the posture's R3), and the whole chain drifts toward a
//! lower-cost, lower-variance, better-packed global state tick by tick.
//!
//! ## The guarantee: **monotone-non-regressing**
//!
//! [`TickReport::net_objective_delta`] is `≥ 0` for ANY set of proposals: an
//! `Accept` fires only when the shadow objective improved past the margin
//! (positive delta), a `Revert` keeps the prior posture (zero delta), and an
//! `Escalate` mutates nothing here (zero delta). Efficiency **compounds** over
//! ticks; it never oscillates. Proven by [`TickReport`] tests + a proptest.

use serde::{Deserialize, Serialize};

use crate::axis::{Link, TickSource};
use crate::lapidar::{evaluate_tune, AxisReading, TuneOutcome, TuneProposal};
use crate::rationale::CostRationale;

/// The knob-vs-structure classification of a proposed move — the load-bearing
/// control-theory seam between the lapidar clock and the posture seven-beat.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Move {
    /// A KNOB (R2): a continuous value lapidar can tune (tier size, band setpoint,
    /// samba quota-pct, prefetch, arch/ladder rank). Handed to accept-if-improved.
    Knob,
    /// A STRUCTURE (R3): a step-function block a knob can never cross (BestEffort
    /// QoS, an RBAC edge, a missing dependency, a dropped `spot_allocation_strategy`,
    /// thrash/oscillation). Escalated UP to the posture, NEVER knob-chased.
    Structure,
}

/// A sealed-algorithm optimum proposed for one axis this tick — the composition
/// border between the sealed algorithm (in its home crate) and the tick loop.
/// Carries the current + shadow readings (shadow-first), the candidate value, its
/// cost rationale, and its knob-vs-structure classification.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct AxisProposal {
    /// The axis this proposal is for (matches a [`crate::catalog`] row name).
    pub axis: &'static str,
    /// The delivery-chain link.
    pub link: Link,
    /// Which clock drives this axis (its primary tick source).
    pub tick_source: TickSource,
    /// The value currently in effect (kept on a revert).
    pub current_value: String,
    /// The candidate value the sealed algorithm computed as the optimum.
    pub proposed_value: String,
    /// The reading with the current value.
    pub before: AxisReading,
    /// The reading measured IN SHADOW with the candidate value (never live).
    pub after_shadow: AxisReading,
    /// The inline cost rationale (loud where the intuitive choice loses).
    pub rationale: CostRationale,
    /// Knob (lapidar-tunable) vs Structure (escalate UP).
    pub move_kind: Move,
}

/// What the tick loop DID with one axis' proposal.
#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(tag = "disposition", rename_all = "kebab-case")]
pub enum Disposition {
    /// lapidar accepted the knob move — the candidate improved past the margin;
    /// applied (shadow-gated). Carries the applied value + the objective gain.
    Accepted { value: String, delta: i64 },
    /// lapidar reverted the knob move — no improvement past the margin; the prior
    /// value is kept (anti-churn / regression rejected).
    Reverted { kept_value: String },
    /// A structural block — routed UP to the named clock, NEVER knob-chased.
    Escalated { to: &'static str, reason: &'static str },
}

/// The per-axis decision — the disposition + its rationale, always shadow-gated.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct AxisDecision {
    pub axis: &'static str,
    pub link: Link,
    pub disposition: Disposition,
    pub rationale: CostRationale,
    /// Always `true` — the Act beat is shadow-first (observe the would-apply, then
    /// commit). A blind flip is unrepresentable in this loop.
    pub shadow_gated: bool,
}

impl AxisDecision {
    /// The objective delta this decision contributed to the tick — positive only
    /// for an `Accepted` move; zero for a revert or an escalate.
    #[must_use]
    pub fn objective_delta(&self) -> i64 {
        match self.disposition {
            Disposition::Accepted { delta, .. } => delta,
            Disposition::Reverted { .. } | Disposition::Escalated { .. } => 0,
        }
    }
}

/// The result of one tick over the whole chain — the per-axis decisions + the
/// aggregate roll-up. [`net_objective_delta`](TickReport::net_objective_delta) is
/// the monotone-non-regressing guarantee: `≥ 0` for any proposal set.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct TickReport {
    pub decisions: Vec<AxisDecision>,
    /// Sum of every decision's objective delta — provably `≥ 0`.
    pub net_objective_delta: i64,
    pub accepts: usize,
    pub reverts: usize,
    pub escalations: usize,
}

/// The tick-by-tick delivery-chain optimizer. Zero state: the loop is a total
/// function of the tick's proposals (the setpoint + observation live at the
/// posture runtime it composes). The honesty core it obeys is `const`
/// ([`crate::honesty`]) — the optimizer has no field it can tune it through.
#[derive(Debug, Clone, Copy, Default)]
pub struct DeliveryOptimizer;

impl DeliveryOptimizer {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Decide ONE axis — route by the knob-vs-structure line, then (for a knob) by
    /// lapidar accept-if-improved-else-revert. Pure/deterministic.
    #[must_use]
    pub fn decide_axis(&self, p: &AxisProposal) -> AxisDecision {
        let disposition = match p.move_kind {
            // STRUCTURE always escalates UP — never knob-chased, even if the
            // axis' primary clock is lapidar (thrash/step-function block).
            Move::Structure => Disposition::Escalated {
                to: TickSource::PostureSevenBeat.as_str(),
                reason: "structural block (R3) — routed UP to the posture, never knob-chased",
            },
            Move::Knob => match p.tick_source {
                // The lapidar knob lane: accept iff the shadow reading improves.
                TickSource::LapidarAcceptIfImproved => {
                    let tp = TuneProposal { before: p.before, after_shadow: p.after_shadow };
                    match evaluate_tune(&tp) {
                        TuneOutcome::Accept => Disposition::Accepted {
                            value: p.proposed_value.clone(),
                            delta: tp.delta(),
                        },
                        TuneOutcome::Revert => Disposition::Reverted {
                            kept_value: p.current_value.clone(),
                        },
                    }
                }
                // A knob whose primary clock is the posture is the posture's to
                // move (setpoint keeper); prumo surfaces + defers, never mutates.
                TickSource::PostureSevenBeat => Disposition::Escalated {
                    to: TickSource::PostureSevenBeat.as_str(),
                    reason: "setpoint owned by the posture seven-beat — deferred",
                },
                // A reactive knob is the fast interrupt lane's — handled between
                // cron ticks (retirada drain / samba back-pressure), not here.
                TickSource::ReactiveNervousSystem => Disposition::Escalated {
                    to: TickSource::ReactiveNervousSystem.as_str(),
                    reason: "interrupt lane — handled between cron ticks (fast lane)",
                },
            },
        };
        AxisDecision {
            axis: p.axis,
            link: p.link,
            disposition,
            rationale: p.rationale.clone(),
            // Every actuator in the chain is shadow-gated (map §0). A blind flip
            // has no code path.
            shadow_gated: true,
        }
    }

    /// Run ONE tick over the whole chain — decide every axis, roll up the report.
    /// The chain optimizes as one eventual-consistency co-optimization: the whole
    /// fabric drifts toward a lower-cost, lower-variance global state, and the net
    /// objective delta is provably `≥ 0` (monotone-non-regressing).
    #[must_use]
    pub fn tick(&self, proposals: &[AxisProposal]) -> TickReport {
        let decisions: Vec<AxisDecision> = proposals.iter().map(|p| self.decide_axis(p)).collect();
        let mut accepts = 0;
        let mut reverts = 0;
        let mut escalations = 0;
        let mut net = 0i64;
        for d in &decisions {
            match d.disposition {
                Disposition::Accepted { .. } => accepts += 1,
                Disposition::Reverted { .. } => reverts += 1,
                Disposition::Escalated { .. } => escalations += 1,
            }
            net += d.objective_delta();
        }
        TickReport { decisions, net_objective_delta: net, accepts, reverts, escalations }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rationale::CostRationale;

    fn reading(cost: u32, perf: u32, res: u8, waste: u32) -> AxisReading {
        AxisReading { cost_milli: cost, perf_score: perf, resiliency: res, waste }
    }

    fn knob_proposal(
        axis: &'static str,
        before: AxisReading,
        after_shadow: AxisReading,
    ) -> AxisProposal {
        AxisProposal {
            axis,
            link: Link::Run,
            tick_source: TickSource::LapidarAcceptIfImproved,
            current_value: "current".into(),
            proposed_value: "candidate".into(),
            before,
            after_shadow,
            rationale: CostRationale::uncontested("candidate", "test"),
            move_kind: Move::Knob,
        }
    }

    #[test]
    fn a_knob_improvement_is_accepted_and_applied_shadow_gated() {
        let opt = DeliveryOptimizer::new();
        let p = knob_proposal("mem-band", reading(500, 70, 60, 500), reading(400, 85, 80, 200));
        let d = opt.decide_axis(&p);
        assert!(d.shadow_gated, "the Act beat is shadow-first");
        match d.disposition {
            Disposition::Accepted { value, delta } => {
                assert_eq!(value, "candidate");
                assert!(delta > 0, "an accepted move has a positive objective delta");
            }
            other => panic!("an improvement must be accepted, got {other:?}"),
        }
    }

    #[test]
    fn a_knob_regression_is_reverted_prior_kept() {
        let opt = DeliveryOptimizer::new();
        let p = knob_proposal("mem-band", reading(400, 85, 80, 200), reading(600, 60, 40, 900));
        match opt.decide_axis(&p).disposition {
            Disposition::Reverted { kept_value } => assert_eq!(kept_value, "current"),
            other => panic!("a regression must be reverted, got {other:?}"),
        }
    }

    #[test]
    fn a_structural_block_escalates_up_never_knob_chased() {
        // Even though the axis' primary clock is lapidar, a Structure move routes
        // UP to the posture — the load-bearing knob-vs-structure line.
        let opt = DeliveryOptimizer::new();
        let mut p = knob_proposal("spot-strategy", reading(500, 70, 60, 0), reading(300, 90, 90, 0));
        p.move_kind = Move::Structure; // e.g. dropped spot_allocation_strategy on the managed NG
        match opt.decide_axis(&p).disposition {
            Disposition::Escalated { to, reason } => {
                assert_eq!(to, "posture-seven-beat");
                assert!(reason.contains("never knob-chased"));
            }
            other => panic!("a structural block must escalate UP, got {other:?}"),
        }
    }

    #[test]
    fn a_posture_owned_axis_is_deferred_not_mutated() {
        let opt = DeliveryOptimizer::new();
        let mut p = knob_proposal("posture-setpoint", reading(500, 70, 60, 0), reading(300, 90, 90, 0));
        p.tick_source = TickSource::PostureSevenBeat;
        match opt.decide_axis(&p).disposition {
            Disposition::Escalated { to, .. } => assert_eq!(to, "posture-seven-beat"),
            other => panic!("a posture-owned axis is deferred, got {other:?}"),
        }
    }

    #[test]
    fn a_tick_rolls_up_accepts_reverts_escalations_and_net_delta() {
        let opt = DeliveryOptimizer::new();
        let improve = knob_proposal("a", reading(500, 70, 60, 500), reading(400, 85, 80, 200));
        let regress = knob_proposal("b", reading(400, 85, 80, 200), reading(600, 60, 40, 900));
        let mut structural = knob_proposal("c", reading(500, 70, 60, 0), reading(300, 90, 90, 0));
        structural.move_kind = Move::Structure;
        let report = opt.tick(&[improve, regress, structural]);
        assert_eq!(report.accepts, 1);
        assert_eq!(report.reverts, 1);
        assert_eq!(report.escalations, 1);
        assert!(report.net_objective_delta > 0, "the one accept made the tick net-positive");
    }

    #[test]
    fn the_loop_is_monotone_non_regressing_net_delta_never_negative() {
        // A tick made entirely of REGRESSIONS nets exactly zero (all reverted) —
        // never negative. Efficiency compounds, never oscillates down.
        let opt = DeliveryOptimizer::new();
        let r1 = knob_proposal("a", reading(400, 85, 80, 200), reading(900, 30, 10, 2000));
        let r2 = knob_proposal("b", reading(300, 90, 90, 0), reading(800, 40, 20, 1500));
        let report = opt.tick(&[r1, r2]);
        assert_eq!(report.accepts, 0);
        assert_eq!(report.reverts, 2);
        assert_eq!(report.net_objective_delta, 0, "all regressions reverted → net zero, never negative");
    }

    #[test]
    fn every_decision_is_shadow_gated() {
        let opt = DeliveryOptimizer::new();
        let report = opt.tick(&[
            knob_proposal("a", reading(500, 70, 60, 500), reading(400, 85, 80, 200)),
            knob_proposal("b", reading(400, 85, 80, 200), reading(600, 60, 40, 900)),
        ]);
        assert!(report.decisions.iter().all(|d| d.shadow_gated), "a blind flip has no code path");
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use crate::rationale::CostRationale;
    use proptest::prelude::*;

    fn arb_reading() -> impl Strategy<Value = AxisReading> {
        (0u32..2000, 0u32..200, 0u8..=100, 0u32..5000).prop_map(|(c, p, r, w)| AxisReading {
            cost_milli: c,
            perf_score: p,
            resiliency: r,
            waste: w,
        })
    }

    proptest! {
        /// THE monotone-non-regressing law (property-tested): over ARBITRARY
        /// proposals, one tick's net objective delta is NEVER negative. An accept
        /// only ever fires on a positive-past-margin improvement; a revert or
        /// escalate contributes zero. Efficiency compounds tick by tick.
        #[test]
        fn tick_net_delta_is_never_negative(
            befores in proptest::collection::vec(arb_reading(), 0..12),
            afters in proptest::collection::vec(arb_reading(), 0..12),
            structural in proptest::collection::vec(any::<bool>(), 0..12),
        ) {
            let opt = DeliveryOptimizer::new();
            let n = befores.len().min(afters.len()).min(structural.len());
            let proposals: Vec<AxisProposal> = (0..n)
                .map(|i| AxisProposal {
                    axis: "ax",
                    link: Link::Run,
                    tick_source: TickSource::LapidarAcceptIfImproved,
                    current_value: "cur".into(),
                    proposed_value: "cand".into(),
                    before: befores[i],
                    after_shadow: afters[i],
                    rationale: CostRationale::uncontested("cand", "prop"),
                    move_kind: if structural[i] { Move::Structure } else { Move::Knob },
                })
                .collect();
            let report = opt.tick(&proposals);
            prop_assert!(report.net_objective_delta >= 0, "a tick must never regress the objective");
            // and every accept had a strictly positive delta
            for d in &report.decisions {
                if let Disposition::Accepted { delta, .. } = d.disposition {
                    prop_assert!(delta > 0, "an accept must strictly improve");
                }
            }
        }
    }
}
