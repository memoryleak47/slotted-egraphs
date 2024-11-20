use crate::*;

mod build;
pub use build::*;

mod my_cost;
pub use my_cost::*;

mod tst;
pub use tst::*;

mod normalize;
pub use normalize::*;

mod realization;
pub use realization::*;

mod subst;
pub use subst::*;

mod step;
pub use step::*;

mod big_step;
pub use big_step::*;

mod lambda_small_step;
pub use lambda_small_step::*;

mod let_small_step;
pub use let_small_step::*;

mod native;
pub use native::*;

define_language! {
    pub enum Lambda {
        Lam(Bind<AppliedId>) = "lam",
        App(AppliedId, AppliedId) = "app",
        Var(Slot) = "var",
        Let(Bind<AppliedId>, AppliedId) = "let",
    }
}
