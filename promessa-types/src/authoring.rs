//! Authoring-decision typed enums — VIGGY-AUTHORING §15.1.
//!
//! These types are the *output* of the five-question diagnostic
//! (VIGGY-AUTHORING §1). They reify the substrate-route discipline so
//! the "is this a promessa?" decision is itself a typed value, not a
//! gut call.

use chrono::NaiveDate;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::kind::PromessaTargetKind;

/// The output of the five-question diagnostic. Either this concern is
/// a promessa, a peer controller, a citation of an existing primitive,
/// a deferred promessa, or not-Viggy-shaped at all (substrate route).
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "decision", rename_all = "kebab-case")]
pub enum AuthoringDecision {
    Promessa(PromessaTargetKind),
    PeerController(PeerControllerKind),
    CitePrimitive { primitive_ref: String },
    Defer(DeferReason),
    NotViggy(SubstrateRoute),
}

/// Where peer controllers live in the substrate.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PeerControllerKind {
    KubeNative,
    PangeaMagma,
    PangeaOperatorReconciler { reconciler_kind: String },
    EngenhoCustomCrd,
    SekibanAdmissionPolicy,
    CofreSecretRef,
    SaguaoCracha,
}

/// Where non-Viggy concerns route — the substrate's alternative
/// primitives for one-shot / build-time / exploratory work.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum SubstrateRoute {
    ShinkaMigration,
    KenshiTestGate,
    ForgeGenCodegen,
    ManualWithRunbook,
    PureCompute,
    ShinryuExplore,
}

/// Why a promessa is deferred. Carries a re-evaluation date so the
/// substrate can audit deferrals over time.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "reason", rename_all = "kebab-case")]
pub enum DeferReason {
    NoAuditRequirement { reevaluate_at: NaiveDate },
    ObservationSourceMissing { needed: String },
    KindNotYetSupported { proposed: String },
}
