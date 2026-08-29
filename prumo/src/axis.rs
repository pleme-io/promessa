//! The typed vocabulary of the delivery chain — the LINKS, the three TICK
//! SOURCES (clocks), the sealed ALGORITHM families (classical + SOTA, **NO ML**),
//! the TIER honesty ladder, and the [`ChainAxis`] record that binds them.
//!
//! A delivery chain is the five-tuple `Controller` composed with itself along the
//! sequence `BUILD → CACHE → TRANSFER → DEPLOY → RUN` (OPERATING-THEORY's BUILD +
//! DELIVER + RUN axes made concrete). Every axis below is ONE point in
//! `Link × SealedAlgorithm × TickSource × Tier`, and the whole set is the
//! self-describing [`crate::catalog`].
//!
//! ── /algorithmic-prowess-seal (best-fit, NO ML) ──
//! Every axis names a sealed CLASSICAL or SOTA algorithm ([`SealedAlgorithm`])
//! drawn from the top of the smallest-sufficient ladder — CPM/list-scheduling,
//! content-addressing + Merkle + BLAKE3, capacity-optimized auction,
//! multiplicative-band carve + least-squares grow, accept-if-improved hill-climb,
//! idempotent-retry. [`AlgorithmFamily::is_ml`] is `false` for every family by
//! construction: an ML axis is *unrepresentable* here (there is no ML arm), and
//! the matrix forcing-function refuses one.

use serde::{Deserialize, Serialize};

// ─────────────────────────────────────────────────────────────────────────────
// LINK — the five delivery-chain stages
// ─────────────────────────────────────────────────────────────────────────────

/// One stage of the delivery chain. The sequence is fixed: source flows
/// `Build → Cache → Transfer → Deploy → Run`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Link {
    /// super-cache-ci: source → N image tarballs (the Nix/gen derivation DAG).
    Build,
    /// the tiered content-addressed super-cache (Redis L1 → Postgres L2 → object L3).
    Cache,
    /// image → node; store → runner (OCI/NAR bytes in flight).
    Transfer,
    /// AUTOBUMP: built → running, across the registry boundary.
    Deploy,
    /// the posture: observed → desired, forever (the running workload).
    Run,
}

impl Link {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Cache => "cache",
            Self::Transfer => "transfer",
            Self::Deploy => "deploy",
            Self::Run => "run",
        }
    }

    /// The five links in chain order — the fixed sequence the optimizer walks.
    pub const ALL: [Link; 5] = [Self::Build, Self::Cache, Self::Transfer, Self::Deploy, Self::Run];
}

// ─────────────────────────────────────────────────────────────────────────────
// TICK SOURCE — the three clocks that drive the one loop
// ─────────────────────────────────────────────────────────────────────────────

/// Which of the three clocks drives an axis. The whole optimizer is these three
/// clocks driving ONE setpoint set (map §0): lapidar tunes KNOBS, the posture
/// seven-beat keeps the SETPOINT and owns STRUCTURE, the reactive clock is the
/// interrupt lane + the escalation trigger.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum TickSource {
    /// lapidar `accept-if-improved-else-revert` — tunes a KNOB (R2 rung: tier
    /// sizes, band setpoints, samba quota-pct, prefetch, ladder rank). The inner
    /// optimizer nested in the posture's Decide beat.
    LapidarAcceptIfImproved,
    /// the posture PromessaController seven-beat — keeps the whole
    /// posture's SETPOINT set and is the ONLY thing that mutates STRUCTURE (R3:
    /// QoS, RBAC, a missing dependency, a dropped `spot_allocation_strategy`).
    PostureSevenBeat,
    /// the reactive nervous-system clock — the fast INTERRUPT lane between cron
    /// ticks (spot-reclaim notice, 429 rate-limit, band `phase:Error`) and the
    /// seam that forces a Decide UP to R3 on thrash.
    ReactiveNervousSystem,
}

impl TickSource {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LapidarAcceptIfImproved => "lapidar-accept-if-improved",
            Self::PostureSevenBeat => "posture-seven-beat",
            Self::ReactiveNervousSystem => "reactive-nervous-system",
        }
    }

    /// Is this axis tuned LIVE by lapidar's hill-climb (a knob), as opposed to
    /// owned by the posture (structure) or the interrupt lane? Only the lapidar
    /// clock is handed to [`crate::lapidar::evaluate_tune`] in the tick loop.
    #[must_use]
    pub const fn is_lapidar_knob(self) -> bool {
        matches!(self, Self::LapidarAcceptIfImproved)
    }

    pub const ALL: [TickSource; 3] = [
        Self::LapidarAcceptIfImproved,
        Self::PostureSevenBeat,
        Self::ReactiveNervousSystem,
    ];
}

