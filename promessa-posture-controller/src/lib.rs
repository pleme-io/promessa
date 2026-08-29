//! `promessa-posture-controller` — the tick-by-tick posture-enforcement Viggy
//! controller.
//!
//! The posture is not a checklist a human sweeps; it is ONE typed
//! invariant set the cluster proves it is holding **tick by tick, eternally**.
//! This crate is the [`TargetController`] that enforces it — the pure
//! `diff → classify → decide` core of the Viggy seven-beat (observe → diff →
//! classify → decide → act → attest → tick). The I/O beats (observe / act /
//! attest) live at the reconcile runtime and the composed guards; the three
//! beats here are pure, deterministic, and fully unit-tested.
//!
//! ## What it enforces (the whole posture as one invariant set)
//!
//! - **breathability holds across mem / cpu / STORAGE** (every band at its
//!   setpoint) — composes breathe, including the now-first-class StorageBand
//!   (`breathe_control::ProvisionVerdict`). A volume that is `OverProvisioned`
//!   is a storage-dimension violation with an exact reclaimable `waste`.
//! - **scale-to-zero holds** (build pools + idle → 0, no stuck node).
//! - **100% spot** — an on-demand node is a hard-law violation (never on-demand).
//! - **arm** (Graviton default) — a non-arm node where arm is required.
//! - **never-stuck** — no wedged node / pool.
//! - **no leak in ANY dimension** (memory / cpu / storage / persistence / volume
//!   / straggler + placement / idle-node / orphan-cost) — composes the autorevivy
//!   leak-guard [`MaintenanceJob`]s by typed reference, never re-implementing them.
//! - **every critical workload is SEALED from interference** — the isolation
//!   invariant (breathe's `IsolationBand` seal dimension). A critical /
//!   interference-sensitive workload observed at BestEffort / no-requests (the
//!   victoria-logs-422 class) is an [`PostureViolation::IsolationSealBroken`];
//!   observed noisy-neighbor contention (throttle / eviction pressure) is an
//!   [`PostureViolation::InterferenceDetected`]. The seal-broken ROOT CAUSE
//!   (raise the requests-floor) composes breathe's isolation seal-carve guard by
//!   tag; re-placement is disruptive and human-gated. The carve is bounded so it
//!   never strips the seal (breathe `carve_respecting_seal` / `SealedCarve`).
//! - **no errors** — the service-auction side is stability-first: errors are
//!   surfaced, never blind-remediated.
//!
//! ## Shadow-first, no blind remediation (the no-errors discipline)
//!
//! Enforcement observes-would-act before acting. `decide` NEVER auto-fires a
//! DISRUPTIVE remediation (a data-volume recreate is always `RequireApproval`);
//! the composed guards it dispatches to (breathe bands, autorevivy leak-guards)
//! are themselves shadow-first (their own `dryRun` / `ShadowGate`), so a
//! posture-driven action can never introduce an error via a blind carve. Under
//! `shadow_first` (the default) the controller surfaces + attests Functional
//! violations rather than auto-correcting them — only the safety-critical
//! composed guards (spot-reacquisition, orphan-cost reap, never-stuck) dispatch,
//! and even those act only through their own shadow gate.
//!
//! ## Composition, not re-implementation
//!
//! The posture controller owns NO leak-guard, NO band law, NO reap loop. It
//! OBSERVES the composed subsystems' state and DISPATCHES to their guards by
//! typed reference (the `guard_tag` values mirror `autorevivy::dag::
//! MaintenanceJob::tag()`), exactly as every Viggy controller dispatches to a
//! reconciler by name. This keeps the posture at the promessa altitude — a
//! cross-controller business outcome — and lets the guards evolve in their own
//! crates without a hard dependency edge.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use promessa_types::action::{ReconcilerKind, TypedAction};
use promessa_types::{Decision, PromessaTargetKind, Severity, TargetController};
use serde::{Deserialize, Serialize};

/// A resource dimension breathe carves to its setpoint. The completed
/// mem/cpu/STORAGE triad + the horizontal replica dimension — every one held at
/// the band by breathe, every one an enforceable part of the posture.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum BandDimension {
    Memory,
    Cpu,
    /// The now-first-class provision-minimal + grow-on-demand storage carve.
    Storage,
    Replica,
}

impl BandDimension {
    /// The stable label (used in dispatch specs + as a stable id).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Cpu => "cpu",
            Self::Storage => "storage",
            Self::Replica => "replica",
        }
    }
}

/// A leak class the posture forbids across ALL dimensions. The three cost-guard
/// classes mirror `autorevivy::dag::MaintenanceJob`; the six dimension leaks are
/// the "no leak any dimension" set the leak-guard family covers.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum LeakClass {
    Memory,
    Cpu,
    Storage,
    Persistence,
    Volume,
    Straggler,
    /// Persistent-service on a transient build pool — autorevivy `PlacementLeakGuard`.
    Placement,
    /// Idle build-pool node past threshold — autorevivy `IdleNodeReap`.
    IdleNode,
    /// Orphaned cloud cost-residue (EBS/LB/PVC/snapshot) — autorevivy `OrphanCostReap`.
    OrphanCost,
}

impl LeakClass {
    /// The autorevivy `MaintenanceJob::tag()` this leak dispatches to — the
    /// compose-by-typed-reference binding to the leak-guard that services it. The
    /// dimension leaks name the forthcoming per-dimension guards; the three cost
    /// classes name the shipped autorevivy jobs verbatim.
    #[must_use]
    pub const fn guard_tag(self) -> &'static str {
        match self {
            Self::Memory => "memory_leak_guard",
            Self::Cpu => "cpu_leak_guard",
            Self::Storage => "storage_leak_guard",
            Self::Persistence => "persistence_leak_guard",
            Self::Volume => "volume_leak_guard",
            Self::Straggler => "straggler_leak_guard",
            Self::Placement => "placement_leak_guard",
            Self::IdleNode => "idle_node_reap",
            Self::OrphanCost => "orphan_cost_reap",
        }
    }

    /// A continuously-billing leak (money bleeds every tick it is open) — these
    /// are `Critical`. An orphaned cloud resource is the canonical case.
    #[must_use]
    pub const fn is_cost_critical(self) -> bool {
        matches!(self, Self::OrphanCost)
    }
}

