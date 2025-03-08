use crate::*;

pub struct LambdaRealLambda;

impl Realization for LambdaRealLambda {
    fn step(eg: &mut EGraph<Lambda>) {
        rewrite_let(eg);
    }
}
// TODO: rewrite tests to use `Runner` and/or uncomment this test.
// Reason: deferred rebuilding behavior has to be rewritten into multiple files in this directory.
// unpack_tests!(LambdaRealLambda);

pub fn rewrite_let(eg: &mut EGraph<Lambda>) {
    apply_rewrites(
        eg,
        &[
            beta(),
            my_let_unused(),
            let_var_same(),
            let_app(),
            let_lam_diff(),
        ],
    );
}

fn beta() -> Rewrite<Lambda> {
    let pat = "(app (lam $1 ?b) ?t)";
    let outpat = "(let $1 ?b ?t)";
    Rewrite::new("beta", pat, outpat)
}

fn my_let_unused() -> Rewrite<Lambda> {
    let pat = "(let $1 ?b ?t)";
    let outpat = "?b";
    Rewrite::new_if("my-let-unused", pat, outpat, |subst, _| {
        !subst["b"].slots().contains(&Slot::numeric(1))
    })
}

fn let_var_same() -> Rewrite<Lambda> {
    let pat = "(let $1 (var $1) ?e)";
    let outpat = "?e";
    Rewrite::new("let-var-same", pat, outpat)
}

fn let_app() -> Rewrite<Lambda> {
    let pat = "(let $1 (app ?a ?b) ?e)";
    let outpat = "(app (let $1 ?a ?e) (let $1 ?b ?e))";
    Rewrite::new_if("let-app", pat, outpat, |subst, _| {
        subst["a"].slots().contains(&Slot::numeric(1))
            || subst["b"].slots().contains(&Slot::numeric(1))
    })
}

fn let_lam_diff() -> Rewrite<Lambda> {
    let pat = "(let $1 (lam $2 ?b) ?e)";
    let outpat = "(lam $2 (let $1 ?b ?e))";
    Rewrite::new_if("let-lam-diff", pat, outpat, |subst, _| {
        subst["b"].slots().contains(&Slot::numeric(1))
    })
}