// ─────────────────────────────────────────────────────────────────────────────
// ALGORITHM FAMILY + the sealed per-axis algorithm — /algorithmic-prowess-seal
// ─────────────────────────────────────────────────────────────────────────────

/// The classical/SOTA algorithm FAMILY an axis' sealed algorithm belongs to.
/// **There is no ML family** — [`is_ml`](AlgorithmFamily::is_ml) is `false` for
/// every arm, so an ML axis is unrepresentable (the /algorithmic-prowess-seal
/// hard constraint: max algorithmic prowess, the top rungs of the smallest-
/// sufficient ladder BELOW ML; brilliance, never a black box).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum AlgorithmFamily {
    /// DAG scheduling / list-scheduling (CPM, HLFET, topo-order).
    DagScheduling,
    /// Content-addressing + Merkle structures (BLAKE3 tree-hash, dedup).
    ContentAddressing,
    /// Cache-replacement priority (LRU-cost, cost-cold ranking).
    CacheReplacement,
    /// Combinatorial / capacity auction (capacity-optimized[-prioritized] bid).
    Auction,
    /// Control theory (multiplicative-band carve, PI setpoint, reduce-τ-first).
    ControlTheory,
    /// Greedy / hill-climb (accept-if-improved coordinate descent — lapidar).
    GreedyHillClimb,
    /// Least-squares / online regression (grow-only fill-velocity forecast).
    LeastSquares,
    /// Rate limiting (leaky-bucket / token-bucket pacing — samba).
    RateLimiting,
    /// Idempotent-retry + resumable DAG (retirada drain, straggler reap).
    IdempotentRetry,
    /// Convex / LP (bounded resource packing).
    ConvexLp,
}

impl AlgorithmFamily {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DagScheduling => "dag-scheduling",
            Self::ContentAddressing => "content-addressing",
            Self::CacheReplacement => "cache-replacement",
            Self::Auction => "auction",
            Self::ControlTheory => "control-theory",
            Self::GreedyHillClimb => "greedy-hill-climb",
            Self::LeastSquares => "least-squares",
            Self::RateLimiting => "rate-limiting",
            Self::IdempotentRetry => "idempotent-retry",
            Self::ConvexLp => "convex-lp",
        }
    }

    /// **The NO-ML seal.** Always `false`: every family is a classical/SOTA
    /// algorithm from the top of the smallest-sufficient ladder, never a model.
    /// The matrix forcing-function asserts this for every catalog axis.
    #[must_use]
    pub const fn is_ml(self) -> bool {
        false
    }
}

