use crate::*;

mod build;
pub use build::*;

mod my_cost;
pub use my_cost::*;

mod tst;

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

#[test]
fn redundancy_matching_bug() {
    let mut eg: EGraph<Lambda> = EGraph::new(());
    let t = RecExpr::parse("(app (lam $x (var $x)) (lam $x (var $x)))").unwrap();
    eg.add_expr(t);

    assert_eq!(eg.total_number_of_nodes(), 3);

    let r: Rewrite<Lambda> = rw!("compose_identity"; "(app (lam $x (var $x)) (lam $x (var $x)))" => "(lam $x (var $x))");
    apply_rewrites(&mut eg, &[r]);

    eg.dump();

    // NOTE: this is a bug. The rule `compose_identity` didn't fire even though it precisely matches the added term `t`.
    // The reason for that is that during e-matching
    // (app (lam $x (var $x)) (lam $x (var $x)))
    // gets rewritten to
    // (app (lam $x1 (var $x1)) (lam $x2 (var $x2)))
    // to avoid naming collisions.

    assert_eq!(eg.total_number_of_nodes(), 2);
}
