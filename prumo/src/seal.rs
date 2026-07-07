//! `seal` — worked SEALED ALGORITHMS that feed the tick loop, proving the
//! optimizer composes real classical/SOTA methods (NO ML) end-to-end.
//!
//! The optimizer's [`tick`](crate::optimizer::DeliveryOptimizer::tick) is general:
//! it takes an [`AxisProposal`] from ANY sealed algorithm (in its home crate) and
//! applies the lapidar accept-if-improved wrapper + the knob-vs-structure routing
//! + the honesty gate. This module ships three concrete producers so the loop is
//! exercised against genuinely-different sealed algorithms:
//!
//! | producer | sealed algorithm | family | composes (doctrine_ref) |
//! |---|---|---|---|
//! | [`arch_cost_optimum`] | arch cost resolve (cheaper-of over a live signal) | greedy-hill-climb | breathe-auction::axis::resolve_arch |
//! | [`multiplicative_band_optimum`] | `⌈used/setpoint⌉` band carve | control-theory | breathe multiplicative-band carve + breathe-invariant |
//! | [`structural_block`] | (none — a step-function block) | escalate | the posture seven-beat R3 |
//!
//! Each is a pure fold over declared inputs; the live-signal read + the actuator
//! are the composed home crate's (breathe MCP / afinar), never re-implemented here.

use std::fmt;

use crate::axis::{Link, TickSource};
use crate::lapidar::AxisReading;
use crate::optimizer::{AxisProposal, Move};
use crate::rationale::CostRationale;

/// A typed MiB quantity label — TYPED EMISSION surface: the `<n>Mi` k8s quantity
/// string is rendered by a `Display` impl (`write!`), never by free `format!()`.
struct Mib(u32);

impl fmt::Display for Mib {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}Mi", self.0)
    }
}

/// **Arch cost resolve** — the map's BUILD/RUN arch axis. Resolve arm-vs-x86 by
/// effective spot cost (cheaper wins; a tie goes to arm, the aspirational push),
/// emitting the loud-where-abnormal [`CostRationale`]. Because AUTOBUMP emits a
/// MULTI-ARCH image, arch is a FREE cost lever: the same call yields the other
/// arch the moment pricing crosses (self-adjusting, no re-decision). A knob move
/// on the lapidar clock. Composes `breathe-auction::axis::resolve_arch` by
/// reference.
///
/// `arm_milli` / `x86_milli` are effective costs (spot $/hr ÷ per-arch throughput)
/// in integer milli-units. `perf_score`/`resiliency` are the shared reading axes
/// the objective folds in.
#[must_use]
pub fn arch_cost_optimum(
    axis: &'static str,
    link: Link,
    current_arch: &str,
    arm_milli: u32,
    x86_milli: u32,
    perf_score: u32,
    resiliency: u8,
) -> AxisProposal {
    // Cost-conflict resolves by COST; arm is the aspirational option.
    let rationale = CostRationale::resolve_by_cost("arch", "arm", arm_milli, "x86", x86_milli);
    let chosen_milli = arm_milli.min(x86_milli);
    let current_milli = if current_arch == "arm" { arm_milli } else { x86_milli };
    AxisProposal {
        axis,
        link,
        tick_source: TickSource::LapidarAcceptIfImproved,
        current_value: current_arch.to_string(),
        proposed_value: rationale.chosen.clone(),
        // before: the reading under the current arch's cost.
        before: AxisReading { cost_milli: current_milli, perf_score, resiliency, waste: 0 },
        // after (shadow): the reading under the chosen (cheaper-or-equal) arch.
        after_shadow: AxisReading { cost_milli: chosen_milli, perf_score, resiliency, waste: 0 },
        rationale,
        move_kind: Move::Knob,
    }
}

/// **Multiplicative-band carve** — the map's RUN mem/cpu band axis. Carve the
/// limit to the utilization setpoint: `carve = ⌈used × 100 / setpoint_pct⌉`,
/// clamped to never go below the startup-observed proven floor (never tighten
/// below what the workload was *seen* to need). Cost DOWN + headroom before the
/// OOM/CFS cliff (dual-purpose). A knob move on the lapidar clock (band re-tune
/// each tick, accept-if-improved-else-revert). Composes breathe's multiplicative-
/// band carve + breathe-invariant's carve law by reference.
///
/// The waste axis is the reclaimed slack (`current_limit − carved`), which the
/// improvement drives down; cost scales with the carved limit.
#[must_use]
pub fn multiplicative_band_optimum(
    axis: &'static str,
    used_mib: u32,
    setpoint_pct: u8,
    floor_mib: u32,
    current_limit_mib: u32,
    cost_per_gib_milli: u32,
) -> AxisProposal {
    let setpoint = u32::from(setpoint_pct.clamp(1, 100));
    // ⌈used × 100 / setpoint⌉ — the multiplicative band carve.
    let raw = (u64::from(used_mib) * 100).div_ceil(u64::from(setpoint));
    let carved = u32::try_from(raw).unwrap_or(u32::MAX).max(floor_mib);
    // cost/waste for a given limit (integer, deterministic).
    let reading = |limit: u32| -> AxisReading {
        let cost = u32::try_from(u64::from(limit) * u64::from(cost_per_gib_milli) / 1024)
            .unwrap_or(u32::MAX);
        let waste = limit.saturating_sub(used_mib);
        AxisReading { cost_milli: cost, perf_score: 100, resiliency: 80, waste }
    };
    AxisProposal {
        axis,
        link: Link::Run,
        tick_source: TickSource::LapidarAcceptIfImproved,
        current_value: Mib(current_limit_mib).to_string(),
        proposed_value: Mib(carved).to_string(),
        before: reading(current_limit_mib),
        after_shadow: reading(carved),
        rationale: CostRationale::uncontested(
            "band-carve",
            "⌈used/setpoint⌉ down to the proven floor — cost down + headroom before the cliff",
        ),
        move_kind: Move::Knob,
    }
}