/// A sealed per-axis algorithm — the specific classical/SOTA method the axis
/// optimizes with, plus the family it belongs to. Each is a small closed value:
/// the invariant it seals is documented at its catalog row, and the illegal
/// choice (an ML method) has no arm here.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SealedAlgorithm {
    // ── BUILD ──
    /// CPM + list-scheduling, critical-path-first (HLFET priority). Optimal is
    /// NP-hard; longest-remaining-path is the best deterministic approx.
    CpmListScheduling,
    /// box-detect + working-set-aware `--max-jobs`/`--cores` derivation (the
    /// landed dynamic tuner; a hardcoded value that under-fills is the anti-pattern).
    BoxDetectMaxJobs,
    /// maximal DAG fine-graining (per-package gen-gomod derivations) — the
    /// invalidation frontier is the changed node's dependents only.
    MaximalDagFineGrain,
    /// arch cost resolve — evaluate arm-vs-x86 effective spot cost, land cheaper
    /// (multi-arch image makes arch a free cost lever; self-adjusting).
    ArchCostResolve,
    /// working-set RAMDISK sizing — hold eval+sandbox+store hot set in RAM
    /// (never-touch-disk by construction: postgres ∧ redis ∧ tmpfs).
    WorkingSetRamdisk,
    // ── CACHE ──
    /// content-addressed BLAKE3 tree-hash + density-greedy tier admit (identical
    /// content ⇒ identical key ⇒ auto-dedup, no lock — eliminate-the-shared-cell).
    ContentAddressedBlake3,
    /// cost-cold eviction ranking — evict by `(coldness × re-derivation-cost)`
    /// (LRU-cost hybrid cache-replacement priority).
    CostColdEviction,
    /// DAG-frontier prefetch — warm the derivations the topo-frontier will need
    /// before the miss (predict → shadow → confirm → allocate).
    DagFrontierPrefetch,
    /// state-in-DB durability — the CA store IS Postgres, so disk-loss-on-reclaim
    /// is unrepresentable; the cache IS the build checkpoint.
    StateInDbDurability,
    // ── TRANSFER ──
    /// content-addressable chunk dedup (Nydus RAFS / zstd:chunked) — re-address
    /// coarse layers to chunk-granular so 99%-unchanged layers don't re-ship.
    ChunkDedupNydus,
    /// P2P swarm source-selection (Dragonfly / Spegel Kademlia-DHT) — rank
    /// local > peer-swarm > faucet > origin.
    P2pSwarmSelect,
    /// samba `LeakyBucket` pacing — rate derived from `X-RateLimit-Limit`,
    /// `quota-pct` the single knob (never trip the secondary rate-limit).
    SambaLeakyBucket,
    // ── DEPLOY ──
    /// native-arch build placement — build `dockerImage-<arch>` per arch on its
    /// matching-arch runner (no QEMU cross-emulation; fixed the hanabi
    /// `exec format error`). A scheduling constraint: the build lands on the
    /// runner whose arch matches the target.
    NativeArchBuild,
    /// sortable exact immutable tag `<arch>-r<run>-<sha>` — `r<run>` numerically
    /// orderable (reconciler picks newest); `:latest` is never a deploy source.
    SortableExactTag,
    /// Flux image-reflector + image-automation — an autobump IS a git commit of an
    /// exact tag (GitOps-native, auditable, revertable).
    FluxImageAutomation,
    // ── RUN ──
    /// multiplicative-band carve `⌈used/setpoint⌉` — floor = the startup-observed
    /// proven minimum (cost DOWN + headroom before the OOM/CFS cliff).
    MultiplicativeBandCarve,
    /// grow-only-predictive storage — born at a floor, grown online via
    /// least-squares `fill_velocity`/`seconds_to_full` (never over-provisions).
    GrowOnlyPredictive,
    /// replica-topology-scale — floor-2 HA; topology picks the algorithm
    /// (stateless free-scale / ordinal-rebalance / read-replicas-only / odd-quorum).
    ReplicaTopologyScale,
    /// capacity-optimized-prioritized auction over the arm∪x86 diversified family
    /// union (MiB-per-milli-dollar bid; on-demand parse-rejected — truly-unrep).
    CapacityOptimizedPrioritized,
    /// cluster-autoscaler 0→N + predictive-start (dormant → near-free in the dark).
    ScaleToZero,
    /// leak-detection FSM + straggler reap — a Viggy `(desired=no-leak, observed,
    /// reap)` controller across the 9 leak classes. COMPOSED by reference from the
    /// autorevivy leak-guards (owned elsewhere) — never re-implemented here.
    LeakDetectionFsm,
    /// capacity-rebalance + 2-min-warning drain-ahead + idempotent re-dispatch
    /// (survive reclaim / replica-down without a fatal loop).
    CapacityRebalanceDrain,
}

