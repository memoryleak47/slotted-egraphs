use crate::*;

pub struct LambdaRealLambda;

impl Realization for LambdaRealLambda {
    fn step(eg: &mut EGraph<Lambda>) {
        rewrite_let(eg);
    }
}

unpack_tests!(LambdaRealLambda);


pub fn rewrite_let(eg: &mut EGraph<Lambda>) {
    apply_rewrites(eg, &[
        beta(),
        my_let_unused(),
        let_var_same(),
        let_app(),
        let_lam_diff(),
    ]);
}

fn beta() -> Rewrite<Lambda> {
    let pat = "(app (lam s1 ?b) ?t)";
    let outpat = "(let s1 ?t ?b)";
    Rewrite::new("beta", pat, outpat)
}

fn my_let_unused() -> Rewrite<Lambda> {
    let pat = "(let s1 ?t ?b)";
    let outpat = "?b";
    Rewrite::new_if("my-let-unused", pat, outpat, |subst| {
        !subst["b"].slots().contains(&Slot::new(1))
    })
}

fn let_var_same() -> Rewrite<Lambda> {
    let pat = "(let s1 ?e (var s1))";
    let outpat = "?e";
    Rewrite::new("let-var-same", pat, outpat)
}

fn let_app() -> Rewrite<Lambda> {
    let pat = "(let s1 ?e (app ?a ?b))";
    let outpat = "(app (let s1 ?e ?a) (let s1 ?e ?b))";
    Rewrite::new_if("let-app", pat, outpat, |subst| {
        subst["a"].slots().contains(&Slot::new(1)) || subst["b"].slots().contains(&Slot::new(1))
    })
}

fn let_lam_diff() -> Rewrite<Lambda> {
    let pat = "(let s1 ?e (lam s2 ?b))";
    let outpat = "(lam s2 (let s1 ?e ?b))";
    Rewrite::new_if("let-lam-diff", pat, outpat, |subst| {
        subst["b"].slots().contains(&Slot::new(1))
    })
}
