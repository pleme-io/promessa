//! `rationale` — cost-conflict resolution **by cost**, with an inline rationale
//! that travels with the chosen value and goes **LOUD where the intuitive choice
//! loses**.
//!
//! The standing rule: when two candidate values conflict, resolve by COST — the
//! cheaper wins. But a cost choice that contradicts the aspirational default (the
//! builder wants arm; arm is *faster*) must not be silent: the [`CostRationale`]
//! carries the number and flags itself `abnormal` so the loss is visible, e.g.
//! the map's exact example —
//!
//! > `arch=m5a floor: m5a −19% vs Graviton NOW`
//!
//! This composes breathe-auction's arch cost lever + its `CostRationale`
//! ([`crate::axis::SealedAlgorithm::ArchCostResolve`]) BY REFERENCE
//! ([`ARCH_COST_DOCTRINE_REF`]) — the resolver semantics are re-expressed here as
//! a pure fold over a live cost signal (self-adjusting: the same call yields the
//! other choice the moment pricing crosses), never a fork.
//!
//! **TYPED EMISSION:** the inline sentence is rendered by the [`Display`] impl
//! ([`write!`] inside `Display` — allowed surface #1), never by free `format!()`
//! string composition; the cached [`CostRationale::why`] is that render, so the
//! serialized receipt carries the sentence while the value stays the source.

use std::fmt;

use serde::{Deserialize, Serialize};

/// The `doctrine_ref` the arch cost lever composes — breathe-auction's shipped
/// cost-optimized arch axis (the multi-arch AUTOBUMP image makes arch a free cost
/// lever; the auction lands the cheapest-deepest arch, self-adjusting).
pub const ARCH_COST_DOCTRINE_REF: &str =
    "breathe-auction::axis::resolve_arch + CostRationale (cost-optimized arch, self-adjusting)";

/// Which render shape a rationale takes — a typed discriminant so the inline
/// sentence is emitted from a closed set, never assembled ad hoc.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RationaleShape {
    /// The aspirational option won and is (at least) as cheap — quiet `+N% cheaper`.
    CheaperQuiet,
    /// The aspirational option LOST on cost — loud `floor: … −N% vs …`.
    FloorLoud,
    /// No cost conflict — a single obviously-correct value with a stored reason.
    Uncontested,
}

/// A resolved cost choice + the inline justification that travels with it. When
/// the chosen option is the aspirational one, the rationale is quiet
/// (`abnormal == false`); when the aspirational option LOSES on cost, it is loud
/// (`abnormal == true`) and carries the exact margin so the loss is auditable,
/// never rounded away.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CostRationale {
    /// The context (e.g. `"arch"`, `"band-carve"`).
    pub context: String,
    /// The value chosen (the cheaper one under a cost conflict).
    pub chosen: String,
    /// The value that lost (the alternative, for the receipt).
    pub rejected: String,
    /// `true` ⇒ the aspirational/intuitive option lost on cost — the choice is
    /// counter-intuitive and the render goes LOUD with its number.
    pub abnormal: bool,
    /// The margin by which the winner beat the loser, as a signed percent of the
    /// loser's cost (e.g. `-19` when the aspirational option is 19% pricier). Sign
    /// carries the direction relative to the aspirational option.
    pub advantage_pct: i32,
    /// The render shape.
    pub shape: RationaleShape,
    /// The cached inline sentence — the [`Display`] render, stored so the
    /// serialized receipt carries it. Rendered via `write!`, never `format!()`.
    pub why: String,
}

impl fmt::Display for CostRationale {
    /// TYPED EMISSION surface #1 — the inline rationale is emitted HERE, via
    /// `write!`, from the typed fields. Loud (`−N%`) where abnormal.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.shape {
            RationaleShape::CheaperQuiet => write!(
                f,
                "{}={}: {} +{}% cheaper vs {} NOW",
                self.context, self.chosen, self.chosen, self.advantage_pct, self.rejected
            ),
            RationaleShape::FloorLoud => write!(
                f,
                "{}={} floor: {} −{}% vs {} NOW",
                self.context,
                self.chosen,
                self.chosen,
                self.advantage_pct.abs(),
                self.rejected
            ),
            RationaleShape::Uncontested => f.write_str(&self.why),
        }
    }
}

impl CostRationale {
    fn build(
        context: &str,
        chosen: &str,
        rejected: &str,
        abnormal: bool,
        advantage_pct: i32,
        shape: RationaleShape,
        note: &str,
    ) -> Self {
        let mut r = Self {
            context: context.to_string(),
            chosen: chosen.to_string(),
            rejected: rejected.to_string(),
            abnormal,
            advantage_pct,
            shape,
            // Uncontested carries its reason directly; the contested shapes render
            // from the typed fields and cache the sentence below.
            why: note.to_string(),
        };
        if !matches!(shape, RationaleShape::Uncontested) {
            r.why = r.to_string(); // cache the Display render (write!, not format!)
        }
        r
    }