impl SealedAlgorithm {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CpmListScheduling => "cpm-list-scheduling",
            Self::BoxDetectMaxJobs => "box-detect-max-jobs",
            Self::MaximalDagFineGrain => "maximal-dag-fine-grain",
            Self::ArchCostResolve => "arch-cost-resolve",
            Self::WorkingSetRamdisk => "working-set-ramdisk",
            Self::ContentAddressedBlake3 => "content-addressed-blake3",
            Self::CostColdEviction => "cost-cold-eviction",
            Self::DagFrontierPrefetch => "dag-frontier-prefetch",
            Self::StateInDbDurability => "state-in-db-durability",
            Self::ChunkDedupNydus => "chunk-dedup-nydus",
            Self::P2pSwarmSelect => "p2p-swarm-select",
            Self::SambaLeakyBucket => "samba-leaky-bucket",
            Self::NativeArchBuild => "native-arch-build",
            Self::SortableExactTag => "sortable-exact-tag",
            Self::FluxImageAutomation => "flux-image-automation",
            Self::MultiplicativeBandCarve => "multiplicative-band-carve",
            Self::GrowOnlyPredictive => "grow-only-predictive",
            Self::ReplicaTopologyScale => "replica-topology-scale",
            Self::CapacityOptimizedPrioritized => "capacity-optimized-prioritized",
            Self::ScaleToZero => "scale-to-zero",
            Self::LeakDetectionFsm => "leak-detection-fsm",
            Self::CapacityRebalanceDrain => "capacity-rebalance-drain",
        }
    }

    /// The classical/SOTA family this sealed algorithm belongs to.
    #[must_use]
    pub const fn family(self) -> AlgorithmFamily {
        match self {
            Self::CpmListScheduling => AlgorithmFamily::DagScheduling,
            Self::BoxDetectMaxJobs => AlgorithmFamily::ConvexLp,
            Self::MaximalDagFineGrain => AlgorithmFamily::DagScheduling,
            Self::ArchCostResolve => AlgorithmFamily::GreedyHillClimb,
            Self::WorkingSetRamdisk => AlgorithmFamily::ControlTheory,
            Self::ContentAddressedBlake3 => AlgorithmFamily::ContentAddressing,
            Self::CostColdEviction => AlgorithmFamily::CacheReplacement,
            Self::DagFrontierPrefetch => AlgorithmFamily::DagScheduling,
            Self::StateInDbDurability => AlgorithmFamily::ContentAddressing,
            Self::ChunkDedupNydus => AlgorithmFamily::ContentAddressing,
            Self::P2pSwarmSelect => AlgorithmFamily::ContentAddressing,
            Self::SambaLeakyBucket => AlgorithmFamily::RateLimiting,
            Self::NativeArchBuild => AlgorithmFamily::DagScheduling,
            Self::SortableExactTag => AlgorithmFamily::ContentAddressing,
            Self::FluxImageAutomation => AlgorithmFamily::ContentAddressing,
            Self::MultiplicativeBandCarve => AlgorithmFamily::ControlTheory,
            Self::GrowOnlyPredictive => AlgorithmFamily::LeastSquares,
            Self::ReplicaTopologyScale => AlgorithmFamily::ControlTheory,
            Self::CapacityOptimizedPrioritized => AlgorithmFamily::Auction,
            Self::ScaleToZero => AlgorithmFamily::ControlTheory,
            Self::LeakDetectionFsm => AlgorithmFamily::ControlTheory,
            Self::CapacityRebalanceDrain => AlgorithmFamily::IdempotentRetry,
        }
    }

    /// **The NO-ML seal, per algorithm** — always `false` (delegates to the
    /// family). An axis whose optimum needs a model does not exist in this enum.
    #[must_use]
    pub const fn is_ml(self) -> bool {
        self.family().is_ml()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TIER — the honesty ladder (shipped vs authored vs design); NEVER round up
// ─────────────────────────────────────────────────────────────────────────────

/// The EVIDENCE tier of an axis — how far its live loop actually is, per the map
/// ledger. This is an **evidence ordering, not a preference**: a higher tier
/// means *more shipped*, and the honesty gate ([`crate::honesty`]) forbids
/// claiming a tier above the evidence (never round up). The optimizer can tune an
/// axis' knobs; it can NEVER promote its tier.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Tier {
    /// DESIGN / LiveTODO — the sealed algorithm is designed + tier-honestly named,
    /// but its live loop is not built (composes a design-stage primitive).
    DesignLiveTodo,
    /// AUTHORED / branch-proven — the sealed algorithm ships + is proven on a
    /// branch (green tests), but is not yet live on the fleet.
    AuthoredProven,
    /// SHIPPED — the sealed algorithm is live + green on the fleet today.
    Shipped,
}

