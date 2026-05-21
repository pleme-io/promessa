//! `TypedAction` — the dispatch surface per VIGGY-LEGOS Part IV.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// What a controller decides to do, in typed form. Per VIGGY-AUTHORING
/// §5 + VIGGY-LEGOS Part IV, every action terminates in either FluxCD,
/// pangea-operator's universal Reconciler engine, cofre, or
/// crachá-controller. The dispatch graph is finite, typed, and
/// exhaustive.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum TypedAction {
    /// Commit a patch under `k8s/clusters/<cluster>/` — backed by
    /// `GithubRepoReconciler` + FluxCD source/kustomize/helm
    /// controllers. The default action (per VIGGY-AUTHORING §5
    /// GITOPS-NATIVE directive).
    FluxCommit { path: String, patch: serde_json::Value },

    /// Apply via one of the catalogued Reconciler kinds. ~30 LoC + thin
    /// chart per kind per CONVERGENCE-SUBSTRATE §III.2.
    ReconcilerApply { reconciler: ReconcilerKind, spec: serde_json::Value },

    /// Apply a Pangea-declared cloud resource via magma.
    MagmaApply { workspace: String, plan_id: String },

    /// Rotate a cofre-managed secret.
    CofreRotate { secret_ref: String },

    /// Patch a saguao crachá AccessPolicy.
    CrachaPatch { policy: String, patch: serde_json::Value },

    /// Escape hatch — three uses force extraction of a new variant.
    Custom { kind: String, payload: serde_json::Value },

    /// Compose multiple actions via shigoto Dag — each step recorded
    /// in the OutcomeReceipt.
    Compose(Vec<TypedAction>),

    /// Explicitly do nothing this tick. Idempotent on `act()` per
    /// trait law `act_idempotent_on_noop` (VIGGY-AUTHORING §5.2).
    Noop,
}

/// Reconciler kinds — the universal `Reconciler` engine targets per
/// CONVERGENCE-SUBSTRATE §III.2. Subset shipping today + the new
/// kinds the Akeyless FedRAMP SCR work pulls forward.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum ReconcilerKind {
    /// Shipping today.
    Terraform,
    InMemoryKv,
    GithubRepo,
    DnsRecord,
    /// Shipping today (per cartorio's MAKE-IT-REAL-PLAN consumer pulls).
    CartorioAdmit,
    /// Akeyless FedRAMP SCR-driven (substrate-ticket S3).
    HarborMirror,
    GhcrTagRevoke,
    CosignAttest,
    SbomGenerate,
    SlsaProvenance,
    RenovateBaseImageBump,
    /// Planned per CONVERGENCE-SUBSTRATE §III.2.
    VaultPolicy,
    HelmRelease,
    K8sNative,
    SlackChannel,
    DatadogMonitor,
    GrafanaDashboard,
    PgSchema,
    StripeProduct,
    Auth0App,
    Istio,
    Kong,
    PagerDutySchedule,
    LinearProject,
    ArgoWorkflow,
    NixFile,
    BrowserState,
    Cron,
    EmailRule,
    MusicLibrary,
}
