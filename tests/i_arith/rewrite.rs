use crate::*;
use crate::i_arith::build::*;

pub fn rewrite_arith(eg: &mut EGraph<ArithENode>) {
    apply_rewrites(eg, &[
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
    ])
}

fn beta() -> Rewrite<ArithENode> {
    let pat = "(app (lam s1 ?b) ?t)";
    let outpat = "(let s1 ?t ?b)";

    Rewrite::new("beta", pat, outpat)
}

fn eta() -> Rewrite<ArithENode> {
    let pat = "(lam s1 (app ?b (var s1)))";
    let outpat = "?b";

    Rewrite::new_if("eta", pat, outpat, |subst| {
        !subst["?b"].slots().contains(&Slot::new(1))
    })
}

fn eta_expansion() -> Rewrite<ArithENode> {
    let pat = "?b";
    let outpat = "(lam s1 (app ?b (var s1)))";
    Rewrite::new("eta-expansion", pat, outpat)
}

fn my_let_unused() -> Rewrite<ArithENode> {
    let pat = "(let s1 ?t ?b)";
    let outpat = "?b";
    Rewrite::new_if("my-let-unused", pat, outpat, |subst| {
        !subst["?b"].slots().contains(&Slot::new(1))
    })
}

fn let_var_same() -> Rewrite<ArithENode> {
    let pat = "(let s1 ?e (var s1))";
    let outpat = "?e";
    Rewrite::new("let-var-same", pat, outpat)
}

fn let_app() -> Rewrite<ArithENode> {
    let pat = "(let s1 ?e (app ?a ?b))";
    let outpat = "(app (let s1 ?e ?a) (let s1 ?e ?b))";
    Rewrite::new_if("let-app", pat, outpat, |subst| {
        subst["?a"].slots().contains(&Slot::new(1)) || subst["?b"].slots().contains(&Slot::new(1))
    })
}

fn let_lam_diff() -> Rewrite<ArithENode> {
    let pat = "(let s1 ?e (lam s2 ?b))";
    let outpat = "(lam s2 (let s1 ?e ?b))";
    Rewrite::new_if("let-lam-diff", pat, outpat, |subst| {
        subst["?b"].slots().contains(&Slot::new(1))
    })
}

pub fn add_comm() -> Rewrite<ArithENode> {
    let pat = "(add ?a ?b)";
    let outpat = "(add ?b ?a)";
    Rewrite::new("add-comm", pat, outpat)
}

fn mul_comm() -> Rewrite<ArithENode> {
    let pat = "(mul ?a ?b)";
    let outpat = "(mul ?b ?a)";
    Rewrite::new("mul-comm", pat, outpat)
}

fn add_assoc1() -> Rewrite<ArithENode> {
    let pat = "(add ?a (add ?b ?c))";
    let outpat = "(add (add ?a ?b) ?c)";
    Rewrite::new("add-assoc1", pat, outpat)
}

fn add_assoc2() -> Rewrite<ArithENode> {
    let pat = "(add (add ?a ?b) ?c)";
    let outpat = "(add ?a (add ?b ?c))";
    Rewrite::new("add-assoc2", pat, outpat)
}

fn mul_assoc1() -> Rewrite<ArithENode> {
    let pat = "(mul ?a (mul ?b ?c))";
    let outpat = "(mul (mul ?a ?b) ?c)";
    Rewrite::new("mul-assoc1", pat, outpat)
}

fn mul_assoc2() -> Rewrite<ArithENode> {
    let pat = "(mul (mul ?a ?b) ?c)";
    let outpat = "(mul ?a (mul ?b ?c))";
    Rewrite::new("mul-assoc2", pat, outpat)
}

fn distr1() -> Rewrite<ArithENode> {
    let pat = "(mul ?a (add ?b ?c))";
    let outpat = "(add (mul ?a ?b) (mul ?a ?c))";
    Rewrite::new("distr1", pat, outpat)
}

fn distr2() -> Rewrite<ArithENode> {
    let pat = "(add (mul ?a ?b) (mul ?a ?c))";
    let outpat = "(mul ?a (add ?b ?c))";
    Rewrite::new("distr2", pat, outpat)
}