/// An over-provisioned volume observed against breathe's provision-minimal carve
/// (`breathe_control::ProvisionVerdict::OverProvisioned`). Carries the exact
/// reclaimable waste + whether the volume is regenerable (safe to recreate) or
/// stateful (must never be blind-recreated).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct OverProvisionedVolume {
    pub pvc: String,
    pub waste_bytes: u64,
    /// `true` ⇒ the volume's data is regenerable (sui-pg / victoria / nats /
    /// alertmanager / zot / lookout) — a one-time recreate reclaims the waste
    /// safely (still human-gated). `false` ⇒ stateful (mysql / rustfs) — never
    /// recreate; the provision-minimal default prevents NEW ones, existing ones
    /// are surfaced only.
    pub regenerable: bool,
}

/// The DECLARED posture — the invariant set the cluster must hold.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PostureSpec {
    pub cluster: String,
    /// The breathe dimensions that must be held at their setpoint (the carve).
    pub carved_dimensions: BTreeSet<BandDimension>,
    /// Build pools + idle workloads must scale to zero.
    pub require_scale_to_zero: bool,
    /// 100% spot — no on-demand node permitted (the hard law).
    pub require_100pct_spot: bool,
    /// arm (Graviton) is the default arch.
    pub require_arm: bool,
    /// No leak in any of these classes.
    pub forbid_leaks: BTreeSet<LeakClass>,
    /// Never-stuck — no wedged node / pool.
    pub require_never_stuck: bool,
    /// Every critical / interference-sensitive workload must be SEALED — a
    /// guaranteed requests-floor + a non-BestEffort QoS (breathe's `IsolationBand`
    /// seal dimension). When set, an unsealed critical workload or observed
    /// noisy-neighbor interference is a posture violation. This is the isolation
    /// invariant enforced tick-by-tick, not a one-time config.
    pub require_isolation_seal: bool,
    /// Shadow-first: enforcement observes-would-act before acting. Under this,
    /// Functional violations are surfaced + attested (Alert), not auto-corrected;
    /// only safety-critical composed-guard dispatches fire (and they are
    /// themselves shadow-first). The footgun-safe default.
    pub shadow_first: bool,
}

impl PostureSpec {
    /// The complete posture: every dimension carved, scale-to-zero, 100%
    /// spot, arm, never-stuck, every leak class forbidden, shadow-first. The one
    /// typed row that arms the whole posture — the peer of breathe's
    /// `SPOT_AGGRESSIVE` preset at the promessa altitude.
    #[must_use]
    pub fn full(cluster: impl Into<String>) -> Self {
        Self {
            cluster: cluster.into(),
            carved_dimensions: [
                BandDimension::Memory,
                BandDimension::Cpu,
                BandDimension::Storage,
                BandDimension::Replica,
            ]
            .into_iter()
            .collect(),
            require_scale_to_zero: true,
            require_100pct_spot: true,
            require_arm: true,
            forbid_leaks: [
                LeakClass::Memory,
                LeakClass::Cpu,
                LeakClass::Storage,
                LeakClass::Persistence,
                LeakClass::Volume,
                LeakClass::Straggler,
                LeakClass::Placement,
                LeakClass::IdleNode,
                LeakClass::OrphanCost,
            ]
            .into_iter()
            .collect(),
            require_never_stuck: true,
            require_isolation_seal: true,
            shadow_first: true,
        }
    }
}

/// One predicate of the posture — an index over the invariant switches on
/// [`PostureSpec`], so a snapshot can say WHICH part of the posture it
/// failed to observe. Not a new taxonomy: each variant names an existing spec
/// field, and its only job is to make blindness attributable.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum PosturePredicate {
    /// The carved dimensions are at their setpoint (`carved_dimensions`).
    Bands,
    /// 100% spot (`require_100pct_spot`).
    Spot,
    /// arm default (`require_arm`).
    Arm,
    /// Build pools + idle → 0 (`require_scale_to_zero`).
    ScaleToZero,
    /// No wedged node / pool (`require_never_stuck`).
    NeverStuck,
    /// No leak in any forbidden class (`forbid_leaks`).
    Leaks,
    /// No over-provisioned volume (the storage-waste surface).
    Storage,
    /// Every critical workload sealed (`require_isolation_seal`).
    IsolationSeal,
    /// The service-auction / reconcile error surface.
    Errors,
}

impl PosturePredicate {
    /// Every predicate, in declaration order.
    pub const ALL: [PosturePredicate; 9] = [
        Self::Bands,
        Self::Spot,
        Self::Arm,
        Self::ScaleToZero,
        Self::NeverStuck,
        Self::Leaks,
        Self::Storage,
        Self::IsolationSeal,
        Self::Errors,
    ];

    /// The stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bands => "bands",
            Self::Spot => "spot",
            Self::Arm => "arm",
            Self::ScaleToZero => "scale-to-zero",
            Self::NeverStuck => "never-stuck",
            Self::Leaks => "leaks",
            Self::Storage => "storage",
            Self::IsolationSeal => "isolation-seal",
            Self::Errors => "errors",
        }
    }
}

/// The OBSERVED posture — what the reconcile runtime's `observe` beat projects
/// the running cluster into (breathe MCP band phases, node capacity types, leak
/// observations, over-provisioned volumes). Pure data.
///
/// ## Why `blind` had to be added
///
/// Every other field on this struct uses an EMPTY collection to mean "none
/// observed" — and an empty collection is produced identically by "I looked and
/// there are none" and by "I could not look". On a cluster with no
/// kube-state-metrics, no node-exporter and no alerting, that collapse is the
/// exact mechanism by which this controller would classify a posture `Cosmetic`
/// while blind. `blind` is the presence-anchor: it names the
/// predicates the observe beat could NOT evaluate this tick, so
/// [`PostureController::verdict`] can refuse to attest a posture it did
/// not actually see. `#[serde(default)]` keeps older payloads deserializable —
/// though note that an old payload therefore decodes as "nothing blind", which
/// is only sound because those payloads predate any blindness-aware producer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PostureSnapshot {
    pub cluster: String,
    pub observed_at: DateTime<Utc>,
    /// Dimensions observed OFF their setpoint band (sustained, not transient).
    pub off_band_dimensions: BTreeSet<BandDimension>,
    /// On-demand nodes observed — should be empty under 100% spot.
    pub on_demand_nodes: Vec<String>,
    /// Non-arm (x86) nodes observed where arm was required.
    pub non_arm_nodes: Vec<String>,
    /// Build pools / idle workloads NOT scaled to zero when they should be.
    pub scale_to_zero_stuck: Vec<String>,
    /// Wedged nodes / pools (never-stuck violation).
    pub stuck_units: Vec<String>,
    /// Leaks observed, by class (composes the leak-guards' observations).
    pub observed_leaks: BTreeSet<LeakClass>,
    /// Over-provisioned volumes (breathe `ProvisionVerdict::OverProvisioned`).
    pub over_provisioned_volumes: Vec<OverProvisionedVolume>,
    /// Critical / interference-sensitive workloads observed WITHOUT a seal
    /// (BestEffort QoS or a zero requests-floor) — the victoria-logs-422 class.
    /// Empty under a held isolation invariant.
    pub unsealed_critical_workloads: Vec<String>,
    /// Workloads observed suffering interference (noisy-neighbor contention:
    /// CPU-throttle ratio / eviction pressure above threshold). The seal-broken
    /// case's runtime symptom; re-placement is the (disruptive, human-gated) fix.
    pub interfered_workloads: Vec<String>,
    /// Service-auction / reconcile errors observed (stability-first surface).
    pub error_count: u32,
    /// Predicates the observe beat could NOT evaluate this tick. A non-empty set
    /// means every "clean" field above is a LOWER BOUND, not a clean bill of
    /// health — see the type docs.
    #[serde(default)]
    pub blind: BTreeSet<PosturePredicate>,
}

