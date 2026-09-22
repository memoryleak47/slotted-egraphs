#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

define_language! {
    pub enum Arith2 {
        Var(Slot) = "var",
        F(AppliedId, AppliedId) = "f",
        Sub(AppliedId, AppliedId) = "sub",
        Lam(Bind<AppliedId>) = "lam",
        Zero() = "zero",
    }
}

fn subxx() -> Rewrite<Arith2> { Rewrite::new("subxx", "(sub ?x ?x)", "zero") }
fn subxx2() -> Rewrite<Arith2> { Rewrite::new("subxx2", "zero", "(sub (var $x) (var $x))") }
fn special() -> Rewrite<Arith2> { Rewrite::new("special", "(f (sub ?x ?x) (sub ?x ?x))", "zero") }
fn special2() -> Rewrite<Arith2> { Rewrite::new("special2", "(f ?x (sub ?x ?x))", "zero") }

#[test]
fn redundancy_matching_bug2() {
    let x = "(f zero zero)";
    let y = "zero";

    let rewrites = &[
        special(),
        subxx(),
        subxx2(),
    ];
    assert_reaches(x, y, rewrites, 3);
}

#[test]
// In this version of the bug, a fresh/redundant variable has to alias a non-redundant variable. So that is also possible.
fn redundancy_matching_bug3() {
    let x = "(f (var $x) zero)";
    let y = "zero";

    let rewrites = &[
        subxx(),
        special2(),
    ];
    assert_reaches(x, y, rewrites, 3);
}



#[test]
fn multipat_test() {
    let mut eg: EGraph<Arith2> = EGraph::new(());
    eg.add_expr(RecExpr::parse("(f (var $x) zero)").unwrap());
    let pat: MultiPattern<Arith2> = MultiPattern::parse("?x == (f ?a ?b), ?b == zero").unwrap();
    let matches = multi_ematch(&pat, &eg);
    dbg!(&matches);
    assert_eq!(matches.len(), 1);
}

#[test]
fn multipat_test2() {
    let mut eg: EGraph<Arith2> = EGraph::new(());

    eg.add_expr(RecExpr::parse("(f (var $x) (sub (var $y) (var $y)))").unwrap());

    let a = eg.add_expr(RecExpr::parse("(sub (var $z) (var $z))").unwrap());
    let b = eg.add_expr(RecExpr::parse("zero").unwrap());
    eg.union(&a, &b);

    let pat: MultiPattern<Arith2> = MultiPattern::parse("?out == (f ?a ?b), ?b == (sub ?a ?a)").unwrap();
    let matches = multi_ematch(&pat, &eg);
    dbg!(&matches);
    assert_eq!(matches.len(), 1);
}

#[test]
fn multipat_test3() {
    let mut eg: EGraph<Arith2> = EGraph::new(());

    eg.add_expr(RecExpr::parse("(var $x)").unwrap());

    let pat: MultiPattern<Arith2> = MultiPattern::parse("?out == (var $y)").unwrap();
    let matches = multi_ematch(&pat, &eg);
    dbg!(&matches);
    assert_eq!(matches.len(), 1);
    let m = &matches[0];
    assert_eq!(m["out"].m.values(), std::iter::once(Slot::named("y")).collect());
}

#[test]
fn multipat_test4() {
    let mut eg: EGraph<Arith2> = EGraph::new(());

    eg.add_expr(RecExpr::parse("(f (var $x) (sub (var $y) (var $y)))").unwrap());

    let a = eg.add_expr(RecExpr::parse("(sub (var $z) (var $z))").unwrap());
    let b = eg.add_expr(RecExpr::parse("zero").unwrap());
    eg.union(&a, &b);

    let pat: MultiPattern<Arith2> = MultiPattern::parse("?out == (f ?a ?b), ?b == (sub ?c ?a), ?c == (var $x)").unwrap();
    let matches = multi_ematch(&pat, &eg);
    dbg!(&matches);
    assert_eq!(matches.len(), 1);
}

#[test]
fn multipat_test5() {
    let mut eg: EGraph<Arith2> = EGraph::new(());

    eg.add_expr(RecExpr::parse("(sub (var $x) (var $y))").unwrap());

    let pat: MultiPattern<Arith2> = MultiPattern::parse("?out == (sub ?a ?a), ?a == (var $a)").unwrap();
    let matches = multi_ematch(&pat, &eg);
    dbg!(&matches);
    assert!(matches.is_empty());
}

#[test]
fn multipat_test6() {
    let mut eg: EGraph<Arith2> = EGraph::new(());

    eg.add_expr(RecExpr::parse("(sub (var $x) (var $y))").unwrap());

    let pat: MultiPattern<Arith2> = MultiPattern::parse("?out == (sub ?a ?a)").unwrap();
    let matches = multi_ematch(&pat, &eg);
    dbg!(&matches);
    assert!(matches.is_empty());
}

