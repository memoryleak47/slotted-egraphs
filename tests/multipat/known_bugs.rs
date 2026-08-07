//! The three `redundancy_matching_bug*` tests fail on the nested `ematch_all`
//! path.  Here each one is replayed with the *same* start term and the *same*
//! rules, but with every pattern rewritten into depth-1 equations and matched
//! with `multi_ematch`: `(f (g ?x) ?y)` becomes `?t == (f ?u ?y), ?u == (g ?x)`.
//!
//! Nothing is added to the e-graph that the original tests do not add
//! themselves, except in `bug3_..._once_subxx2_is_included`, which says so in
//! its name.
//!
//! Rewriting a pattern this way does not always preserve its meaning -- see
//! `flattening_is_not_faithful_for_a_sibling_slot_literal` at the bottom.  It
//! does preserve the meaning of these three, which is what these tests check.
//! It is not a licence to do it to every rule.

use crate::multipat::*;
use crate::*;

/// Apply one flattened rule once.
fn step(eg: &mut MPGraph, atoms: &[&str], root: &str, rhs: &str) {
    let pat: MultiPattern<MP> = MultiPattern::parse(&atoms.join(", ")).unwrap();
    let from = Pattern::PVar(root.to_string());
    let to: Pattern<MP> = Pattern::parse(rhs).unwrap();
    for s in multi_ematch(&pat, eg) {
        eg.union_instantiations(&from, &to, &s, None);
    }
}

/// `arith2::redundancy_matching_bug2`: start `f(zero, zero)`, rules `special`,
/// `subxx`, `subxx2`.  `subxx2` puts a `sub(t,t)` node into `zero`'s class,
/// `subxx` then makes that node's slot redundant, and `special` has to notice
/// that the slot reached through its left child and the one reached through its
/// right child can be the same slot.
#[test]
fn bug2_reaches_the_goal_under_multipat() {
    let mut eg = MPGraph::default();
    mp_add(&mut eg, "(f zero zero)");

    for _ in 0..6 {
        // special: (f (sub ?x ?x) (sub ?x ?x)) => zero
        step(
            &mut eg,
            &["?p == (f ?a ?b)", "?a == (sub ?x ?x)", "?b == (sub ?x ?x)"],
            "p",
            "zero",
        );
        // subxx: (sub ?x ?x) => zero
        step(&mut eg, &["?p == (sub ?x ?x)"], "p", "zero");
        // subxx2: zero => (sub (var $x) (var $x))
        step(&mut eg, &["?p == zero"], "p", "(sub (var $x) (var $x))");
    }
    assert!(mp_eq(&eg, "(f zero zero)", "zero"));
}

/// `arith2::redundancy_matching_bug3`: start `f(var $x, zero)`, rules `subxx`
/// and `special2`.
///
/// As written the test cannot pass on any matcher: neither rule can put a `sub`
/// node into `zero`'s class, so `special2`'s `(sub ?x ?x)` has nothing to match
/// against.  `subxx2`, which is what seeds that node in bug2, is missing from
/// the list.  With it added, multipat reaches the goal.
#[test]
fn bug3_reaches_the_goal_under_multipat_once_subxx2_is_included() {
    let mut eg = MPGraph::default();
    mp_add(&mut eg, "(f (var $x) zero)");

    for _ in 0..6 {
        step(&mut eg, &["?p == (sub ?x ?x)"], "p", "zero");
        step(&mut eg, &["?p == zero"], "p", "(sub (var $x) (var $x))"); // not in the test's rule list
                                                                        // special2: (f ?x (sub ?x ?x)) => zero
        step(
            &mut eg,
            &["?p == (f ?x ?q)", "?q == (sub ?x ?x)"],
            "p",
            "zero",
        );
    }
    assert!(mp_eq(&eg, "(f (var $x) zero)", "zero"));
}

/// Without `subxx2` nothing can put a `sub` node into the e-graph, so the goal
/// is out of reach for the depth-1 matcher as well.  This is a property of the
/// test, not of any matcher.
#[test]
fn bug3_as_written_is_unreachable() {
    let mut eg = MPGraph::default();
    mp_add(&mut eg, "(f (var $x) zero)");
    for _ in 0..6 {
        step(&mut eg, &["?p == (sub ?x ?x)"], "p", "zero");
        step(
            &mut eg,
            &["?p == (f ?x ?q)", "?q == (sub ?x ?x)"],
            "p",
            "zero",
        );
    }
    assert!(!mp_eq(&eg, "(f (var $x) zero)", "zero"));
}

