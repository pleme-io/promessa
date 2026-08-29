//! `catalog` — the self-describing DELIVERY-CHAIN AXIS LEDGER (CATALOG
//! REFLECTION): every optimization axis across `BUILD → CACHE → TRANSFER →
//! DEPLOY → RUN` as one typed const array, each row naming its sealed algorithm,
//! its clock, its **tier-honest** live-loop maturity, the shipped primitive it
//! composes BY REFERENCE, and where its invariant is sealed.
//!
//! This IS the map's tier ledger as typed data — the honest answer to "which
//! axes optimize LIVE tick-by-tick vs shadow/design?". The forcing-function tests
//! make an ML axis, a duplicate name, a rounded-up tier, and an axis that names
//! nothing it composes all **CI-caught** — the same discipline breathe-invariant's
//! `no_dimension_claimed_but_uncarved` applies to the breathability dimensions.
//!
//! Tiers are authored to match the map ledger and rounded DOWN when in doubt
//! (never up): the live-loop counts below are asserted, so a future edit that
//! promotes a `DesignLiveTodo` to `Shipped` without the fleet catching up trips
//! [`tests::the_shipped_set_matches_the_map_ledger_exactly`].

use crate::axis::{ChainAxis, Link, SealedAlgorithm, TickSource, Tier};

