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
        Sum(AppliedId, Bind<Bind<AppliedId>>) = "sum",
    }
}