/// Rewriting a nested pattern into depth-1 equations does not always mean the
/// same thing.  A nested pattern is matched against the e-graph using a single
/// renaming for the whole pattern; a multipattern picks a fresh renaming for
/// each equation.  So when a slot written in the pattern appears both under a
/// binder and outside it, the two forms disagree.
///
/// Here `$x` is bound by the `lam` in the left argument of the `app`, and also
/// written in the right argument, where that binder does not reach.  The nested
/// pattern correctly finds nothing.  The depth-1 version matches, binding the
/// binder's slot to the term's free slot -- the binder has escaped.  That is a
/// sound multipattern, it just asks a weaker question than the nested one.
///
/// So "flatten every rule and match it with `multi_ematch`" is not a safe
/// blanket transformation.  It is safe for the three tests above, and no rule
/// currently in this repo has this shape, but a flattener would need to reject
/// or rename patterns that reuse a bound slot outside its binder.
#[test]
fn flattening_is_not_faithful_for_a_sibling_slot_literal() {
    let mut eg = MPGraph::default();
    mp_add(&mut eg, "(app (lam $0 (var $0)) (var $1))");

    let nested: Vec<Subst> =
        ematch_all(&eg, &Pattern::parse("(app (lam $x ?c) (var $x))").unwrap());
    assert_eq!(nested.len(), 0, "a bound slot is not the term's free slot");

    let flat: MultiPattern<MP> =
        MultiPattern::parse("?p == (app ?a ?b), ?a == (lam $x ?c), ?b == (var $x)").unwrap();
    assert_eq!(multi_ematch(&flat, &eg).len(), 1, "the binder escaped");
}

/// For contrast: when the reused slot sits inside the binder's own subtree, the
/// two forms agree.  There the slot is carried in the child's applied id rather
/// than written in a separate equation, and the requirement that one node's
/// slots stay distinct rules the match out either way.  This is the shape of
/// `let-var-same` and of `props::flattening_through_a_binder`.
#[test]
fn flattening_is_faithful_when_the_slot_is_under_its_own_binder() {
    let mut eg = MPGraph::default();
    mp_add(&mut eg, "(lam $0 (var $1))");

    let nested: Vec<Subst> = ematch_all(&eg, &Pattern::parse("(lam $x (var $x))").unwrap());
    let flat: MultiPattern<MP> = MultiPattern::parse("?p == (lam $x ?b), ?b == (var $x)").unwrap();
    assert_eq!(nested.len(), 0);
    assert_eq!(multi_ematch(&flat, &eg).len(), 0);
}

/// `lambda::redundancy_matching_bug`: start
/// `(app (lam $x (var $x)) (lam $x (var $x)))`, one rule mapping it to
/// `(lam $x (var $x))`.
///
/// The pattern writes `$x` for two binders that have nothing to do with each
/// other.  The nested matcher gives the e-graph's two bound slots different
/// names and then cannot match the pattern's single `$x` against both.  The
/// depth-1 version does match: each equation looks its node up separately and
/// gets its own name for that node's bound slot, and setting both of those to
/// `$x` constrains nothing, since neither name is used anywhere else.
///
/// Note the pattern is used exactly as written.  Renaming a pattern's binders
/// apart automatically would not be an option: `beta`
/// (`(app (lam $1 ?b) ?t) => (let $1 ?b ?t)`) and `let-lam-diff` deliberately
/// write the same slot for a binder on the left and on the right, so that the
/// right-hand side captures it.
#[test]
fn lambda_bug_reaches_the_goal_under_multipat() {
    let mut eg = MPGraph::default();
    mp_add(&mut eg, "(app (lam $x (var $x)) (lam $x (var $x)))");
    assert_eq!(eg.ids().len(), 3);

    step(
        &mut eg,
        &[
            "?p == (app ?a ?b)",
            "?a == (lam $x ?c)",
            "?c == (var $x)",
            "?b == (lam $x ?d)",
            "?d == (var $x)",
        ],
        "p",
        "(lam $x (var $x))",
    );
    assert_eq!(eg.ids().len(), 2, "the rule should have fired");
}