/// A single typed posture violation. The whole `diff` is the set of these; each
/// carries its own severity and its own (shadow-first) remediation.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "violation", rename_all = "kebab-case")]
pub enum PostureViolation {
    /// A carved dimension is off its setpoint band — breathe is the carver; the
    /// posture surfaces + attests.
    ///
    /// **FALSIFIED 2026-08-01 — this variant used to justify its `AlertOnly`
    /// routing with "breathe converges it on its own loop". That deference is no
    /// longer sound and must not be re-asserted.** Observed on a live production
    /// cluster: a stateful-database CPU band carries
    /// `postureRef: critical-stateful`, whose tuple
    /// is `shrinkBelow: 0.0` (never shrink) and `growFactor: 1.5`, and it still
    /// emitted `shadow: 1000 -> 900` shrink proposals at `util 0.005` and
    /// `1000 -> 1250` grow proposals — a `1.25x` step, which is the superseded
    /// `platform-default` factor, not its declared one. So the band's EFFECTIVE
    /// policy is not its DECLARED policy, and "breathe will converge it" is an
    /// assumption about a loop that is running different numbers than the ones
    /// authored.
    ///
    /// The routing below is UNCHANGED — surfacing remains correct, and blind
    /// auto-correction would be worse. What changed is the justification: the
    /// posture now surfaces because remediation here is a judgement call, NOT
    /// because breathe is trusted to converge. The independent detector for the
    /// underlying defect is `sarar::selfobs::classify_policy_resolution`
    /// (`IssueKind::PolicyResolutionDivergence`), which compares a band's
    /// observed proposals against what its declared tier could possibly emit,
    /// and which rests on no authored YAML being correct.
    BandOffSetpoint { dimension: BandDimension },
    /// On-demand nodes present — the 100%-spot hard law is broken (Critical).
    OnDemandDetected { nodes: Vec<String> },
    /// Non-arm nodes where arm was required.
    NotArm { nodes: Vec<String> },
    /// Build pools / idle workloads not scaled to zero.
    ScaleToZeroStuck { units: Vec<String> },
    /// A wedged node / pool — the never-stuck invariant is broken (Critical).
    NeverStuckViolated { units: Vec<String> },
    /// A leak in a forbidden dimension — dispatched to its leak-guard.
    LeakDetected { class: LeakClass },
    /// An over-provisioned volume — reclaimable storage waste.
    OverProvisioned {
        pvc: String,
        waste_bytes: u64,
        regenerable: bool,
    },
    /// A critical / interference-sensitive workload with NO seal (BestEffort /
    /// no-requests) — the victoria-logs-422 ROOT CAUSE. The correction (raise the
    /// requests-floor) composes breathe's isolation seal-carve guard by tag,
    /// shadow-first. Distinct from `NeverStuckViolated` (the SYMPTOM): a stuck
    /// workload trips never-stuck; the missing seal is why it stuck.
    IsolationSealBroken { workloads: Vec<String> },
    /// Noisy-neighbor interference observed on a workload (throttle / eviction
    /// pressure) — the seal-broken symptom. The fix (re-place with anti-affinity /
    /// isolate-away) is DISRUPTIVE, so it is human-gated, never auto-fired.
    InterferenceDetected { workloads: Vec<String> },
    /// Service-auction / reconcile errors — surfaced, never blind-remediated.
    ErrorsObserved { count: u32 },
}

/// How a single violation is remediated — the shadow-first routing.
enum Remediation {
    /// A shadow-safe composed-guard dispatch (the guard is itself shadow-first).
    Auto(TypedAction),
    /// A disruptive remediation (a data-volume recreate) — always human-gated,
    /// never auto-fired (the no-errors discipline).
    Approval(TypedAction),
    /// Observe + attest only; the composed subsystem's own loop converges it.
    AlertOnly,
}

impl PostureViolation {
    /// The severity of this one violation. `classify` takes the max across the
    /// set, which is monotone in the violation set (a superset can only raise it).
    #[must_use]
    pub fn severity(&self) -> Severity {
        match self {
            // Hard-law + money + wedged: immediate, Critical.
            Self::OnDemandDetected { .. } | Self::NeverStuckViolated { .. } => Severity::Critical,
            Self::LeakDetected { class } if class.is_cost_critical() => Severity::Critical,
            // Efficiency / correctness breaches worth acting on.
            Self::BandOffSetpoint { .. }
            | Self::ScaleToZeroStuck { .. }
            | Self::LeakDetected { .. }
            | Self::OverProvisioned { .. }
            | Self::IsolationSealBroken { .. }
            | Self::InterferenceDetected { .. }
            | Self::ErrorsObserved { .. } => Severity::Functional,
            // A placement preference, not a breach.
            Self::NotArm { .. } => Severity::Cosmetic,
        }
    }

