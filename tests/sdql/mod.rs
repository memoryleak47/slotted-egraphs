#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

mod rewrite;
pub use rewrite::*;

define_language! {
    pub enum Sdql {
        Lam(Bind<AppliedId>) = "lambda",
        Var(Slot) = "var",
        Sing(AppliedId, AppliedId) = "sing",
        Equality(AppliedId, AppliedId) = "eq",
        Get(AppliedId, AppliedId) = "get",
        IfThen(AppliedId, AppliedId) = "ifthen",
        Sum(AppliedId, Bind<Bind<AppliedId>>) = "sum",
        Merge(AppliedId, AppliedId, Bind<Bind<Bind<AppliedId>>>) = "merge",
        Let(AppliedId, Bind<AppliedId>) = "let",
    }
}