/// The complete delivery-chain optimization catalog — 25 axes across 5 links.
pub const AXES: [ChainAxis; 25] = [
    // ─────────────────────────── LINK 1 — BUILD ───────────────────────────
    ChainAxis {
        link: Link::Build,
        name: "build-dag-scheduling",
        objective: "min wall-clock; don't idle cores on the critical path",
        algorithm: SealedAlgorithm::CpmListScheduling,
        tick_source: TickSource::PostureSevenBeat,
        tier: Tier::Shipped,
        composes: "nix-image/run.tlisp (6/6) + super-cache-ci-build-matrix.yml (within+across fan)",
        sealed_at: "breathe-catalog::builder::ParallelismContract (max_jobs_is_not_a_hardcoded_integer)",
    },
    ChainAxis {
        link: Link::Build,
        name: "build-max-jobs-tuner",
        objective: "saturate ANY box without over/under-fill",
        algorithm: SealedAlgorithm::BoxDetectMaxJobs,
        tick_source: TickSource::LapidarAcceptIfImproved,
        tier: Tier::Shipped,
        composes: "ni:parallel-flags (box-detect nproc × RAM headroom → --max-jobs/--cores)",
        sealed_at: "breathe-catalog::builder::ParallelismContract (auto/0/uncapped, tested)",
    },
    ChainAxis {
        link: Link::Build,
        name: "build-closure-granularity",
        objective: "max cache reuse; invalidation frontier = changed node's dependents only",
        algorithm: SealedAlgorithm::MaximalDagFineGrain,
        tick_source: TickSource::PostureSevenBeat,
        tier: Tier::AuthoredProven,
        composes: "gen-gomod per-package derivations (branch: gen/gen-gomod-m1-encoder 42-green)",
        sealed_at: "GEN-TYPED-SPEC-CONTRACT I1/I2/I3 (stale spec = CI failure)",
    },
    ChainAxis {
        link: Link::Build,
        name: "build-arch-cost",
        objective: "pick cheapest-deepest arch; image is MULTI-ARCH so it runs wherever the auction lands",
        algorithm: SealedAlgorithm::ArchCostResolve,
        tick_source: TickSource::LapidarAcceptIfImproved,
        tier: Tier::AuthoredProven,
        composes: "breathe-auction::axis::resolve_arch (cost-typed arch axis, in-flight) + shipped DegradeTier ladders",
        sealed_at: "breathe-catalog::builder::DegradeTier + ARM64/AMD64_DEGRADE_LADDER (total-order-proven)",
    },
    ChainAxis {
        link: Link::Build,
        name: "build-instance-family",
        objective: "deepest spot pool → survive reclaim; never on-demand",
        algorithm: SealedAlgorithm::CapacityOptimizedPrioritized,
        tick_source: TickSource::LapidarAcceptIfImproved,
        tier: Tier::AuthoredProven,
        composes: "breathe-catalog::builder DegradeTier ladder (shipped) — managed-NG strategy DROPPED (config-spread GAP)",
        sealed_at: "DegradeTier total-order + FORBIDDEN_ON_DEMAND_KEYS/reject_on_demand!",
    },
    ChainAxis {
        link: Link::Build,
        name: "build-ramdisk-size",
        objective: "hold eval+sandbox+store working-set in RAM; undersize→spill, oversize→steal --cores RAM",
        algorithm: SealedAlgorithm::WorkingSetRamdisk,
        tick_source: TickSource::LapidarAcceptIfImproved,
        tier: Tier::AuthoredProven,
        composes: "builder_node_group.rb ramdisk_gib:64 (size signal shipped); live MemoryBand LiveTODO",
        sealed_at: "breathe-catalog::builder::BuilderBreatheClass.ramdisk_gib >= 32 (tested)",
    },
    // ─────────────────────────── LINK 2 — CACHE ───────────────────────────
    ChainAxis {
        link: Link::Cache,
        name: "cache-tier-hit",
        objective: "max hit-ratio at min RAM cost (the memory-hierarchy admit decision)",
        algorithm: SealedAlgorithm::ContentAddressedBlake3,
        tick_source: TickSource::LapidarAcceptIfImproved,
        tier: Tier::AuthoredProven,
        composes: "sui CA+BLAKE3+no-lock (shipped) + TieredBackend keystone (branch sui/feat/tiered-backend, parity 5/5)",
        sealed_at: "key=content-hash ⇒ two-builds-one-key truly-unrep (BLAKE3 preimage)",
    },
    ChainAxis {
        link: Link::Cache,
        name: "cache-eviction-cost-cold",
        objective: "evict the coldest / cheapest-to-re-derive first",
        algorithm: SealedAlgorithm::CostColdEviction,
        tick_source: TickSource::LapidarAcceptIfImproved,
        tier: Tier::Shipped,
        composes: "sui-supercacheci::memory::classify_freshness + sui_cache::gc::collect_garbage (real GC)",
        sealed_at: "freshness::classify_freshness verdict typed; GC is a shigoto job",
    },
    ChainAxis {
        link: Link::Cache,
        name: "cache-preheat-frontier",
        objective: "pre-warm the derivations the DAG frontier will need before the miss",
        algorithm: SealedAlgorithm::DagFrontierPrefetch,
        tick_source: TickSource::ReactiveNervousSystem,
        tier: Tier::DesignLiveTodo,
        composes: "sui-supercacheci::memory::plan_precarve (derivation shipped); DAG-frontier feed + carve-drive LiveTODO",
        sealed_at: "memory::plan_precarve over MemoryForecast/AllocationPhase (predict→shadow→confirm→allocate)",
    },
    ChainAxis {
        link: Link::Cache,
        name: "cache-reclaim-durability",
        objective: "a reclaimed cache tier loses nothing",
        algorithm: SealedAlgorithm::StateInDbDurability,
        tick_source: TickSource::ReactiveNervousSystem,
        tier: Tier::AuthoredProven,
        composes: "BuilderBreatheClass.retry_on_spot_reclaim (posture shipped); CA-cache-as-checkpoint DESIGN-until-store",
        sealed_at: "state-in-DB (Postgres) ⇒ disk-loss-on-reclaim unrepresentable; loss bounded to in-flight derivation (C5)",
    },
    // ────────────────────────── LINK 3 — TRANSFER ─────────────────────────
    ChainAxis {
        link: Link::Transfer,
        name: "transfer-chunk-dedup",
        objective: "re-address coarse per-layer digests to chunk-granular so 99%-unchanged layers don't re-ship",
        algorithm: SealedAlgorithm::ChunkDedupNydus,
        tick_source: TickSource::PostureSevenBeat,
        tier: Tier::DesignLiveTodo,
        composes: "ADOPT Nydus RAFS / zstd:chunked (graduated OSS); ORAS provenance-carry schema DESIGN",
        sealed_at: "typed ORAS-referrers annotation schema (sui address + tameshi verdict travel with the chunks)",
    },
    ChainAxis {
        link: Link::Transfer,
        name: "transfer-swarm-select",
        objective: "many peer sources not one registry; rank local > peer-swarm > faucet > origin",
        algorithm: SealedAlgorithm::P2pSwarmSelect,
        tick_source: TickSource::LapidarAcceptIfImproved,
        tier: Tier::DesignLiveTodo,
        composes: "ADOPT Spegel (Kademlia-DHT, P0) / Dragonfly (CNCF Graduated); enjulho (deftransfer-env) + Viggy optimizer DESIGN",
        sealed_at: "typed (deftransfer-env) enjulho surface referencing sui/zot/AUTOBUMP (one merged config)",
    },
    ChainAxis {
        link: Link::Transfer,
        name: "transfer-pacing",
        objective: "never trip the registry's secondary rate-limit on a burst pull",
        algorithm: SealedAlgorithm::SambaLeakyBucket,
        tick_source: TickSource::ReactiveNervousSystem,
        tier: Tier::AuthoredProven,
        composes: "samba LeakyBucket (shipped, rate from X-RateLimit-Limit); wired into transfer via the optimizer DESIGN",
        sealed_at: "samba bucket shipped; quota-pct the afinar-tunable knob",
    },
    ChainAxis {
        link: Link::Transfer,
        name: "transfer-leg-a-substitutor",
        objective: "same content-addressing for build AND transfer, literally (leg A: store → runner)",
        algorithm: SealedAlgorithm::P2pSwarmSelect,
        tick_source: TickSource::PostureSevenBeat,
        tier: Tier::DesignLiveTodo,
        composes: "sui-store::Substitutor/BinaryCacheStore (leg-A base shipped); P2P rides DRAFT maré/enxame/iroh (zero-code)",
        sealed_at: "sui-store multi-cache-in-order (Postgres) shipped; DHT peers = more caches, same NarInfo sig check",
    },
    // ─────────────────────────── LINK 4 — DEPLOY ──────────────────────────
    ChainAxis {
        link: Link::Deploy,
        name: "deploy-native-arch",
        objective: "ship the multi-arch image on a NATIVE-arch runner (no QEMU)",
        algorithm: SealedAlgorithm::NativeArchBuild,
        tick_source: TickSource::PostureSevenBeat,
        tier: Tier::Shipped,
        composes: "image-push.yml native runner (rio-builder x86_64), no-QEMU matrix; AUTOBUMP emits :arm64+:amd64",
        sealed_at: "native-arch build seal (fixed the hanabi exec-format-error: amd64 tags were aarch64 binaries)",
    },
    ChainAxis {
        link: Link::Deploy,
        name: "deploy-exact-pin",
        objective: "\"what is running?\" answered by reading the committed manifest, never a moving tag",
        algorithm: SealedAlgorithm::SortableExactTag,
        tick_source: TickSource::PostureSevenBeat,
        tier: Tier::Shipped,
        composes: "image-push.yml emits <arch>-r<run>-<sha> (r<run> numerically orderable, sha traceable)",
        sealed_at: "the exact tag lives IN GIT; a moving tag as a deploy source is the forbidden anti-pattern",
    },
    ChainAxis {
        link: Link::Deploy,
        name: "deploy-subscribe",
        objective: "an env auto-bumps to the newest known-good exact tag as a git commit",
        algorithm: SealedAlgorithm::FluxImageAutomation,
        tick_source: TickSource::PostureSevenBeat,
        tier: Tier::Shipped,
        composes: "Flux image-reflector+automation LIVE on rio (hanabi + pangea-operator subscribe, pin highest run)",
        sealed_at: "an autobump IS a git commit of an exact tag — GitOps-native, auditable, revertable",
    },
    // ──────────────────────────── LINK 5 — RUN ────────────────────────────
    ChainAxis {
        link: Link::Run,
        name: "run-mem-cpu-band",
        objective: "carve RAM/cores to the setpoint — cost DOWN + headroom before the OOM/CFS cliff (dual-purpose)",
        algorithm: SealedAlgorithm::MultiplicativeBandCarve,
        tick_source: TickSource::LapidarAcceptIfImproved,
        tier: Tier::Shipped,
        composes: "breathe mem/cpu bands LIVE on rio (band re-tune each tick, accept-if-improved-else-revert)",
        sealed_at: "breathe-invariant C1 carved-by-a-band + C2 setpoint (basis-points, parse-rejected to (0,1)) + C6 dual-purpose",
    },
    ChainAxis {
        link: Link::Run,
        name: "run-storage-band",
        objective: "provision-minimal + grow-on-demand — no 155GiB-holding-890MiB waste",
        algorithm: SealedAlgorithm::GrowOnlyPredictive,
        tick_source: TickSource::LapidarAcceptIfImproved,
        tier: Tier::AuthoredProven,
        composes: "breathe StorageBand (main 42b96ae, 309 green); live carve gated on per-volume accounting CSI (LiveTODO)",
        sealed_at: "breathe_carve_never_over_provisions theorem (carve output never OverProvisioned — CI forcing-function, C1)",
    },
    ChainAxis {
        link: Link::Run,
        name: "run-replica-band",
        objective: "floor-2 HA + algorithmic scale; scale-down-idle (topology picks the algorithm)",
        algorithm: SealedAlgorithm::ReplicaTopologyScale,
        tick_source: TickSource::LapidarAcceptIfImproved,
        tier: Tier::AuthoredProven,
        composes: "breathe_control::replica 4-arm axis (topology invariants tested); live kanchi discovery + failover DESIGN",
        sealed_at: "master_slave_scales_read_replicas_but_never_the_primary (tested); odd-quorum-never-cross-majority",
    },
    ChainAxis {
        link: Link::Run,
        name: "run-database-matrix",
        objective: "right-size engine caches + failover-safe replicas + never-starve pool",
        algorithm: SealedAlgorithm::MultiplicativeBandCarve,
        tick_source: TickSource::LapidarAcceptIfImproved,
        tier: Tier::DesignLiveTodo,
        composes: "breathe-catalog::db_matrix (MySQL+Neo4j typed = 2/5); Postgres/Redis/Mongo DESIGN (honestly pending-breathe)",
        sealed_at: "per-engine knobs as AppParam (architecture-aware-engine carve)",
    },
    ChainAxis {
        link: Link::Run,
        name: "run-cost-auction",
        objective: "min $ — always best-available spot, never on-demand, cheapest-deepest arch",
        algorithm: SealedAlgorithm::CapacityOptimizedPrioritized,
        tick_source: TickSource::ReactiveNervousSystem,
        tier: Tier::AuthoredProven,
        composes: "reject_on_demand! + DegradeTier ladder (never-on-demand SHIPPED); live auction plan_memory_auction LiveTODO",
        sealed_at: "reject_on_demand! + FORBIDDEN_ON_DEMAND_KEYS (on-demand parse-rejected at the Ruby boundary — truly-unrep)",
    },
    ChainAxis {
        link: Link::Run,
        name: "run-scale-to-zero",
        objective: "dormant → near-free in the dark; wake predictively",
        algorithm: SealedAlgorithm::ScaleToZero,
        tick_source: TickSource::ReactiveNervousSystem,
        tier: Tier::Shipped,
        composes: "cluster-autoscaler 0→N (proven on one production cluster) + predictive-start",
        sealed_at: "breathe lifecycle-breath (AtFloor/Holding posture)",
    },
    ChainAxis {
        link: Link::Run,
        name: "run-leak-guard",
        objective: "no orphans/residue/drift in 9 classes",
        algorithm: SealedAlgorithm::LeakDetectionFsm,
        tick_source: TickSource::ReactiveNervousSystem,
        tier: Tier::AuthoredProven,
        composes: "the posture controller dispatches autorevivy MaintenanceJob leak-guards by typed tag — REFERENCED, never edited (owned by autorevivy)",
        sealed_at: "posture composes them (pure core shipped, 14 tests); closed loop LiveTODO",
    },
    ChainAxis {
        link: Link::Run,
        name: "run-retirada-drain",
        objective: "survive reclaim / replica-down without a fatal loop",
        algorithm: SealedAlgorithm::CapacityRebalanceDrain,
        tick_source: TickSource::ReactiveNervousSystem,
        tier: Tier::DesignLiveTodo,
        composes: "Spot::InterruptionHandler (pangea-spot skeleton, shippable/opt-in); drain agent + NATS reclaim-publish LiveTODO",
        sealed_at: "capacity-rebalance + 2-min-warning drain-ahead + idempotent re-dispatch (RETIRADA_LIVETODO — never rounded up)",
    },
];