    /// The shadow-first remediation routing for this violation.
    fn remediation(&self, cluster: &str) -> Remediation {
        match self {
            // Safety-critical composed-guard dispatches (the guard is shadow-first).
            Self::OnDemandDetected { nodes } => Remediation::Auto(dispatch_guard(
                "spot_reacquisition",
                serde_json::json!({ "cluster": cluster, "onDemandNodes": nodes }),
            )),
            Self::NeverStuckViolated { units } => Remediation::Auto(dispatch_guard(
                "never_stuck_reconcile",
                serde_json::json!({ "cluster": cluster, "stuckUnits": units }),
            )),
            Self::ScaleToZeroStuck { units } => Remediation::Auto(dispatch_guard(
                LeakClass::IdleNode.guard_tag(),
                serde_json::json!({ "cluster": cluster, "units": units }),
            )),
            Self::LeakDetected { class } => Remediation::Auto(dispatch_guard(
                class.guard_tag(),
                serde_json::json!({ "cluster": cluster, "leakClass": class }),
            )),
            // A regenerable over-provisioned volume: a one-time recreate reclaims
            // the waste — DISRUPTIVE, so always human-gated. A stateful one is
            // never recreated (surface only); the provision-minimal default
            // prevents NEW over-provisioning going forward.
            Self::OverProvisioned {
                pvc,
                regenerable: true,
                ..
            } => Remediation::Approval(TypedAction::FluxCommit {
                path: format!("k8s/clusters/{cluster}/apps/{pvc}"),
                patch: serde_json::json!({ "recreatePvcAtProvisionMinimal": pvc }),
            }),
            Self::OverProvisioned {
                regenerable: false, ..
            } => Remediation::AlertOnly,
            // The isolation SEAL is broken (a critical workload lost its floor) —
            // the ROOT-CAUSE fix is to raise the requests-floor, a breathe
            // isolation seal-carve. Dispatched to breathe's guard by tag
            // (shadow-first AT the guard); a raised floor is a bounded carve that
            // never strips the seal (breathe `SealedCarve`), so it is safe to
            // auto-dispatch in live mode + surface under shadow-first.
            Self::IsolationSealBroken { workloads } => Remediation::Auto(dispatch_guard(
                "isolation_seal_carve",
                serde_json::json!({ "cluster": cluster, "workloads": workloads }),
            )),
            // Observed interference: the fix is RE-PLACEMENT (anti-affinity /
            // isolate-away) — a spec change that reschedules pods, DISRUPTIVE, so
            // it is human-gated, never auto-fired (the no-errors discipline).
            Self::InterferenceDetected { workloads } => {
                Remediation::Approval(TypedAction::FluxCommit {
                    path: format!("k8s/clusters/{cluster}/apps"),
                    patch: serde_json::json!({ "isolateAwayWithAntiAffinity": workloads }),
                })
            }
            // breathe is the carver; the posture observes + attests. Placement +
            // errors are surfaced, never blind-mutated.
            Self::BandOffSetpoint { .. } | Self::NotArm { .. } | Self::ErrorsObserved { .. } => {
                Remediation::AlertOnly
            }
        }
    }
}

/// Dispatch to a composed guard by its typed tag — a `ReconcilerApply` naming the
/// guard the posture defers execution to. The guard runs its own shadow-first
/// loop; the posture never re-implements it.
fn dispatch_guard(tag: &str, spec: serde_json::Value) -> TypedAction {
    let mut spec = spec;
    if let serde_json::Value::Object(map) = &mut spec {
        map.insert(
            "guard".to_string(),
            serde_json::Value::String(tag.to_string()),
        );
    }
    TypedAction::ReconcilerApply {
        reconciler: ReconcilerKind::K8sNative,
        spec,
    }
}

/// The typed drift — the set of posture violations this tick. Empty ⇒ the whole
/// posture holds (a `Cosmetic` classify, the quiet steady state).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct PostureDrift {
    pub violations: Vec<PostureViolation>,
}

impl PostureDrift {
    /// Total reclaimable storage waste across all over-provisioned volumes — the
    /// exact figure an over-provisioned-volume receipt turns into once observed.
    #[must_use]
    pub fn total_waste_bytes(&self) -> u64 {
        self.violations
            .iter()
            .filter_map(|v| match v {
                PostureViolation::OverProvisioned { waste_bytes, .. } => Some(*waste_bytes),
                _ => None,
            })
            .sum()
    }
}

/// The result of a tick that knows whether it could SEE what it is claiming.
///
/// `classify` alone answers "how bad is what I found", which silently assumes
/// the observe beat found everything there was. This type separates that from
/// "did I actually look", so the two can never be conflated by a caller. A
/// `Degraded` verdict's severity is a LOWER BOUND — the real posture can only be
/// worse than what a partial read reported.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "verdict", rename_all = "kebab-case")]
pub enum PostureVerdict {
    /// Every predicate was observed; the severity is the whole truth.
    Attested { severity: Severity },
    /// One or more predicates could not be observed. The severity is a floor,
    /// and [`PostureVerdict::holds`] is `false` no matter how clean it looks.
    Degraded {
        severity: Severity,
        blind: BTreeSet<PosturePredicate>,
    },
}

impl PostureVerdict {
    /// Whether the posture is AFFIRMATIVELY held. `true` requires both a clean
    /// classification and a complete observation — a degraded read never
    /// qualifies, however quiet it looks. This is the one predicate that keeps
    /// "we saw nothing wrong" from being reported as "nothing is wrong".
    #[must_use]
    pub fn holds(&self) -> bool {
        matches!(
            self,
            Self::Attested {
                severity: Severity::Cosmetic
            }
        )
    }

    /// The classified severity — a lower bound when [`Self::Degraded`].
    #[must_use]
    pub const fn severity(&self) -> Severity {
        match self {
            Self::Attested { severity } | Self::Degraded { severity, .. } => *severity,
        }
    }

    /// The predicates that could not be observed.
    #[must_use]
    pub fn blind(&self) -> &BTreeSet<PosturePredicate> {
        static EMPTY: std::sync::OnceLock<BTreeSet<PosturePredicate>> = std::sync::OnceLock::new();
        match self {
            Self::Attested { .. } => EMPTY.get_or_init(BTreeSet::new),
            Self::Degraded { blind, .. } => blind,
        }
    }
}

/// The posture controller — the tick-by-tick enforcement of the whole
/// posture as one typed invariant set.
#[derive(Debug, Clone, Copy, Default)]
pub struct PostureController;

impl PostureController {
    /// Beat 3', the presence-anchored classify: pair the classified severity
    /// with whether the observe beat could actually see the whole posture.
    ///
    /// Callers that need "is the posture held?" must ask
    /// [`PostureVerdict::holds`] rather than comparing `classify` to
    /// `Cosmetic` — the latter reports a blind tick as healthy, which is the
    /// exact failure this pairing exists to prevent.
    #[must_use]
    pub fn verdict(&self, spec: &PostureSpec, snapshot: &PostureSnapshot) -> PostureVerdict {
        let severity = self.classify(&self.diff(spec, snapshot));
        if snapshot.blind.is_empty() {
            PostureVerdict::Attested { severity }
        } else {
            PostureVerdict::Degraded {
                severity,
                blind: snapshot.blind.clone(),
            }
        }
    }
}