    /// Resolve a cost conflict between an ASPIRATIONAL option and its ALTERNATIVE,
    /// **by cost**. The cheaper wins. If the aspirational option is at least as
    /// cheap it wins quietly; if the alternative is cheaper the alternative wins
    /// and the rationale goes LOUD (`abnormal`) with the exact margin.
    ///
    /// `aspirational_milli` / `alternative_milli` are effective costs (e.g. spot
    /// $/hr ÷ throughput), integer milli-units for deterministic comparison.
    #[must_use]
    pub fn resolve_by_cost(
        context: &str,
        aspirational: &str,
        aspirational_milli: u32,
        alternative: &str,
        alternative_milli: u32,
    ) -> Self {
        if aspirational_milli <= alternative_milli {
            // The aspirational option is (at least) as cheap — it wins quietly.
            let adv = pct_cheaper(aspirational_milli, alternative_milli);
            Self::build(context, aspirational, alternative, false, adv, RationaleShape::CheaperQuiet, "")
        } else {
            // The aspirational option LOSES on cost — go loud with the number.
            let adv = pct_cheaper(alternative_milli, aspirational_milli);
            Self::build(context, alternative, aspirational, true, -adv, RationaleShape::FloorLoud, "")
        }
    }

    /// A quiet rationale for a choice with no cost conflict (a single obviously
    /// correct value) — carries the reason, `abnormal == false`, zero margin.
    #[must_use]
    pub fn uncontested(chosen: &str, why: &str) -> Self {
        Self::build(chosen, chosen, "", false, 0, RationaleShape::Uncontested, why)
    }
}

/// How much cheaper `winner` is than `loser`, as a rounded percent of the loser's
/// cost (so a `−19%` reads "19% below the loser"). Guards a zero loser cost.
fn pct_cheaper(winner: u32, loser: u32) -> i32 {
    if loser == 0 {
        return 0;
    }
    let diff = i64::from(loser) - i64::from(winner);
    // percent of the loser (the pricier baseline), rounded to nearest.
    let pct = (diff * 100 + i64::from(loser) / 2) / i64::from(loser);
    i32::try_from(pct).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arm_wins_the_builder_quietly_when_cheaper_and_faster() {
        // Builder: arm ≈ −37%/build-hr → arm is the cheaper effective cost. Quiet.
        let r = CostRationale::resolve_by_cost("arch", "arm", 63, "x86", 100);
        assert_eq!(r.chosen, "arm");
        assert!(!r.abnormal, "the aspirational arm winning is not abnormal");
        assert!(r.advantage_pct > 0, "arm advantage is positive");
        assert!(r.why.contains("arm"));
        assert_eq!(r.why, r.to_string(), "the cached why equals the Display render");
    }

    #[test]
    fn the_floor_picks_x86_loudly_the_maps_exact_example() {
        // THE map example, reproduced byte-for-byte: the floor picks the m5a
        // (x86) because m5a large-spot is −19% vs Graviton (arm) right NOW — the
        // aspirational arm LOSES on cost, so the rationale is LOUD. Effective
        // costs: Graviton 123, m5a 100 → m5a 19% below the pricier Graviton.
        let r = CostRationale::resolve_by_cost("arch", "Graviton", 123, "m5a", 100);
        assert_eq!(r.chosen, "m5a", "cost-conflicts resolve by COST — m5a (x86) is cheaper at the floor");
        assert!(r.abnormal, "the aspirational Graviton (arm) losing IS abnormal — the rationale must be loud");
        assert_eq!(r.advantage_pct, -19, "m5a is 19% below the pricier Graviton (100 vs 123)");
        // The vocal, inline justification travels with the choice — the map string.
        assert_eq!(
            r.why, "arch=m5a floor: m5a −19% vs Graviton NOW",
            "the loud inline rationale is the map's exact example"
        );
    }

    #[test]
    fn auto_adjusts_when_pricing_crosses_no_re_decision() {
        // The self-adjust proof: same resolve call, flip the signal, get the other
        // arch — cost-driven, never a hardcode (mirrors breathe-auction resolve_arch).
        let before = CostRationale::resolve_by_cost("arch", "arm", 119, "x86", 100);
        let after = CostRationale::resolve_by_cost("arch", "arm", 90, "x86", 100);
        assert_eq!(before.chosen, "x86", "x86 wins while Graviton is pricier");
        assert_eq!(after.chosen, "arm", "arm wins the instant Graviton crosses");
        assert!(!after.abnormal, "arm winning is the quiet aspirational case");
    }

    #[test]
    fn a_tie_goes_to_the_aspirational_option() {
        let r = CostRationale::resolve_by_cost("arch", "arm", 100, "x86", 100);
        assert_eq!(r.chosen, "arm", "a tie resolves to the aspirational push (arm)");
        assert!(!r.abnormal);
    }

    #[test]
    fn uncontested_choice_is_quiet() {
        let r = CostRationale::uncontested("capacity-optimized", "deepest pool, fewest reclaims");
        assert!(!r.abnormal);
        assert_eq!(r.advantage_pct, 0);
        assert!(r.rejected.is_empty());
        assert_eq!(r.why, "deepest pool, fewest reclaims");
        assert_eq!(r.to_string(), r.why, "Display renders the stored reason for an uncontested choice");
    }
}
