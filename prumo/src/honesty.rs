//! `honesty` — **the fixed core the optimizer may NOT tune.**
//!
//! lapidar tunes knobs. The honesty gate does not have knobs. Two things are
//! `const`, not fields, so no tick, no proposal, and no accept-if-improved move
//! can ever reach them:
//!
//! 1. **The tier ladder + never-round-up.** An axis may only *claim* the tier its
//!    evidence earns ([`can_claim`]); rounding a `DesignLiveTodo` up to `Shipped`
//!    is not a knob the optimizer can turn — [`attest_tier`] returns the axis'
//!    const catalog tier, and there is no promote path. "SHIPPED vs LiveTODO" is a
//!    fact about the fleet, never a value the optimizer improves toward.
//! 2. **The accept margin.** [`crate::lapidar::TUNE_MARGIN`] is a `const`; a tune
//!    that does not clear it is reverted. The optimizer cannot lower the margin to
//!    make a non-improving move "improve" — that would be self-flattery, exactly
//!    the round-up the seal forbids.
//!
//! This is the UNREPRESENTABILITY model applied to the optimizer itself: a
//! rounded-up claim, and a margin-dodging accept, have **no code path**. The
//! optimizer's honesty is not policed at runtime — it is structural.

use crate::axis::Tier;

/// Whether an axis may CLAIM `claimed` given its actual `evidence` tier. You may
/// UNDER-claim (state a lower tier than the evidence — false modesty is allowed,
/// if discouraged); you may NEVER OVER-claim (round up). Because [`Tier`] orders
/// by evidence (`Shipped > AuthoredProven > DesignLiveTodo`), the rule is exactly
/// `claimed <= evidence`.
///
/// This is the **never-round-up** law as a total function. It is used by the
/// catalog matrix to refuse any row whose stated tier exceeds its composition
/// evidence.
#[must_use]
pub const fn can_claim(claimed: Tier, evidence: Tier) -> bool {
    // `Tier: Ord` — derived, evidence-ordered. `<=` is const-evaluable via the
    // discriminant order (DesignLiveTodo < AuthoredProven < Shipped).
    (claimed as u8) <= (evidence as u8)
}

/// Attest an axis' tier — return exactly its const catalog tier, never a rounded
/// value. This is the ONLY tier-attestation surface, and it is the identity: the
/// optimizer has no method that raises it. Naming the honest tier IS the whole
/// discipline; there is no path that "improves" it.
#[must_use]
pub const fn attest_tier(catalog_tier: Tier) -> Tier {
    catalog_tier
}

/// The two things that are FIXED (not tunable) — named as data so a test can
/// assert the honesty core is closed. If a future edit tries to make either a
/// knob, the [`FIXED_INVARIANTS`] test row it violates is the forcing-function.
pub const FIXED_INVARIANTS: [&str; 2] = [
    "tier-ladder + never-round-up (attest_tier is the identity; can_claim forbids over-claim)",
    "accept-margin (lapidar::TUNE_MARGIN is a const; a sub-margin move is reverted, never re-scored)",
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::axis::Tier;
    use crate::lapidar::TUNE_MARGIN;

    #[test]
    fn you_may_never_round_up() {
        // DesignLiveTodo CANNOT claim Shipped or AuthoredProven — the round-up.
        assert!(!can_claim(Tier::Shipped, Tier::DesignLiveTodo), "design cannot claim shipped");
        assert!(!can_claim(Tier::AuthoredProven, Tier::DesignLiveTodo), "design cannot claim authored");
        assert!(!can_claim(Tier::Shipped, Tier::AuthoredProven), "authored cannot claim shipped");
    }

    #[test]
    fn you_may_claim_exactly_or_under() {
        // Claiming the true tier is fine; under-claiming is allowed (never round up).
        assert!(can_claim(Tier::Shipped, Tier::Shipped));
        assert!(can_claim(Tier::DesignLiveTodo, Tier::Shipped), "under-claim allowed");
        assert!(can_claim(Tier::AuthoredProven, Tier::Shipped));
    }

    #[test]
    fn attest_tier_is_the_identity_no_promote_path() {
        // The only attestation surface returns exactly the const tier — there is
        // no way to attest higher. Naming the honest tier is the whole discipline.
        for t in [Tier::DesignLiveTodo, Tier::AuthoredProven, Tier::Shipped] {
            assert_eq!(attest_tier(t), t, "attest must not round up {:?}", t);
        }
    }

    #[test]
    fn the_accept_margin_is_a_const_not_a_knob() {
        // TUNE_MARGIN is a const the optimizer cannot lower — it is the same value
        // as the shipped lapidar core, imported, not re-declared as a field.
        assert_eq!(TUNE_MARGIN, 50, "the fixed margin matches the shipped lapidar core");
    }

    #[test]
    fn the_fixed_core_names_exactly_the_two_untunable_things() {
        assert_eq!(FIXED_INVARIANTS.len(), 2);
        assert!(FIXED_INVARIANTS[0].contains("never-round-up"));
        assert!(FIXED_INVARIANTS[1].contains("margin"));
    }
}
