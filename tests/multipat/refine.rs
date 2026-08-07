//! Open question: when matching leaves two slots free to be either equal or
//! different, should it return both answers instead of only the one that keeps
//! them apart?
//!
//! A redundant slot stands for "any slot", so when a match reaches two of them
//! through two separate node lookups, setting them to the same slot is as valid
//! an answer as setting them to different ones.  Def. 8 does require the slots
//! of any one looked-up node to stay distinct, but that requirement applies to
//! each lookup on its own, so it says nothing about two slots from two lookups.
//!
//! `unify` already makes exactly this merge when a pattern demands it -- see the
//! `control_*` tests, which write the same variable in both equations.  What is
//! missing is making it when the pattern leaves the choice open, in which case
//! only the keep-them-apart answer is returned and the other is lost.
//!
//! These two tests are `#[ignore]`d: they describe what we think should happen,
//! not what happens today.

use crate::multipat::*;
use crate::*;

/// sub(x,x) = zero gives zero's class a node with one redundant slot, and
/// k(zero, zero) reaches it through two independent child positions.
fn setup(eg: &mut MPGraph) {
    mp_union(eg, "(sub (var $9) (var $9))", "zero");
    mp_add(eg, "(k zero zero)");
    mp_add(eg, "(h (var $0) (var $1))");
    mp_add(eg, "(h (var $0) (var $0))");
    mp_add(eg, "(g zero zero)");
}

#[test]
#[ignore = "wanted: matching should also return refinements of a match"]
fn refinement_is_offered() {
    let mut eg = MPGraph::default();
    setup(&mut eg);
    mp_saturate(
        &mut eg,
        &["?p == (k ?a ?b)", "?a == (sub ?u ?u)", "?b == (sub ?v ?v)"],
        "p",
        "(h ?u ?v)",
    );

    // the general match
    assert!(mp_eq(&eg, "(h (var $0) (var $1))", "(k zero zero)"));
    // the refinement ?u == ?v, which is just as valid
    assert!(
        mp_eq(&eg, "(h (var $0) (var $0))", "(k zero zero)"),
        "the refinement ?u == ?v was never offered"
    );
}

/// Control: the same equality does hold -- write ?u in both atoms and it appears.
#[test]
fn control_refinement_stated_in_the_pattern() {
    let mut eg = MPGraph::default();
    setup(&mut eg);
    mp_saturate(
        &mut eg,
        &["?p == (k ?a ?b)", "?a == (sub ?u ?u)", "?b == (sub ?u ?u)"],
        "p",
        "(h ?u ?u)",
    );
    assert!(mp_eq(&eg, "(h (var $0) (var $0))", "(k zero zero)"));
}

/// Why it matters: without the refinement a second rule that needed it never
/// fires, so the goal is unreachable.  Unioning h with two distinct slots into
/// the slotless k(zero,zero) makes both slots redundant, and `(h ?x ?x)` then
/// correctly refuses to match one node's two redundant slots.
#[test]
#[ignore = "wanted: matching should also return refinements of a match"]
fn refinement_is_needed_by_a_later_rule() {
    let mut eg = MPGraph::default();
    setup(&mut eg);
    mp_saturate(
        &mut eg,
        &["?p == (k ?a ?b)", "?a == (sub ?u ?u)", "?b == (sub ?v ?v)"],
        "p",
        "(h ?u ?v)",
    );
    mp_saturate(&mut eg, &["?q == (h ?x ?x)"], "q", "(g zero zero)");

    // note rule 2 *does* fire, on the standalone h($0,$0) that `setup` adds --
    // what never happens is it firing on k's class, because rule 1 only ever put
    // an h with two *distinct* slots there.
    assert!(
        mp_eq(&eg, "(k zero zero)", "(g zero zero)"),
        "rule 2 did not fire on k's class, because rule 1 could not produce h($s,$s) there"
    );
}

/// Control for the chain: with the refinement written into the first rule, the
/// second rule fires and the goal is reached.
#[test]
fn control_chain_with_refinement_stated() {
    let mut eg = MPGraph::default();
    setup(&mut eg);
    mp_saturate(
        &mut eg,
        &["?p == (k ?a ?b)", "?a == (sub ?u ?u)", "?b == (sub ?u ?u)"],
        "p",
        "(h ?u ?u)",
    );
    mp_saturate(&mut eg, &["?q == (h ?x ?x)"], "q", "(g zero zero)");
    assert!(mp_eq(&eg, "(k zero zero)", "(g zero zero)"));
}
