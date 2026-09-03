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

    assert_eq!(eg.ids().len(), 3);

    // The pattern writes one `$x` for two independent binders. E-matching freshens each
    // binder's bound slot, so the two occurrences meet different egraph slots:
    // (app (lam $x (var $x)) (lam $x (var $x)))  is matched against
    // (app (lam $x1 (var $x1)) (lam $x2 (var $x2))).
    // Identifying $x1 with $x2 is a renaming, both being bound, so the rule does fire;
    // `ematch` keeps only the pattern's FREE slots injective.
    let r: Rewrite<Lambda> = rw!("compose_identity"; "(app (lam $x (var $x)) (lam $x (var $x)))" => "(lam $x (var $x))");
    apply_rewrites(&mut eg, &[r]);

    eg.dump();

    // `compose_identity` fired, so the `app` class and the identity `lam` are one.
    assert_eq!(eg.ids().len(), 2);
}
