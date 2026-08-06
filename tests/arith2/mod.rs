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

pub fn get_all_rewrites2() -> Vec<Rewrite<Arith2>> {
    vec![
        special(),
        subxx(),
        subxx2(),
    ]
}

fn subxx() -> Rewrite<Arith2> {
    let pat = "(sub ?x ?x)";
    let outpat = "zero";

    Rewrite::new("subxx", pat, outpat)
}

fn subxx2() -> Rewrite<Arith2> {
    let pat = "zero";
    let outpat = "(sub (var $x) (var $x))";

    Rewrite::new("subxx2", pat, outpat)
}

fn special() -> Rewrite<Arith2> {
    let pat = "(f (sub ?x ?x) (sub ?x ?x))";
    let outpat = "zero";

    Rewrite::new("special", pat, outpat)
}

#[test]
fn redundancy_matching_bug2() {
    let x = "(f zero zero)";
    let y = "zero";

    assert_reaches(x, y, &get_all_rewrites2()[..], 3);
}