impl Tier {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DesignLiveTodo => "design-live-todo",
            Self::AuthoredProven => "authored-proven",
            Self::Shipped => "shipped",
        }
    }

    /// Does this axis optimize LIVE tick-by-tick today (Shipped), as opposed to
    /// authored-on-a-branch or design-stage? The catalog's honest split.
    #[must_use]
    pub const fn is_live(self) -> bool {
        matches!(self, Self::Shipped)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// CHAIN AXIS — the record binding link × algorithm × clock × tier
// ─────────────────────────────────────────────────────────────────────────────

/// One optimization axis of the delivery chain — a self-describing row of the
/// [`crate::catalog`]. It names WHAT it optimizes, WHICH sealed algorithm does
/// it, WHICH clock ticks it, WHAT tier its live loop is (tier-honest), and WHAT
/// shipped primitive it composes BY REFERENCE (`doctrine_ref`) — never a fork.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainAxis {
    /// The delivery-chain link this axis lives on.
    pub link: Link,
    /// The axis' stable name (globally unique across the catalog).
    pub name: &'static str,
    /// One-line objective (what it minimizes/maximizes, s.t. its constraint).
    pub objective: &'static str,
    /// The sealed classical/SOTA algorithm (NO ML).
    pub algorithm: SealedAlgorithm,
    /// Which of the three clocks drives it.
    pub tick_source: TickSource,
    /// The tier-honest live-loop maturity (never rounded up).
    pub tier: Tier,
    /// The shipped/authored primitive this axis COMPOSES by reference — a
    /// `doctrine_ref`, never a re-implementation.
    pub composes: &'static str,
    /// Where the invariant is sealed (the typed border that makes the illegal
    /// state unrepresentable / parse-rejected / CI-caught).
    pub sealed_at: &'static str,
}

impl ChainAxis {
    /// Does this axis optimize LIVE tick-by-tick today (Shipped tier)?
    #[must_use]
    pub const fn optimizes_live(&self) -> bool {
        self.tier.is_live()
    }

    /// Is this axis a lapidar KNOB (tuned by accept-if-improved), as opposed to a
    /// posture-owned structure or a reactive interrupt?
    #[must_use]
    pub const fn is_lapidar_knob(&self) -> bool {
        self.tick_source.is_lapidar_knob()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_in_chain_order_and_unique() {
        assert_eq!(Link::ALL, [Link::Build, Link::Cache, Link::Transfer, Link::Deploy, Link::Run]);
        let labels: Vec<&str> = Link::ALL.iter().map(|l| l.as_str()).collect();
        let mut sorted = labels.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), labels.len(), "link labels are unique");
    }

    #[test]
    fn there_is_no_ml_family() {
        // /algorithmic-prowess-seal: NO ML. Every family (and hence every sealed
        // algorithm) reports is_ml() == false — an ML axis is unrepresentable.
        for f in [
            AlgorithmFamily::DagScheduling,
            AlgorithmFamily::ContentAddressing,
            AlgorithmFamily::CacheReplacement,
            AlgorithmFamily::Auction,
            AlgorithmFamily::ControlTheory,
            AlgorithmFamily::GreedyHillClimb,
            AlgorithmFamily::LeastSquares,
            AlgorithmFamily::RateLimiting,
            AlgorithmFamily::IdempotentRetry,
            AlgorithmFamily::ConvexLp,
        ] {
            assert!(!f.is_ml(), "{} must be classical/SOTA, never ML", f.as_str());
        }
    }

    #[test]
    fn only_the_lapidar_clock_is_a_knob() {
        assert!(TickSource::LapidarAcceptIfImproved.is_lapidar_knob());
        assert!(!TickSource::PostureSevenBeat.is_lapidar_knob());
        assert!(!TickSource::ReactiveNervousSystem.is_lapidar_knob());
    }

    #[test]
    fn tier_orders_by_evidence_shipped_is_highest() {
        assert!(Tier::Shipped > Tier::AuthoredProven);
        assert!(Tier::AuthoredProven > Tier::DesignLiveTodo);
        assert!(Tier::Shipped.is_live());
        assert!(!Tier::AuthoredProven.is_live());
        assert!(!Tier::DesignLiveTodo.is_live());
    }

    #[test]
    fn every_sealed_algorithm_maps_to_a_family_and_is_not_ml() {
        // Exhaustive over the enum via a representative sample; the catalog test
        // covers the full set. Here: a spot-check that family() is total + no-ML.
        for a in [
            SealedAlgorithm::CpmListScheduling,
            SealedAlgorithm::ArchCostResolve,
            SealedAlgorithm::ContentAddressedBlake3,
            SealedAlgorithm::SambaLeakyBucket,
            SealedAlgorithm::MultiplicativeBandCarve,
            SealedAlgorithm::CapacityOptimizedPrioritized,
        ] {
            assert!(!a.is_ml(), "{} must not be ML", a.as_str());
            assert!(!a.family().as_str().is_empty());
        }
    }
}
