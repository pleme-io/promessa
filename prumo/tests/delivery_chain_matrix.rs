//! The delivery-chain VERIFICATION MATRIX — the forcing function (CLOSED-LOOP
//! MASS-SYNTHESIS rule 1). One matrix over every catalog axis: failures aggregate
//! before the assert so one run reports EVERY broken axis, not just the first. A
//! new axis that lands without a valid (no-ML, tier-honest, composed) row, or a
//! loop that regresses the objective, fails the build.

use prumo::axis::{Link, TickSource};
use prumo::optimizer::{DeliveryOptimizer, Disposition, Move};
use prumo::{seal, AXES};

#[test]
fn every_catalog_axis_is_a_valid_sealed_composed_no_ml_row() {
    let mut failures: Vec<String> = Vec::new();
    for a in &AXES {
        if a.algorithm.is_ml() {
            failures.push(format!("{}: sealed algorithm is ML (forbidden)", a.name));
        }
        if a.composes.is_empty() {
            failures.push(format!("{}: composes nothing (a fork, not a composition)", a.name));
        }
        if a.sealed_at.is_empty() {
            failures.push(format!("{}: names no sealed invariant border", a.name));
        }
        if a.objective.is_empty() {
            failures.push(format!("{}: states no objective", a.name));
        }
        if !TickSource::ALL.contains(&a.tick_source) {
            failures.push(format!("{}: tick source is not one of the three clocks", a.name));
        }
    }
    assert!(
        failures.is_empty(),
        "{} axis(es) failed the matrix:\n  - {}",
        failures.len(),
        failures.join("\n  - ")
    );
}

#[test]
fn the_matrix_covers_every_link_in_the_chain() {
    // A link with no optimization axis is a hole in the chain — CI-caught.
    let mut missing: Vec<&str> = Vec::new();
    for link in Link::ALL {
        if !AXES.iter().any(|a| a.link == link) {
            missing.push(link.as_str());
        }
    }
    assert!(missing.is_empty(), "links with no axis: {}", missing.join(", "));
    assert!(AXES.len() >= 20, "the chain has at least 20 optimization axes, got {}", AXES.len());
}

#[test]
fn a_full_tick_over_one_worked_axis_per_link_is_monotone_non_regressing() {
    // Drive the loop with a genuinely-different sealed algorithm per link and
    // prove the whole tick never regresses (the load-bearing guarantee).
    let opt = DeliveryOptimizer::new();
    let proposals = vec![
        // BUILD: arch cost resolve — the floor flips to x86 (cheaper), accepted.
        seal::arch_cost_optimum("build-arch-cost", Link::Build, "arm", 123, 100, 80, 70),
        // CACHE: a structural block (a missing dependency) — escalated, not chased.
        seal::structural_block("cache-tier-hit", Link::Cache, "l1=redis", "TieredBackend not yet live"),
        // TRANSFER: pacing knob that does NOT improve — reverted (no churn).
        {
            let mut p = seal::multiplicative_band_optimum("transfer-pacing", 4000, 80, 2000, 5000, 100);
            p.link = Link::Transfer;
            p.tick_source = TickSource::LapidarAcceptIfImproved;
            // before == after (already at the carve) → no improvement → revert.
            p.before = p.after_shadow;
            p
        },
        // DEPLOY: (structural — deploy is posture-owned) escalated.
        seal::structural_block("deploy-subscribe", Link::Deploy, "pinned", "Flux automation owns the commit"),
        // RUN: mem band carve reclaiming waste — accepted.
        seal::multiplicative_band_optimum("run-mem-cpu-band", 4000, 80, 2000, 12000, 100),
    ];
    let report = opt.tick(&proposals);
    assert!(report.net_objective_delta >= 0, "a full-chain tick must never regress the objective");
    assert!(report.accepts >= 2, "the cheaper arch + the waste-reclaiming carve are accepted");
    assert!(report.escalations >= 2, "the structural blocks are escalated, not knob-chased");
    // every decision is shadow-gated — a blind flip has no code path.
    assert!(report.decisions.iter().all(|d| d.shadow_gated));
}

#[test]
fn a_structural_block_is_never_knob_chased_matrix_wide() {
    // For every link, a Structure move escalates UP — the knob-vs-structure line
    // holds regardless of which axis raised it.
    let opt = DeliveryOptimizer::new();
    for link in Link::ALL {
        let p = seal::structural_block("x", link, "cur", "step-function block");
        assert!(
            matches!(p.move_kind, Move::Structure),
            "structural_block must emit a Structure move"
        );
        assert!(
            matches!(
                opt.decide_axis(&p).disposition,
                Disposition::Escalated { to: "posture-seven-beat", .. }
            ),
            "a structural block on {} must escalate to the posture, never knob-chase",
            link.as_str()
        );
    }
}

#[test]
fn the_maps_arch_floor_example_flows_through_the_whole_loop() {
    // End-to-end: the map's vocal cost-conflict — the aspirational arm loses at
    // the floor to x86 by −19% — is produced by the sealed algorithm, and the
    // loop accepts the cheaper arch with the LOUD inline rationale intact. (The
    // rationale::… unit test reproduces the exact m5a/Graviton instance labels;
    // the seal helper uses the generic arm/x86 arch labels — same −19% loud loss.)
    let opt = DeliveryOptimizer::new();
    let p = seal::arch_cost_optimum("run-cost-auction", Link::Run, "arm", 123, 100, 80, 70);
    assert_eq!(p.rationale.why, "arch=x86 floor: x86 −19% vs arm NOW");
    assert!(p.rationale.abnormal, "arm losing at the floor is loud");
    assert_eq!(p.rationale.advantage_pct, -19);
    let d = opt.decide_axis(&p);
    assert!(matches!(d.disposition, Disposition::Accepted { .. }), "the cheaper arch is accepted");
    assert_eq!(d.rationale.why, "arch=x86 floor: x86 −19% vs arm NOW", "the rationale travels with the decision");
}
