use crate::*;
use crate::i_arith::build::*;

pub fn rewrite_arith(eg: &mut EGraph<ArithENode>) {
    beta(eg);
    eta(eg);
    // eta_expansion(eg);

    my_let_unused(eg);
    let_var_same(eg);
    let_app(eg);
    let_lam_diff(eg);

    add_comm(eg);
    add_assoc(eg);

    mul_comm(eg);
    mul_assoc(eg);

    distr(eg);
}

fn beta(eg: &mut EGraph<ArithENode>) {
    // (\s1. ?b) ?t
    let pat = app(lam(1, pvar("?b")), pvar("?t"));

    // let s1 ?t ?b
    let outpat = let_(1, pvar("?t"), pvar("?b"));

    rewrite(eg, pat, outpat);
}

fn eta(eg: &mut EGraph<ArithENode>) {
    // \s1. ?b s1
    let pat = lam(1, app(pvar("?b"), var(1)));

    // ?b
    let outpat = pvar("?b");

    rewrite_if(eg, pat, outpat, |subst| {
        !subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn eta_expansion(eg: &mut EGraph<ArithENode>) {
    // ?b
    let pat = pvar("?b");

    // \s1. ?b s1
    let outpat = lam(1, app(pvar("?b"), var(1)));

    rewrite(eg, pat, outpat);
}

fn my_let_unused(eg: &mut EGraph<ArithENode>) {
    let pat = let_(1, pvar("?t"), pvar("?b"));
    let outpat = pvar("?b");
    rewrite_if(eg, pat, outpat, |subst| {
        !subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn let_var_same(eg: &mut EGraph<ArithENode>) {
    let pat = let_(1, pvar("?e"), var(1));
    let outpat = pvar("?e");
    rewrite(eg, pat, outpat);
}

fn let_app(eg: &mut EGraph<ArithENode>) {
    let pat = let_(1, pvar("?e"), app(pvar("?a"), pvar("?b")));
    let outpat = app(
        let_(1, pvar("?e"), pvar("?a")),
        let_(1, pvar("?e"), pvar("?b"))
    );
    rewrite_if(eg, pat, outpat, |subst| {
        subst["?a"].slots().contains(&Slot::new(1)) || subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn let_lam_diff(eg: &mut EGraph<ArithENode>) {
    let pat = let_(1, pvar("?e"), lam(2, pvar("?b")));
    let outpat = lam(2,
        let_(1, pvar("?e"), pvar("?b")),
    );
    rewrite_if(eg, pat, outpat, |subst| {
        subst["?b"].slots().contains(&Slot::new(1))
    });
}

pub fn add_comm(eg: &mut EGraph<ArithENode>) {
    let pat = add2(pvar("?a"), pvar("?b"));
    let outpat = add2(pvar("?b"), pvar("?a"));
    rewrite(eg, pat, outpat);
}

fn mul_comm(eg: &mut EGraph<ArithENode>) {
    let pat = mul2(pvar("?a"), pvar("?b"));
    let outpat = mul2(pvar("?b"), pvar("?a"));
    rewrite(eg, pat, outpat);
}

fn add_assoc(eg: &mut EGraph<ArithENode>) {
    let pat = add2(pvar("?a"), add2(pvar("?b"), pvar("?c")));
    let outpat = add2(add2(pvar("?a"), pvar("?b")), pvar("?c"));
    rewrite_bi(eg, pat, outpat);
}

fn mul_assoc(eg: &mut EGraph<ArithENode>) {
    let pat = mul2(pvar("?a"), mul2(pvar("?b"), pvar("?c")));
    let outpat = mul2(mul2(pvar("?a"), pvar("?b")), pvar("?c"));
    rewrite_bi(eg, pat, outpat);
}

fn distr(eg: &mut EGraph<ArithENode>) {
    let pat = mul2(pvar("?a"), add2(pvar("?b"), pvar("?c"))); // a * (b+c)
    let outpat = add2(mul2(pvar("?a"), pvar("?b")), mul2(pvar("?a"), pvar("?c"))); // (a*b) + (a*c)
    rewrite_bi(eg, pat, outpat);
}