impl TargetController for PostureController {
    type Spec = PostureSpec;
    type Snapshot = PostureSnapshot;
    type Drift = PostureDrift;

    /// The posture is a CROSS-controller business outcome, not one of the five
    /// per-domain kinds — the `Custom` escape hatch is its canonical kind.
    const KIND: PromessaTargetKind = PromessaTargetKind::Custom;

    /// Beat 2 — compute the typed violation set from the required posture and the
    /// observed snapshot. Pure + deterministic: the violations are produced in a
    /// fixed order so `diff` is referentially transparent.
    fn diff(&self, spec: &Self::Spec, snapshot: &Self::Snapshot) -> Self::Drift {
        let mut violations = Vec::new();

        // breathability across every carved dimension (mem / cpu / STORAGE / replica).
        for dim in &spec.carved_dimensions {
            if snapshot.off_band_dimensions.contains(dim) {
                violations.push(PostureViolation::BandOffSetpoint { dimension: *dim });
            }
        }
        // 100% spot — the hard law.
        if spec.require_100pct_spot && !snapshot.on_demand_nodes.is_empty() {
            violations.push(PostureViolation::OnDemandDetected {
                nodes: snapshot.on_demand_nodes.clone(),
            });
        }
        // arm default.
        if spec.require_arm && !snapshot.non_arm_nodes.is_empty() {
            violations.push(PostureViolation::NotArm {
                nodes: snapshot.non_arm_nodes.clone(),
            });
        }
        // scale-to-zero.
        if spec.require_scale_to_zero && !snapshot.scale_to_zero_stuck.is_empty() {
            violations.push(PostureViolation::ScaleToZeroStuck {
                units: snapshot.scale_to_zero_stuck.clone(),
            });
        }
        // never-stuck.
        if spec.require_never_stuck && !snapshot.stuck_units.is_empty() {
            violations.push(PostureViolation::NeverStuckViolated {
                units: snapshot.stuck_units.clone(),
            });
        }
        // no leak in any forbidden dimension (deterministic order — BTreeSet).
        for class in &spec.forbid_leaks {
            if snapshot.observed_leaks.contains(class) {
                violations.push(PostureViolation::LeakDetected { class: *class });
            }
        }
        // over-provisioning — the storage-dimension waste (composes the seal).
        for v in &snapshot.over_provisioned_volumes {
            violations.push(PostureViolation::OverProvisioned {
                pvc: v.pvc.clone(),
                waste_bytes: v.waste_bytes,
                regenerable: v.regenerable,
            });
        }
        // isolation — the SEAL invariant (breathe's IsolationBand seal dimension).
        // A critical workload without a seal is the ROOT CAUSE (raise the floor);
        // observed interference is the symptom (re-place, human-gated).
        if spec.require_isolation_seal {
            if !snapshot.unsealed_critical_workloads.is_empty() {
                violations.push(PostureViolation::IsolationSealBroken {
                    workloads: snapshot.unsealed_critical_workloads.clone(),
                });
            }
            if !snapshot.interfered_workloads.is_empty() {
                violations.push(PostureViolation::InterferenceDetected {
                    workloads: snapshot.interfered_workloads.clone(),
                });
            }
        }
        // errors — stability-first surface.
        if snapshot.error_count > 0 {
            violations.push(PostureViolation::ErrorsObserved {
                count: snapshot.error_count,
            });
        }

        PostureDrift { violations }
    }

    /// Beat 3 — classify the drift into a severity tier. Monotone: the max over
    /// the violation set, so a superset of violations is at-least-as-severe.
    fn classify(&self, drift: &Self::Drift) -> Severity {
        drift
            .violations
            .iter()
            .map(PostureViolation::severity)
            .max()
            .unwrap_or(Severity::Cosmetic)
    }

    /// Beat 4 — decide the shadow-first action. NO blind remediation: a disruptive
    /// (data-volume recreate) is always `RequireApproval`; safety-critical composed
    /// guards dispatch through `AutoCorrect` (and act only through their own shadow
    /// gate); under `shadow_first`, Functional violations are surfaced (`Alert`),
    /// not auto-corrected. Pure.
    fn decide(&self, spec: &Self::Spec, _severity: Severity, drift: &Self::Drift) -> Decision {
        if drift.violations.is_empty() {
            return Decision::NoAction;
        }

        let mut approvals: Vec<TypedAction> = Vec::new();
        let mut critical_autos: Vec<TypedAction> = Vec::new();
        let mut functional_autos: Vec<TypedAction> = Vec::new();

        for v in &drift.violations {
            match v.remediation(&spec.cluster) {
                Remediation::Approval(a) => approvals.push(a),
                Remediation::Auto(a) => {
                    if v.severity() == Severity::Critical {
                        critical_autos.push(a);
                    } else {
                        functional_autos.push(a);
                    }
                }
                Remediation::AlertOnly => {}
            }
        }

        // A disruptive remediation NEVER auto-fires — human-gate the whole plan.
        if !approvals.is_empty() {
            return Decision::RequireApproval(compose(approvals));
        }
        // Safety-critical composed guards dispatch (shadow-first at the guard).
        if !critical_autos.is_empty() {
            return Decision::AutoCorrect(compose(critical_autos));
        }
        // Functional composed guards: auto-corrected only in LIVE mode; under
        // shadow-first they are surfaced + attested (the composed loops converge).
        if !functional_autos.is_empty() && !spec.shadow_first {
            return Decision::AutoCorrect(compose(functional_autos));
        }
        Decision::Alert
    }
}

/// Compose one-or-many actions — a single action passes through un-nested, so a
/// one-violation decision is not wrapped in a needless `Compose`.
fn compose(mut actions: Vec<TypedAction>) -> TypedAction {
    if actions.len() == 1 {
        actions.pop().unwrap_or(TypedAction::Noop)
    } else {
        TypedAction::Compose(actions)
    }
}