#[test]
fn multipat_test7() {
    let mut eg: EGraph<Arith2> = EGraph::new(());

    eg.add_expr(RecExpr::parse("(sub (var $x) (var $x))").unwrap());

    let pat: MultiPattern<Arith2> = MultiPattern::parse("?out == (sub ?a ?a), ?a == (var $a)").unwrap();
    let matches = multi_ematch(&pat, &eg);
    dbg!(&matches);
    assert_eq!(matches.len(), 1);
}

#[test]
fn multipat_test8() {
    let mut eg: EGraph<Arith2> = EGraph::new(());

    eg.add_expr(RecExpr::parse("(sub (var $x) (var $x))").unwrap());

    let pat: MultiPattern<Arith2> = MultiPattern::parse("?out == (sub ?a ?a)").unwrap();
    let matches = multi_ematch(&pat, &eg);
    dbg!(&matches);
    assert_eq!(matches.len(), 1);
}

#[test]
fn multipat_test9() {
    let mut eg: EGraph<Arith2> = EGraph::new(());

    eg.add_expr(RecExpr::parse("(f (var $x) (sub (var $y) (var $y)))").unwrap());

    let a = eg.add_expr(RecExpr::parse("(sub (var $z) (var $z))").unwrap());
    let b = eg.add_expr(RecExpr::parse("zero").unwrap());
    eg.union(&a, &b);

    let pat: MultiPattern<Arith2> = MultiPattern::parse("?out == (f ?a ?b), ?b == (sub ?c ?c)").unwrap();
    let matches = multi_ematch(&pat, &eg);
    dbg!(&matches);
    assert_eq!(matches.len(), 2);
    // There are two matches because ?a and ?c can agree or disagree on their slot. f($x, $y-$y) versus f($x, $x-$x).
}

#[test]
fn multipat_test10() { // testcase found by oflatt-claude.
    let mut eg: EGraph<Arith2> = EGraph::new(());

    eg.add_expr(RecExpr::parse("(lam $x (var $x))").unwrap());

    let pat: MultiPattern<Arith2> = MultiPattern::parse("?out == (lam $v ?b)").unwrap();
    let matches = multi_ematch(&pat, &eg);
    dbg!(&matches);
    assert_eq!(matches.len(), 1);

    let m = &matches[0];
    let vals = &m["b"].m.values();
    let correct_vals = std::iter::once(Slot::named("v")).collect();

    assert_eq!(*vals, correct_vals);
}

#[test]
// `final_refine` may read a binder's bound slot as the name of one of the term's free
// variables. As a MATCH that is a fine alpha-variant of the term. But a rewrite whose
// right-hand side rebinds that slot over a variable matched outside the binder then
// captures it. The nested matcher never does this: its bound slots stay injective.
//
//     (f (lam $0 (sub (var $0) (var $0))) (var $2))          the term
//     (f (lam $y ?body) ?e)  =>  (lam $y (f ?body ?e))       the rewrite
//
// With `$y` read as `$2` the right-hand side is `(lam $2 (f (sub $2 $2) $2))`: `?e`
// captured, a closed term, and the class stops depending on `$2`. On the paper's
// array language the same happens through `let-lam-diff` at zero parameters, six
// rounds in, after which every class is one class.
fn multipat_rhs_binder_captures_a_free_variable() {
    let term = "(f (lam $0 (sub (var $0) (var $0))) (var $2))";
    let lhs = "(f (lam $y ?body) ?e)";
    let rhs = "(lam $y (f ?body ?e))";
    let sound = "(lam $1 (f (sub (var $1) (var $1)) (var $2)))";
    let captured = "(lam $1 (f (sub (var $1) (var $1)) (var $1)))";

    // nested matcher: the sound result, and nothing else
    let mut eg: EGraph<Arith2> = EGraph::new(());
    let start = eg.add_expr(RecExpr::parse(term).unwrap());
    apply_rewrites(&mut eg, &[Rewrite::new("push", lhs, rhs)]);
    let sound_id = eg.add_expr(RecExpr::parse(sound).unwrap());
    let captured_id = eg.add_expr(RecExpr::parse(captured).unwrap());
    assert!(eg.eq(&start, &sound_id));
    assert!(!eg.eq(&start, &captured_id));

    // multipattern matcher: the same rule, flattened
    let mut eg: EGraph<Arith2> = EGraph::new(());
    let start = eg.add_expr(RecExpr::parse(term).unwrap());
    let pat: MultiPattern<Arith2> = MultiPattern::parse("?p == (f ?l ?e), ?l == (lam $y ?body)").unwrap();
    let matches = multi_ematch(&pat, &eg);
    let (from, to) = (Pattern::PVar("p".to_string()), Pattern::parse(rhs).unwrap());
    for s in &matches {
        eg.union_instantiations(&from, &to, s, None);
    }
    let sound_id = eg.add_expr(RecExpr::parse(sound).unwrap());
    let captured_id = eg.add_expr(RecExpr::parse(captured).unwrap());
    assert!(eg.eq(&start, &sound_id));
    assert!(
        !eg.eq(&start, &captured_id),
        "{} matches, one reading the binder as the free $2: {matches:?}",
        matches.len()
    );
}
