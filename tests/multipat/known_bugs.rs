//! The three `redundancy_matching_bug*` tests fail on the nested `ematch_all`
//! path.  Here each one is replayed with the *same* start term and the *same*
//! rules, but with every pattern flattened into depth-1 equations and matched
//! with `multi_ematch`.  Flattening is mechanical: `(f (g ?x) ?y)` becomes
//! `?t == (f ?u ?y), ?u == (g ?x)`.
//!
//! Nothing is seeded that the original tests do not add themselves.

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
/// `subxx`, `subxx2`.  `subxx2` seeds `zero`'s class with `sub(t,t)`, `subxx`
/// makes that slot redundant, and `special` then has to merge the redundant
/// slots reached through its two children.
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

/// Without `subxx2` nothing can seed the `sub` node, so the goal is out of reach
/// for the multipattern matcher too.  This is a property of the test, not of the
/// matcher.
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

/// `lambda::redundancy_matching_bug`: start
/// `(app (lam $x (var $x)) (lam $x (var $x)))`, one rule mapping it to
/// `(lam $x (var $x))`.  The pattern uses one slot name for two independent
/// binders; the nested matcher renames the e-graph side apart and then cannot
/// match, while the literal flattening does, because the two bound slots are
/// freshened per lookup and merging both into the one pattern slot costs
/// nothing.
///
/// Note this needs no renaming of the pattern.  Renaming a pattern's binders
/// apart automatically would not be safe: `beta`
/// (`(app (lam $1 ?b) ?t) => (let $1 ?b ?t)`) and `let-lam-diff` deliberately
/// reuse an lhs binder slot in the rhs so that `?b` is captured.
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