// ---------------------------------------------------------------------------
// Back-compat aliases for the 0.1.3 names (★★ MODULARIZE, DON'T DELETE).
//
// The old names said WHERE the posture first ran; the new ones say WHAT it is.
// The values and behaviour are identical — only the names moved off one estate —
// so a consumer on the previous names keeps compiling across the rename.
//
// The two alias FORMS are not a style choice, they are measured:
//
//   * `pub type` aliases the TYPE namespace only. That is enough for the three
//     data structs, which are always named in type position or in a struct
//     literal, and it carries a real `#[deprecated]` warning to the use site.
//   * `PostureController` is a UNIT struct, so consumers write it in VALUE
//     position (`let c = PostureController;`). A `type` alias does not bring a
//     unit struct's constructor into the value namespace, and that use fails
//     with E0423 "expected value, found type alias" — so the controller needs a
//     `pub use` re-export, which carries both namespaces.
//
// The cost of that second form, stated rather than hidden: `#[deprecated]` on a
// `use` declaration is silently ineffective — it compiles, but emits no warning
// at the use site. So the controller alias keeps a consumer building; it cannot
// tell them to move. The doc comment is the only notice they get.
// ---------------------------------------------------------------------------

/// DEPRECATED alias for [`PostureSpec`], kept so a 0.1.3 consumer keeps compiling.
#[deprecated(note = "renamed to `PostureSpec` — the type names a posture, not an estate")]
pub type CamelotPostureSpec = PostureSpec;

/// DEPRECATED alias for [`PostureSnapshot`], kept so a 0.1.3 consumer keeps compiling.
#[deprecated(note = "renamed to `PostureSnapshot` — the type names a posture, not an estate")]
pub type CamelotPostureSnapshot = PostureSnapshot;

/// DEPRECATED alias for [`PostureDrift`], kept so a 0.1.3 consumer keeps compiling.
#[deprecated(note = "renamed to `PostureDrift` — the type names a posture, not an estate")]
pub type CamelotPostureDrift = PostureDrift;

/// DEPRECATED alias for [`PostureController`], kept so a 0.1.3 consumer keeps
/// compiling. A `pub use` rather than a `type` because the controller is a unit
/// struct used in value position; see the note above for why that costs the
/// deprecation warning.
pub use PostureController as CamelotPostureController;

