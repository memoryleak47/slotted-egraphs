use crate::*;

pub fn rewrite_let(eg: &mut EGraph<LetENode>) {
    beta(eg);
    my_let_unused(eg);
    let_var_same(eg);
    let_app(eg);
    let_lam_diff(eg);
}

fn beta(eg: &mut EGraph<LetENode>) {
    let pat = Pattern::parse("(app (lam s1 ?b) ?t)").unwrap();
    let outpat = Pattern::parse("(let s1 ?t ?b)").unwrap();
    rewrite(eg, pat, outpat);
}

fn my_let_unused(eg: &mut EGraph<LetENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?t"), pvar_pat("?b"));
    let outpat = pvar_pat("?b");
    rewrite_if(eg, pat, outpat, |subst| {
        !subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn let_var_same(eg: &mut EGraph<LetENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?e"), var_pat(Slot::new(1)));
    let outpat = pvar_pat("?e");
    rewrite(eg, pat, outpat);
}

fn let_app(eg: &mut EGraph<LetENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?e"), app_pat(pvar_pat("?a"), pvar_pat("?b")));
    let outpat = app_pat(
        let_pat(Slot::new(1), pvar_pat("?e"), pvar_pat("?a")),
        let_pat(Slot::new(1), pvar_pat("?e"), pvar_pat("?b"))
    );
    rewrite_if(eg, pat, outpat, |subst| {
        subst["?a"].slots().contains(&Slot::new(1)) || subst["?b"].slots().contains(&Slot::new(1))
    });
}

fn let_lam_diff(eg: &mut EGraph<LetENode>) {
    let pat = let_pat(Slot::new(1), pvar_pat("?e"), lam_pat(Slot::new(2), pvar_pat("?b")));
    let outpat = lam_pat(Slot::new(2),
        let_pat(Slot::new(1), pvar_pat("?e"), pvar_pat("?b")),
    );
    rewrite_if(eg, pat, outpat, |subst| {
        subst["?b"].slots().contains(&Slot::new(1))
    });
}

// aux functions.
fn pvar_pat(s: &str) -> Pattern<LetENode> {
    Pattern {
        node: ENodeOrPVar::PVar(s.to_string()),
        children: vec![],
    }
}

fn app_pat(l: Pattern<LetENode>, r: Pattern<LetENode>) -> Pattern<LetENode> {
    Pattern {
        node: ENodeOrPVar::ENode(LetENode::App(AppliedId::null(), AppliedId::null())),
        children: vec![l, r],
    }
}

fn var_pat(s: Slot) -> Pattern<LetENode> {
    Pattern {
        node: ENodeOrPVar::ENode(LetENode::Var(s)),
        children: vec![],
    }
}

fn lam_pat(s: Slot, b: Pattern<LetENode>) -> Pattern<LetENode> {
    Pattern {
        node: ENodeOrPVar::ENode(LetENode::Lam(s, AppliedId::null())),
        children: vec![b],
    }
}

fn let_pat(s: Slot, t: Pattern<LetENode>, b: Pattern<LetENode>) -> Pattern<LetENode> {
    Pattern {
        node: ENodeOrPVar::ENode(LetENode::Let(s, AppliedId::null(), AppliedId::null())),
        children: vec![t, b],
    }
}
