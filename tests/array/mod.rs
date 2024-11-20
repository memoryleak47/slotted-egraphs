#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

mod parse;
pub use parse::*;

mod tst;

// This is a close-as possible to SymbolLang to be comparable with https://github.com/Bastacyclop/egg-sketches/blob/main/tests/maps.rs
define_language! {
    pub enum Array {
        Lam(Bind<AppliedId>) = "lam",
        App(AppliedId, AppliedId) = "app",
        Var(Slot) = "var",
        Let(Bind<AppliedId>, AppliedId) = "let",

        Symbol(Symbol),
    }
}
