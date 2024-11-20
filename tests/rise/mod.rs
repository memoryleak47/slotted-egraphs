use crate::*;

mod tst;
pub use tst::*;

mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

define_language! {
    pub enum Rise {
        // lambda calculus:
        Lam(Bind<AppliedId>) = "lam",
        App(AppliedId, AppliedId) = "app",
        Var(Slot) = "var",
        Let(Bind<AppliedId>, AppliedId) = "let",

        // rest:
        Number(u32),
        Symbol(Symbol),
    }
}
