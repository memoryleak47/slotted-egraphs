//! Open question: should matching also emit *refinements* -- substitutions
//! where slots that were merely allowed to be equal have been made equal?
//!
//! Two redundant slots reached through two different lookups may denote the same
//! slot: Def. 8's injectivity is per-lookup, so this is allowed under the strict
//! reading the matcher already implements.  `unify` performs exactly this merge
//! when a pattern forces it (see `control_*` below) -- what is missing is
//! offering it when the pattern does not force it.
//!
//! These are `#[ignore]`d because they describe wanted behaviour, not current
//! behaviour.

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

    assert!(
        mp_eq(&eg, "(k zero zero)", "(g zero zero)"),
        "the second rule never fires, because the first could not produce h($s,$s)"
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
