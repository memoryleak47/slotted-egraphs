#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

mod tst;
pub use tst::*;

mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

mod const_prop;
pub use const_prop::*;

define_language! {
    pub enum Arith {
        // lambda calculus:
        Lam(Bind<AppliedId>) = "lam",
        App(AppliedId, AppliedId) = "app",
        Var(Slot) = "var",
        Let(Bind<AppliedId>, AppliedId) = "let",

        Add(AppliedId, AppliedId) = "add",
        Mul(AppliedId, AppliedId) = "mul",

        // rest:
        Number(u32),
        Symbol(Symbol),
    }
}
