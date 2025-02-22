use crate::*;

pub fn rewrite_arith(eg: &mut EGraph<Arith>) {
    apply_rewrites(
        eg,
        &[
            beta(),
            eta(),
            my_let_unused(),
            let_var_same(),
            let_app(),
            let_lam_diff(),
            add_comm(),
            add_assoc1(),
            add_assoc2(),
            mul_comm(),
            mul_assoc1(),
            mul_assoc2(),
            distr1(),
            distr2(),
        ],
    );
}

fn beta() -> Rewrite<Arith> {
    let pat = "(app (lam $1 ?b) ?t)";
    let outpat = "(let $1 ?b ?t)";

    Rewrite::new("beta", pat, outpat)
}

fn eta() -> Rewrite<Arith> {
    let pat = "(lam $1 (app ?b (var $1)))";
    let outpat = "?b";

    Rewrite::new_if("eta", pat, outpat, |subst, _| {
        !subst["b"].slots().contains(&Slot::numeric(1))
    })
}

fn eta_expansion() -> Rewrite<Arith> {
    let pat = "?b";
    let outpat = "(lam $1 (app ?b (var $1)))";
    Rewrite::new("eta-expansion", pat, outpat)
}

fn my_let_unused() -> Rewrite<Arith> {
    let pat = "(let $1 ?b ?t)";
    let outpat = "?b";
    Rewrite::new_if("my-let-unused", pat, outpat, |subst, _| {
        !subst["b"].slots().contains(&Slot::numeric(1))
    })
}

fn let_var_same() -> Rewrite<Arith> {
    let pat = "(let $1 (var $1) ?e)";
    let outpat = "?e";
    Rewrite::new("let-var-same", pat, outpat)
}

fn let_app() -> Rewrite<Arith> {
    let pat = "(let $1 (app ?a ?b) ?e)";
    let outpat = "(app (let $1 ?a ?e) (let $1 ?b ?e))";
    Rewrite::new_if("let-app", pat, outpat, |subst, _| {
        subst["a"].slots().contains(&Slot::numeric(1))
            || subst["b"].slots().contains(&Slot::numeric(1))
    })
}

fn let_lam_diff() -> Rewrite<Arith> {
    let pat = "(let $1 (lam $2 ?b) ?e)";
    let outpat = "(lam $2 (let $1 ?b ?e))";
    Rewrite::new_if("let-lam-diff", pat, outpat, |subst, _| {
        subst["b"].slots().contains(&Slot::numeric(1))
    })
}

pub fn add_comm() -> Rewrite<Arith> {
    let pat = "(add ?a ?b)";
    let outpat = "(add ?b ?a)";
    Rewrite::new("add-comm", pat, outpat)
}

fn mul_comm() -> Rewrite<Arith> {
    let pat = "(mul ?a ?b)";
    let outpat = "(mul ?b ?a)";
    Rewrite::new("mul-comm", pat, outpat)
}

fn add_assoc1() -> Rewrite<Arith> {
    let pat = "(add ?a (add ?b ?c))";
    let outpat = "(add (add ?a ?b) ?c)";
    Rewrite::new("add-assoc1", pat, outpat)
}

fn add_assoc2() -> Rewrite<Arith> {
    let pat = "(add (add ?a ?b) ?c)";
    let outpat = "(add ?a (add ?b ?c))";
    Rewrite::new("add-assoc2", pat, outpat)
}

fn mul_assoc1() -> Rewrite<Arith> {
    let pat = "(mul ?a (mul ?b ?c))";
    let outpat = "(mul (mul ?a ?b) ?c)";
    Rewrite::new("mul-assoc1", pat, outpat)
}

fn mul_assoc2() -> Rewrite<Arith> {
    let pat = "(mul (mul ?a ?b) ?c)";
    let outpat = "(mul ?a (mul ?b ?c))";
    Rewrite::new("mul-assoc2", pat, outpat)
}

fn distr1() -> Rewrite<Arith> {
    let pat = "(mul ?a (add ?b ?c))";
    let outpat = "(add (mul ?a ?b) (mul ?a ?c))";
    Rewrite::new("distr1", pat, outpat)
}

fn distr2() -> Rewrite<Arith> {
    let pat = "(add (mul ?a ?b) (mul ?a ?c))";
    let outpat = "(mul ?a (add ?b ?c))";
    Rewrite::new("distr2", pat, outpat)
}
