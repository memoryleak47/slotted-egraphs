#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

define_language! {
    pub enum Arith2 {
        Var(Slot) = "var",
        F(AppliedId, AppliedId) = "f",
        Sub(AppliedId, AppliedId) = "sub",
        Zero() = "zero",
    }
}

fn subxx() -> Rewrite<Arith2> { Rewrite::new("subxx", "(sub ?x ?x)", "zero") }
fn subxx2() -> Rewrite<Arith2> { Rewrite::new("subxx2", "zero", "(sub (var $x) ($var x))") }
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
