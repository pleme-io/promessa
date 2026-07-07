//! # prumo — the tick-by-tick DELIVERY-CHAIN OPTIMIZER
//!
//! *prumo* (Brazilian-Portuguese: the **plumb line / plumb bob**) — the craft
//! tool that continuously finds and holds **true** (the optimum) and self-corrects
//! when perturbed. It is the plumb line for the delivery chain: every tick it
//! re-finds true — the cost+performance optimum — across every axis of
//! `BUILD → CACHE → TRANSFER → DEPLOY → RUN`, and a perturbation that regresses is
//! let to settle back (revert), never shipped. Sibling of **lapidar** (gem-cutting
//! self-tune); prumo sits ABOVE it and drives the WHOLE chain.
//!
//! ## What it is
//!
//! The continuous optimizer that composes **lapidar** (the accept-if-improved-
//! else-revert tick self-tuner) + the **camelot-posture PromessaController** (the
//! landed seven-beat) into ONE loop optimizing every delivery-chain axis each
//! tick, converging to the cost+performance optimum. Each optimization axis is a
//! **sealed typed classical/SOTA algorithm (NO ML)** wrapped in lapidar's
//! **shadow-first** accept-if-improved-else-revert. Cost-conflicts resolve by
//! **cost**; the chosen value carries its inline rationale, **loud where the
//! intuitive choice loses** (e.g. `arch=m5a floor: m5a −19% vs Graviton NOW`).
//!
//! ## The loop (one tick)
//!
//! For each axis: read the current cost/perf → take the sealed-algorithm optimum
//! → route by the **knob-vs-structure line** (a knob → lapidar accept-if-improved;
//! a structural block → escalated UP to the posture seven-beat R3, **never
//! knob-chased**) → resolve cost-conflicts by cost → apply **shadow-gated**, else
//! revert. Guarantee: **monotone-non-regressing** — the tick's net objective delta
//! is `≥ 0`; efficiency compounds, never oscillates
//! ([`optimizer::DeliveryOptimizer::tick`]).
//!
//! ## COMPOSE, do not rebuild
//!
//! prumo owns **no new algebra** — it is a *composition index* over shipped/
//! authored primitives, composed BY REFERENCE (the breathe-invariant idiom:
//! standalone workspace root, serde-only deps, `doctrine_ref` strings), never
//! forking their crates:
//!
//! - **lapidar** = `sui-supercacheci::memory::evaluate_tune` — the accept-if-
//!   improved-else-revert core ([`lapidar`], [`lapidar::LAPIDAR_DOCTRINE_REF`]).
//! - **the posture seven-beat** = `camelot-posture-controller::CamelotPostureController`
//!   (a `promessa_types::TargetController`) — the setpoint keeper + the only
//!   mutator of STRUCTURE (R3); prumo is its Decide-beat inner knob optimizer.
//! - **the config-spread** = `breathe-auction` (arch × spot × ladder × perf ×
//!   placement × interruption) — the arch cost lever ([`rationale`],
//!   [`rationale::ARCH_COST_DOCTRINE_REF`]).
//! - **breathe + breathe-invariant** = the RUN bands (multiplicative carve,
//!   grow-only storage, replica topology) + the carve/setpoint lock.
//! - **super-cache-ci + breathe-catalog** = the BUILD axes (DAG scheduling,
//!   dynamic max-jobs, arch, RAMDISK).
//!
//! ## The honesty gate is FIXED (lapidar's fixed core)
//!
//! The **tier ladder + never-round-up** and the **accept margin** are `const`,
//! not knobs ([`honesty`]): a rounded-up tier claim and a margin-dodging accept
//! have **no code path**. The [`catalog`] is the tier-honest ledger of every axis
//! — which optimize LIVE tick-by-tick (Shipped) vs authored-on-a-branch vs
//! design/LiveTODO — with forcing-function tests that CI-catch an ML axis, a
//! duplicate name, and a rounded-up tier.
//!
//! ## Tier-honest status (never rounded up)
//!
//! - **SHIPPED (this crate):** the tick loop, the accept-if-improved wrapper, the
//!   knob-vs-structure routing, the cost-conflict resolver, the fixed honesty
//!   gate, the 25-axis self-describing catalog, and the worked sealed algorithms
//!   ([`seal`]) — all pure, deterministic, and green without a cluster.
//! - **DESIGN / LiveTODO:** the Observe/Act reconcile *loop* that reads the live
//!   signals (grafana-MCP + breathe-MCP) and drives the composed actuators is the
//!   camelot-posture / autorevivy coordinator (design-stage; runs manually today)
//!   — prumo is the substrate it drives, not a second controller. Each axis' own
//!   live-loop tier is in the [`catalog`].

pub mod axis;
pub mod catalog;
pub mod honesty;
pub mod lapidar;
pub mod optimizer;
pub mod rationale;
pub mod seal;

// The load-bearing surface, re-exported at the crate root.
pub use axis::{ChainAxis, Link, SealedAlgorithm, TickSource, Tier};
pub use catalog::{count_at_tier, live_axes, AXES};
pub use honesty::{attest_tier, can_claim};
pub use lapidar::{evaluate_tune, AxisReading, TuneOutcome, TuneProposal, TUNE_MARGIN};
pub use optimizer::{AxisDecision, AxisProposal, DeliveryOptimizer, Disposition, Move, TickReport};
pub use rationale::CostRationale;

/// The `doctrine_ref`s prumo composes BY REFERENCE — named as data so a test can
/// assert the crate is a composition index (it cites its sources), not a fork.
pub const COMPOSED_DOCTRINE_REFS: [&str; 5] = [
    "lapidar = sui-supercacheci::memory::evaluate_tune (accept-if-improved-else-revert)",
    "posture = camelot-posture-controller::CamelotPostureController (the seven-beat)",
    "config-spread = breathe-auction (arch × spot × ladder × perf × placement × interruption)",
    "run-bands = breathe + breathe-invariant (carve/setpoint lock)",
    "build-axes = super-cache-ci + breathe-catalog::builder (DAG/max-jobs/arch/ramdisk)",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_crate_is_a_composition_index_not_a_fork() {
        // prumo composes five shipped/authored primitives by reference — it owns
        // no new algebra. Each doctrine_ref names a real primitive.
        assert_eq!(COMPOSED_DOCTRINE_REFS.len(), 5);
        assert!(COMPOSED_DOCTRINE_REFS.iter().any(|r| r.contains("lapidar")));
        assert!(COMPOSED_DOCTRINE_REFS.iter().any(|r| r.contains("posture")));
        assert!(COMPOSED_DOCTRINE_REFS.iter().any(|r| r.contains("breathe-auction")));
    }

    #[test]
    fn end_to_end_a_tick_composes_lapidar_and_the_seal_over_the_catalog() {
        // Smoke: feed the loop a real sealed-algorithm proposal (the arch floor)
        // and a structural block; the loop accepts the cheaper arch and escalates
        // the block — the whole composition working end to end.
        let opt = DeliveryOptimizer::new();
        let arch = seal::arch_cost_optimum("run-cost-auction", Link::Run, "arm", 123, 100, 80, 70);
        let block = seal::structural_block(
            "build-instance-family",
            Link::Build,
            "capacity_type=spot",
            "managed-NG drops spot_allocation_strategy",
        );
        let report = opt.tick(&[arch, block]);
        assert_eq!(report.accepts, 1, "the cheaper arch is accepted");
        assert_eq!(report.escalations, 1, "the structural block is escalated, not knob-chased");
        assert!(report.net_objective_delta > 0, "the tick is net-positive (monotone-non-regressing)");
    }
}