/// **A structural block** — a step-function a knob can NEVER cross (BestEffort
/// QoS, an RBAC edge, a missing dependency, a dropped `spot_allocation_strategy`
/// on a managed node-group — the config-spread GAP). It is emitted as a
/// [`Move::Structure`] so the tick loop ESCALATES it UP to the posture seven-beat
/// (R3), never knob-chases it. The readings are irrelevant (a knob can't move it);
/// the rationale names the block.
#[must_use]
pub fn structural_block(
    axis: &'static str,
    link: Link,
    current_value: &str,
    block_reason: &str,
) -> AxisProposal {
    let flat = AxisReading { cost_milli: 0, perf_score: 0, resiliency: 0, waste: 0 };
    AxisProposal {
        axis,
        link,
        tick_source: TickSource::LapidarAcceptIfImproved,
        current_value: current_value.to_string(),
        proposed_value: current_value.to_string(),
        before: flat,
        after_shadow: flat,
        rationale: CostRationale::uncontested(current_value, block_reason),
        move_kind: Move::Structure,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimizer::{DeliveryOptimizer, Disposition};

    #[test]
    fn arch_optimum_flips_the_floor_to_x86_loudly_and_the_loop_accepts_it() {
        // The map's floor: Graviton (arm) 123 vs m5a (x86) 100 → x86 wins on cost,
        // loud rationale, and a workload currently on arm gets an accepted flip.
        let p = arch_cost_optimum("build-arch", Link::Build, "arm", 123, 100, 80, 70);
        assert_eq!(p.proposed_value, "x86");
        assert!(p.rationale.abnormal, "arm losing at the floor is loud");
        assert_eq!(p.rationale.advantage_pct, -19);
        // The loop accepts the cheaper arch (before cost 123 → after 100 is a win).
        let opt = DeliveryOptimizer::new();
        match opt.decide_axis(&p).disposition {
            Disposition::Accepted { value, delta } => {
                assert_eq!(value, "x86");
                assert!(delta > 0);
            }
            other => panic!("the cheaper arch must be accepted, got {other:?}"),
        }
    }

    #[test]
    fn arch_optimum_keeps_arm_when_arm_is_cheaper_quietly() {
        // Builder: arm 63 vs x86 100 → arm wins quietly; a workload already on arm
        // sees no cost change → the loop reverts (no churn), staying on arm.
        let p = arch_cost_optimum("build-arch", Link::Build, "arm", 63, 100, 90, 80);
        assert_eq!(p.proposed_value, "arm");
        assert!(!p.rationale.abnormal);
        let opt = DeliveryOptimizer::new();
        // before == after (already on the cheaper arm) → no improvement → revert.
        assert!(matches!(
            opt.decide_axis(&p).disposition,
            Disposition::Reverted { .. }
        ));
    }

    #[test]
    fn band_carve_reclaims_waste_and_the_loop_accepts_it() {
        // used 4000 MiB, 80% setpoint → carve ⌈4000/0.8⌉ = 5000 MiB; a current
        // 12000 MiB limit is wildly over-carved — reclaim 7000 MiB of waste.
        let p = multiplicative_band_optimum("mem-band", 4000, 80, 2000, 12000, 100);
        assert_eq!(p.proposed_value, "5000Mi");
        assert!(p.after_shadow.waste < p.before.waste, "carve reclaims waste");
        assert!(p.after_shadow.cost_milli < p.before.cost_milli, "carve lowers cost");
        let opt = DeliveryOptimizer::new();
        match opt.decide_axis(&p).disposition {
            Disposition::Accepted { value, delta } => {
                assert_eq!(value, "5000Mi");
                assert!(delta > 0);
            }
            other => panic!("a waste-reclaiming carve must be accepted, got {other:?}"),
        }
    }

    #[test]
    fn band_carve_never_tightens_below_the_proven_floor() {
        // used 100 MiB, 80% setpoint → ⌈125⌉, but the proven floor is 2000 → clamp.
        let p = multiplicative_band_optimum("mem-band", 100, 80, 2000, 3000, 100);
        assert_eq!(p.proposed_value, "2000Mi", "never carve below the startup-observed floor");
    }

    #[test]
    fn a_dropped_spot_strategy_is_a_structural_block_that_escalates() {
        // The config-spread GAP: EksDrillNodeGroup takes capacity_type but no
        // spot_allocation_strategy — a STRUCTURE the loop escalates, never knob-chases.
        let p = structural_block(
            "spot-strategy",
            Link::Run,
            "capacity_type=spot",
            "managed-NG drops spot_allocation_strategy — structural, escalate to R3",
        );
        let opt = DeliveryOptimizer::new();
        assert!(matches!(
            opt.decide_axis(&p).disposition,
            Disposition::Escalated { to: "posture-seven-beat", .. }
        ));
    }
}