/// The fixture cluster name for this crate's tests, named ONCE.
///
/// Every case reads this const rather than re-typing a literal, so a future
/// rename cannot leave a stale copy behind in one case while the others move.
/// It lives at the crate root rather than inside `mod tests` because the
/// property cases are a SIBLING module (`mod proptests`), not a nested one —
/// each reaches it through its own `use super::*`.
#[cfg(test)]
const CLUSTER: &str = "test-cluster";

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(cluster: &str) -> PostureSnapshot {
        PostureSnapshot {
            cluster: cluster.to_string(),
            observed_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
            off_band_dimensions: BTreeSet::new(),
            on_demand_nodes: vec![],
            non_arm_nodes: vec![],
            scale_to_zero_stuck: vec![],
            stuck_units: vec![],
            observed_leaks: BTreeSet::new(),
            over_provisioned_volumes: vec![],
            unsealed_critical_workloads: vec![],
            interfered_workloads: vec![],
            error_count: 0,
            blind: BTreeSet::new(),
        }
    }

    #[test]
    fn kind_is_custom_posture() {
        assert_eq!(PostureController::KIND, PromessaTargetKind::Custom);
    }

    #[test]
    fn a_healthy_posture_is_cosmetic_no_action() {
        let c = PostureController;
        let spec = PostureSpec::full(CLUSTER);
        let drift = c.diff(&spec, &snap(CLUSTER));
        assert!(
            drift.violations.is_empty(),
            "a clean cluster has no violations"
        );
        assert_eq!(c.classify(&drift), Severity::Cosmetic);
        assert_eq!(
            c.decide(&spec, Severity::Cosmetic, &drift),
            Decision::NoAction
        );
    }

    #[test]
    fn on_demand_is_the_critical_hard_law_and_auto_reacquires_spot() {
        let c = PostureController;
        let spec = PostureSpec::full(CLUSTER);
        let mut s = snap(CLUSTER);
        s.on_demand_nodes = vec!["ip-10-0-1-9".into()];
        let drift = c.diff(&spec, &s);
        assert_eq!(
            c.classify(&drift),
            Severity::Critical,
            "on-demand breaks the 100%-spot hard law"
        );
        // Even under shadow-first, the safety-critical spot-reacquisition dispatches.
        match c.decide(&spec, Severity::Critical, &drift) {
            Decision::AutoCorrect(_) => {}
            other => panic!("on-demand must auto-reacquire spot, got {other:?}"),
        }
    }

    #[test]
    fn an_orphan_cost_leak_is_critical_and_dispatches_the_reap() {
        let c = PostureController;
        let spec = PostureSpec::full(CLUSTER);
        let mut s = snap(CLUSTER);
        s.observed_leaks.insert(LeakClass::OrphanCost);
        let drift = c.diff(&spec, &s);
        assert_eq!(c.classify(&drift), Severity::Critical);
        match c.decide(&spec, Severity::Critical, &drift) {
            Decision::AutoCorrect(TypedAction::ReconcilerApply { spec, .. }) => {
                assert_eq!(
                    spec["guard"], "orphan_cost_reap",
                    "dispatches the autorevivy reap by tag"
                );
            }
            other => panic!("orphan-cost must dispatch the reap, got {other:?}"),
        }
    }

    #[test]
    fn an_over_provisioned_regenerable_volume_is_human_gated_never_auto_recreated() {
        // THE no-errors discipline: a data-volume recreate is disruptive and never
        // auto-fires — it is surfaced for human approval, even though the storage
        // waste is real. (The provision-minimal default prevents NEW ones.)
        let c = PostureController;
        let spec = PostureSpec::full(CLUSTER);
        let mut s = snap(CLUSTER);
        s.over_provisioned_volumes = vec![OverProvisionedVolume {
            pvc: "data-sui-pg-0".into(),
            waste_bytes: 48 << 30,
            regenerable: true,
        }];
        let drift = c.diff(&spec, &s);
        assert_eq!(
            drift.total_waste_bytes(),
            48 << 30,
            "exact reclaimable waste"
        );
        match c.decide(&spec, c.classify(&drift), &drift) {
            Decision::RequireApproval(_) => {}
            other => panic!("a recreate must be human-gated, never auto, got {other:?}"),
        }
    }

    #[test]
    fn a_stateful_over_provisioned_volume_is_surfaced_never_recreated() {
        let c = PostureController;
        let spec = PostureSpec::full(CLUSTER);
        let mut s = snap(CLUSTER);
        s.over_provisioned_volumes = vec![OverProvisionedVolume {
            pvc: "data-mysql-0".into(),
            waste_bytes: 10 << 30,
            regenerable: false,
        }];
        let drift = c.diff(&spec, &s);
        // No approval action (never recreate a stateful volume) → Alert (surface).
        assert_eq!(c.decide(&spec, c.classify(&drift), &drift), Decision::Alert);
    }

    #[test]
    fn a_storage_band_off_setpoint_is_surfaced_breathe_is_the_carver() {
        let c = PostureController;
        let spec = PostureSpec::full(CLUSTER);
        let mut s = snap(CLUSTER);
        s.off_band_dimensions.insert(BandDimension::Storage);
        let drift = c.diff(&spec, &s);
        assert_eq!(drift.violations.len(), 1);
        // breathe converges the band on its own loop; posture surfaces + attests.
        assert_eq!(c.decide(&spec, c.classify(&drift), &drift), Decision::Alert);
    }

    #[test]
    fn shadow_first_surfaces_functional_leaks_live_mode_auto_dispatches() {
        let c = PostureController;
        let mut s = snap(CLUSTER);
        s.observed_leaks.insert(LeakClass::Placement); // Functional, has an Auto guard
        // Shadow-first: a Functional composed-guard dispatch is surfaced, not fired.
        let shadow = PostureSpec::full(CLUSTER);
        let drift = c.diff(&shadow, &s);
        assert_eq!(
            c.decide(&shadow, c.classify(&drift), &drift),
            Decision::Alert
        );
        // Live mode: the same Functional dispatch auto-fires.
        let live = PostureSpec {
            shadow_first: false,
            ..PostureSpec::full(CLUSTER)
        };
        match c.decide(&live, c.classify(&drift), &drift) {
            Decision::AutoCorrect(TypedAction::ReconcilerApply { spec, .. }) => {
                assert_eq!(spec["guard"], "placement_leak_guard");
            }
            other => panic!("live mode must auto-dispatch the placement guard, got {other:?}"),
        }
    }

    #[test]
    fn diff_is_pure_and_deterministic() {
        let c = PostureController;
        let spec = PostureSpec::full(CLUSTER);
        let mut s = snap(CLUSTER);
        s.on_demand_nodes = vec!["n1".into()];
        s.observed_leaks.insert(LeakClass::OrphanCost);
        s.off_band_dimensions.insert(BandDimension::Cpu);
        let d1 = c.diff(&spec, &s);
        let d2 = c.diff(&spec, &s);
        assert_eq!(d1, d2, "diff must be referentially transparent");
    }

    #[test]
    fn classify_is_monotone_in_the_violation_set() {
        // A superset of violations is at-least-as-severe — the classify_monotonic
        // trait law. Add a Critical to a Functional-only drift; severity rises.
        let c = PostureController;
        let spec = PostureSpec::full(CLUSTER);
        let mut s = snap(CLUSTER);
        s.off_band_dimensions.insert(BandDimension::Memory); // Functional
        let functional = c.classify(&c.diff(&spec, &s));
        s.on_demand_nodes = vec!["n1".into()]; // + Critical
        let with_critical = c.classify(&c.diff(&spec, &s));
        assert!(
            with_critical >= functional,
            "adding a violation cannot lower severity"
        );
        assert_eq!(with_critical, Severity::Critical);
    }

    #[test]
    fn the_full_posture_arms_every_dimension_and_leak_class() {
        let spec = PostureSpec::full(CLUSTER);
        assert!(
            spec.carved_dimensions.contains(&BandDimension::Storage),
            "storage is a first-class carved dimension"
        );
        assert_eq!(spec.carved_dimensions.len(), 4);
        assert_eq!(spec.forbid_leaks.len(), 9, "every leak class forbidden");
        assert!(
            spec.require_100pct_spot
                && spec.require_arm
                && spec.require_scale_to_zero
                && spec.require_never_stuck
        );
        assert!(
            spec.require_isolation_seal,
            "the isolation seal invariant is armed by default"
        );
        assert!(spec.shadow_first, "shadow-first is the default");
    }

    #[test]
    fn an_unsealed_critical_workload_dispatches_the_seal_carve_in_live_mode() {
        // THE isolation invariant, reactive: a critical workload observed without
        // a seal (BestEffort / no-requests — the victoria-logs-422 ROOT CAUSE) is
        // corrected by raising the requests-floor (breathe's isolation seal-carve),
        // Functional + composed-guard. Surfaced under shadow-first; auto in live.
        let c = PostureController;
        let mut s = snap(CLUSTER);
        s.unsealed_critical_workloads = vec!["victoria-logs".into()];
        // Shadow-first (default): surfaced + attested, not blind-fired.
        let shadow = PostureSpec::full(CLUSTER);
        let drift = c.diff(&shadow, &s);
        assert_eq!(drift.violations.len(), 1);
        assert_eq!(c.classify(&drift), Severity::Functional);
        assert_eq!(
            c.decide(&shadow, c.classify(&drift), &drift),
            Decision::Alert
        );
        // Live mode: the seal-carve auto-dispatches (a bounded carve — never
        // strips the seal), the root-cause correction for the stuck class.
        let live = PostureSpec {
            shadow_first: false,
            ..PostureSpec::full(CLUSTER)
        };
        match c.decide(&live, c.classify(&drift), &drift) {
            Decision::AutoCorrect(TypedAction::ReconcilerApply { spec, .. }) => {
                assert_eq!(
                    spec["guard"], "isolation_seal_carve",
                    "dispatches breathe's seal-carve by tag"
                );
            }
            other => panic!("live mode must auto-dispatch the seal-carve, got {other:?}"),
        }
    }

    #[test]
    fn observed_interference_is_human_gated_never_auto_replaced() {
        // Re-placement (anti-affinity / isolate-away) reschedules pods — DISRUPTIVE
        // — so an observed-interference remediation is always human-gated, never
        // auto-fired, even in live mode (the no-errors discipline).
        let c = PostureController;
        let mut s = snap(CLUSTER);
        s.interfered_workloads = vec!["noisy-batch".into()];
        let live = PostureSpec {
            shadow_first: false,
            ..PostureSpec::full(CLUSTER)
        };
        let drift = c.diff(&live, &s);
        match c.decide(&live, c.classify(&drift), &drift) {
            Decision::RequireApproval(_) => {}
            other => panic!("a re-placement must be human-gated, got {other:?}"),
        }
    }

    #[test]
    fn isolation_seal_is_not_enforced_when_the_invariant_is_disarmed() {
        // require_isolation_seal is the gate: with it off, an unsealed critical is
        // not a posture violation (a repo that opts out via its own spec).
        let c = PostureController;
        let mut s = snap(CLUSTER);
        s.unsealed_critical_workloads = vec!["x".into()];
        let disarmed = PostureSpec {
            require_isolation_seal: false,
            ..PostureSpec::full(CLUSTER)
        };
        assert!(
            c.diff(&disarmed, &s).violations.is_empty(),
            "disarmed isolation invariant emits no violation"
        );
    }

    /// THE presence-anchor: a tick that found NOTHING wrong but could not
    /// observe part of the posture must not report the posture as held. Before
    /// `blind` existed this snapshot was byte-identical to a genuinely clean one,
    /// which is precisely how a blind loop reports health.
    #[test]
    fn a_blind_tick_never_reports_the_posture_as_held() {
        let c = PostureController;
        let spec = PostureSpec::full(CLUSTER);
        let mut s = snap(CLUSTER);
        // Nothing observed wrong — because the band dimension was unreadable.
        s.blind.insert(PosturePredicate::Bands);
        let drift = c.diff(&spec, &s);
        assert!(
            drift.violations.is_empty(),
            "a blind read finds no violations"
        );
        assert_eq!(
            c.classify(&drift),
            Severity::Cosmetic,
            "and classifies clean"
        );
        let verdict = c.verdict(&spec, &s);
        assert!(
            !verdict.holds(),
            "a blind tick must not claim the posture holds"
        );
        assert_eq!(
            verdict.severity(),
            Severity::Cosmetic,
            "the severity is a LOWER BOUND"
        );
        assert!(verdict.blind().contains(&PosturePredicate::Bands));
    }

    /// The complementary direction — a complete, clean read DOES attest.
    #[test]
    fn a_complete_clean_tick_attests() {
        let c = PostureController;
        let spec = PostureSpec::full(CLUSTER);
        let verdict = c.verdict(&spec, &snap(CLUSTER));
        assert!(verdict.holds());
        assert_eq!(
            verdict,
            PostureVerdict::Attested {
                severity: Severity::Cosmetic
            }
        );
        assert!(verdict.blind().is_empty());
    }

    /// A degraded read that ALSO found a violation keeps both facts: the
    /// severity is real and the coverage is still incomplete.
    #[test]
    fn a_degraded_tick_keeps_both_the_severity_and_the_blindness() {
        let c = PostureController;
        let spec = PostureSpec::full(CLUSTER);
        let mut s = snap(CLUSTER);
        s.on_demand_nodes = vec!["ip-10-0-1-9".into()];
        s.blind.insert(PosturePredicate::IsolationSeal);
        let verdict = c.verdict(&spec, &s);
        assert_eq!(verdict.severity(), Severity::Critical);
        assert!(!verdict.holds());
        assert_eq!(verdict.blind().len(), 1);
    }

    #[test]
    fn spec_and_snapshot_round_trip_through_serde() {
        let spec = PostureSpec::full(CLUSTER);
        let json = serde_json::to_string(&spec).unwrap();
        let back: PostureSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(spec, back);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        /// THE classify_monotonic TRAIT LAW (property-tested): observing MORE
        /// wrong (an extra on-demand node, more errors, an extra leak) can only
        /// raise — never lower — the classified severity. Over arbitrary observed
        /// posture, adding an on-demand node makes it exactly `Critical`.
        #[test]
        fn adding_an_on_demand_node_never_lowers_severity(
            errors in 0u32..5,
            off_cpu in any::<bool>(),
            leak_placement in any::<bool>(),
        ) {
            let c = PostureController;
            let spec = PostureSpec::full(CLUSTER);
            let mut s = PostureSnapshot {
                cluster: CLUSTER.into(),
                observed_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
                off_band_dimensions: BTreeSet::new(),
                on_demand_nodes: vec![],
                non_arm_nodes: vec![],
                scale_to_zero_stuck: vec![],
                stuck_units: vec![],
                observed_leaks: BTreeSet::new(),
                over_provisioned_volumes: vec![],
                unsealed_critical_workloads: vec![],
                interfered_workloads: vec![],
                error_count: errors,
                blind: BTreeSet::new(),
            };
            if off_cpu { s.off_band_dimensions.insert(BandDimension::Cpu); }
            if leak_placement { s.observed_leaks.insert(LeakClass::Placement); }
            let before = c.classify(&c.diff(&spec, &s));
            s.on_demand_nodes.push("n".into());
            let after = c.classify(&c.diff(&spec, &s));
            prop_assert!(after >= before, "adding a violation lowered severity");
            prop_assert_eq!(after, Severity::Critical);
        }

        /// `decide` is TOTAL — it never panics for any observed posture, and it
        /// never auto-corrects a disruptive recreate (the no-errors discipline).
        #[test]
        fn decide_is_total_and_never_auto_recreates(
            regen_waste in 0u64..(1 << 40),
            stateful_waste in 0u64..(1 << 40),
        ) {
            let c = PostureController;
            let spec = PostureSpec::full(CLUSTER);
            let mut s = PostureSnapshot {
                cluster: CLUSTER.into(),
                observed_at: DateTime::<Utc>::from_timestamp(0, 0).unwrap(),
                off_band_dimensions: BTreeSet::new(),
                on_demand_nodes: vec![],
                non_arm_nodes: vec![],
                scale_to_zero_stuck: vec![],
                stuck_units: vec![],
                observed_leaks: BTreeSet::new(),
                over_provisioned_volumes: vec![],
                unsealed_critical_workloads: vec![],
                interfered_workloads: vec![],
                error_count: 0,
                blind: BTreeSet::new(),
            };
            if regen_waste > 0 {
                s.over_provisioned_volumes.push(OverProvisionedVolume { pvc: "r".into(), waste_bytes: regen_waste, regenerable: true });
            }
            if stateful_waste > 0 {
                s.over_provisioned_volumes.push(OverProvisionedVolume { pvc: "s".into(), waste_bytes: stateful_waste, regenerable: false });
            }
            let drift = c.diff(&spec, &s);
            let decision = c.decide(&spec, c.classify(&drift), &drift);
            // A regenerable recreate is only ever offered for human approval.
            if regen_waste > 0 {
                prop_assert!(matches!(decision, Decision::RequireApproval(_)), "a recreate must be human-gated");
            }
        }
    }
}
