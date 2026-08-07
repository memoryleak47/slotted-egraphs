//! Regression tests for multipattern matching.

use crate::multipat::*;
use crate::*;

/// `extend_subst` used to insert the child `AppliedId` without running it
/// through the slot union-find, so a slot that `matches_raw` had already merged
/// into a pattern slot survived under its old name.  For a binder that means the
/// body comes back referring to a fresh slot instead of the bound one, i.e. the
/// binding escapes.
#[test]
fn bound_slot_reaches_the_body() {
    let mut eg = MPGraph::default();
    mp_add(&mut eg, "(lam $0 (var $0))");

    let pat: MultiPattern<MP> = MultiPattern::parse("?p == (lam $v ?b)").unwrap();
    let ms = multi_ematch(&pat, &eg);
    assert_eq!(ms.len(), 1);

    let b = &ms[0]["b"];
    assert!(
        b.m.values().contains(&Slot::named("v")),
        "?b should refer to the binder's slot $v, got {:?}",
        b.m.values()
    );
}

/// Same, with a free slot alongside the bound one: exactly one of ?b's slots is
/// the pattern slot $v, the other is the (fresh) free slot of the lambda class.
#[test]
fn bound_slot_reaches_the_body_with_a_free_slot() {
    let mut eg = MPGraph::default();
    mp_add(&mut eg, "(lam $0 (f (var $0) (var $1)))");

    let pat: MultiPattern<MP> = MultiPattern::parse("?p == (lam $v ?b)").unwrap();
    let ms = multi_ematch(&pat, &eg);
    assert_eq!(ms.len(), 1);

    let vals = ms[0]["b"].m.values();
    assert_eq!(vals.len(), 2);
    assert!(
        vals.contains(&Slot::named("v")),
        "bound slot missing from ?b: {vals:?}"
    );
}

/// A pattern node carrying a slot literal, matched against a bound slot.  This
/// used to panic in `allows_directed_union` because the slot had no `slot_kind`
/// entry yet.
#[test]
fn binder_pattern_does_not_panic() {
    let mut eg = MPGraph::default();
    mp_add(&mut eg, "(lam $0 (var $0))");
    let pat: MultiPattern<MP> = MultiPattern::parse("?p == (lam $v ?q)").unwrap();
    assert_eq!(multi_ematch(&pat, &eg).len(), 1);
}

/// Same, against a redundant slot rather than a bound one.
#[test]
fn slot_literal_over_redundancy_does_not_panic() {
    let mut eg = MPGraph::default();
    mp_union(&mut eg, "(var $0)", "(var $1)"); // the Var class loses its slot
    let pat: MultiPattern<MP> = MultiPattern::parse("?p == (var $s)").unwrap();
    assert_eq!(multi_ematch(&pat, &eg).len(), 1);
}

/// Two redundant slots of one node must stay distinct: a renaming is a
/// bijection, so `(f ?a ?a)` does not match a node whose two redundant slots
/// would have to collide.
#[test]
fn same_node_redundant_slots_stay_distinct() {
    let mut eg = MPGraph::default();
    mp_union(&mut eg, "(f (var $0) (var $1))", "zero");

    let both: MultiPattern<MP> = MultiPattern::parse("?p == (f ?a ?a)").unwrap();
    assert_eq!(multi_ematch(&both, &eg).len(), 0);

    let apart: MultiPattern<MP> = MultiPattern::parse("?p == (f ?a ?b)").unwrap();
    assert_eq!(multi_ematch(&apart, &eg).len(), 1);
}

/// A class's own live slots are distinct, so a pattern that would have to
/// identify two of them does not match.
#[test]
fn live_slots_of_one_class_stay_distinct() {
    let mut eg = MPGraph::default();
    mp_add(&mut eg, "(k (var $0) (var $1))"); // k keeps both slots -- no union, no redundancy

    let apart: MultiPattern<MP> = MultiPattern::parse("?p == (k ?u ?v)").unwrap();
    assert_eq!(multi_ematch(&apart, &eg).len(), 1);

    let together: MultiPattern<MP> = MultiPattern::parse("?p == (k ?u ?u)").unwrap();
    assert_eq!(
        multi_ematch(&together, &eg).len(),
        0,
        "k's two live slots are distinct, so ?u cannot be both"
    );
}