/// Count the catalog axes at a given tier — the tier-honest live-loop split.
#[must_use]
pub fn count_at_tier(tier: Tier) -> usize {
    AXES.iter().filter(|a| a.tier == tier).count()
}

/// The axes that optimize LIVE tick-by-tick today (Shipped) — the honest "what
/// runs now" set.
#[must_use]
pub fn live_axes() -> Vec<&'static str> {
    AXES.iter().filter(|a| a.optimizes_live()).map(|a| a.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::AlgorithmFamily;
    use crate::honesty::{attest_tier, can_claim};

    #[test]
    fn catalog_has_every_link_covered() {
        for link in Link::ALL {
            assert!(
                AXES.iter().any(|a| a.link == link),
                "link {} must have at least one optimization axis",
                link.as_str()
            );
        }
    }

    #[test]
    fn every_axis_name_is_globally_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for a in &AXES {
            assert!(seen.insert(a.name), "axis name collision: {}", a.name);
        }
        assert_eq!(seen.len(), AXES.len());
    }

    #[test]
    fn no_axis_is_ml_the_algorithmic_prowess_seal() {
        // /algorithmic-prowess-seal hard constraint: NO ML. Every axis' sealed
        // algorithm is classical/SOTA — an ML axis is CI-caught (and unrepresentable).
        for a in &AXES {
            assert!(!a.algorithm.is_ml(), "{}: sealed algorithm must be classical/SOTA, never ML", a.name);
            assert!(!a.algorithm.family().is_ml(), "{}: family must not be ML", a.name);
        }
        // and the family enum genuinely has no ML arm.
        assert!(!AlgorithmFamily::GreedyHillClimb.is_ml());
    }

    #[test]
    fn every_axis_names_what_it_composes_and_where_it_is_sealed() {
        // CATALOG REFLECTION: a row that composes nothing (a fork) or names no
        // sealed border is refused — the same discipline as breathe-invariant.
        for a in &AXES {
            assert!(!a.composes.is_empty(), "{}: must name what it composes by reference", a.name);
            assert!(!a.sealed_at.is_empty(), "{}: must name where its invariant is sealed", a.name);
            assert!(!a.objective.is_empty(), "{}: must state its objective", a.name);
        }
    }

    #[test]
    fn every_tick_source_is_one_of_the_three_clocks() {
        for a in &AXES {
            assert!(
                TickSource::ALL.contains(&a.tick_source),
                "{}: tick source must be one of the three clocks",
                a.name
            );
        }
    }

    #[test]
    fn the_shipped_set_matches_the_map_ledger_exactly() {
        // The tier-honest ledger, LOCKED. A future edit that rounds a
        // DesignLiveTodo up to Shipped (without the fleet catching up) trips this.
        // Counts authored to match the map §7 ledger, rounded DOWN when in doubt.
        assert_eq!(count_at_tier(Tier::Shipped), 8, "8 axes optimize LIVE tick-by-tick today");
        assert_eq!(count_at_tier(Tier::AuthoredProven), 11, "11 authored/branch-proven, not yet fleet-live");
        assert_eq!(count_at_tier(Tier::DesignLiveTodo), 6, "6 design/LiveTODO");
        assert_eq!(count_at_tier(Tier::Shipped) + count_at_tier(Tier::AuthoredProven) + count_at_tier(Tier::DesignLiveTodo), AXES.len());
    }

    #[test]
    fn the_shipped_axes_are_the_expected_named_ones() {
        // Name the 8 live axes explicitly so the honest "what runs now" set is
        // pinned, not just a count.
        let mut live = live_axes();
        live.sort_unstable();
        assert_eq!(
            live,
            vec![
                "build-dag-scheduling",
                "build-max-jobs-tuner",
                "cache-eviction-cost-cold",
                "deploy-exact-pin",
                "deploy-native-arch",
                "deploy-subscribe",
                "run-mem-cpu-band",
                "run-scale-to-zero",
            ]
        );
    }

    #[test]
    fn every_row_attests_its_own_tier_never_rounds_up() {
        // The honesty gate: attest_tier is the identity, and a row may claim
        // exactly its evidence tier (never above). This is the never-round-up law
        // applied to the catalog itself.
        for a in &AXES {
            assert_eq!(attest_tier(a.tier), a.tier, "{}: attested tier must equal the const tier", a.name);
            assert!(can_claim(a.tier, a.tier), "{}: a row may claim its own evidence tier", a.name);
        }
    }

    #[test]
    fn only_lapidar_axes_are_knob_tuned_the_rest_route_up() {
        // The knob-vs-structure line, catalog-wide: a lapidar axis is a knob; a
        // posture/reactive axis is NOT handed to accept-if-improved (it routes up).
        for a in &AXES {
            assert_eq!(
                a.is_lapidar_knob(),
                a.tick_source == TickSource::LapidarAcceptIfImproved,
                "{}: knob-tuned iff on the lapidar clock",
                a.name
            );
        }
    }
}
