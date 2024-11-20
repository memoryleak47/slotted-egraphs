use crate::*;

pub enum RiseSubstMethod {
    Extraction,
    SmallStep,
    SmallStepUnoptimized,
}

pub fn rise_rules(subst_m: RiseSubstMethod) -> Vec<Rewrite<Rise>> {
    let mut rewrites = Vec::new();

    rewrites.push(eta());
    rewrites.push(eta_expansion());

    rewrites.push(map_fusion());
    rewrites.push(map_fission());

    rewrites.push(remove_transpose_pair());
    rewrites.push(slide_before_map());
    rewrites.push(map_slide_before_transpose());
    rewrites.push(slide_before_map_map_f());
    rewrites.push(separate_dot_vh_simplified());
    rewrites.push(separate_dot_hv_simplified());

    match subst_m {
        RiseSubstMethod::Extraction => {
            rewrites.push(beta_extr_direct());
        },
        RiseSubstMethod::SmallStep => {
            rewrites.push(beta());
            rewrites.push(my_let_unused());
            rewrites.push(let_var_same());
            rewrites.push(let_app());
            rewrites.push(let_lam_diff());
        },
        RiseSubstMethod::SmallStepUnoptimized => {
            rewrites.push(beta());
            rewrites.push(let_var_same());
            rewrites.push(let_var_diff());
            rewrites.push(let_app_unopt());
            rewrites.push(let_lam_diff_unopt());
            rewrites.push(let_const());
        },
    }

    rewrites
}

fn beta() -> Rewrite<Rise> {
    let pat = "(app (lam $1 ?body) ?e)";
    let outpat = "(let $1 ?body ?e)";

    Rewrite::new("beta", pat, outpat)
}

fn eta() -> Rewrite<Rise> {
    let pat = "(lam $1 (app ?f (var $1)))";
    let outpat = "?f";

    Rewrite::new_if("eta", pat, outpat, |subst, _| {
        !subst["f"].slots().contains(&Slot::numeric(1))
    })
}

fn eta_expansion() -> Rewrite<Rise> {
    let pat = "?f";
    let outpat = "(lam $1 (app ?f (var $1)))";

    Rewrite::new("eta-expansion", pat, outpat)
}

fn my_let_unused() -> Rewrite<Rise> {
    let pat = "(let $1 ?b ?t)";
    let outpat = "?b";
    Rewrite::new_if("my-let-unused", pat, outpat, |subst, _| {
        !subst["b"].slots().contains(&Slot::numeric(1))
    })
}

fn let_var_same() -> Rewrite<Rise> {
    let pat = "(let $1 (var $1) ?e)";
    let outpat = "?e";
    Rewrite::new("let-var-same", pat, outpat)
}

fn let_var_diff() -> Rewrite<Rise> {
    let pat = "(let $1 (var $2) ?e)";
    let outpat = "(var $2)";
    Rewrite::new("let-var-diff", pat, outpat)
}

fn let_app() -> Rewrite<Rise> {
    let pat = "(let $1 (app ?a ?b) ?e)";
    let outpat = "(app (let $1 ?a ?e) (let $1 ?b ?e))";
    Rewrite::new_if("let-app", pat, outpat, |subst, _| {
        subst["a"].slots().contains(&Slot::numeric(1)) || subst["b"].slots().contains(&Slot::numeric(1))
    })
}

fn let_app_unopt() -> Rewrite<Rise> {
    let pat = "(let $1 (app ?a ?b) ?e)";
    let outpat = "(app (let $1 ?a ?e) (let $1 ?b ?e))";
    Rewrite::new("let-app-unopt", pat, outpat)
}

fn let_lam_diff() -> Rewrite<Rise> {
    let pat = "(let $1 (lam $2 ?body) ?e)";
    let outpat = "(lam $2 (let $1 ?body ?e))";
    Rewrite::new_if("let-lam-diff", pat, outpat, |subst, _| {
        subst["body"].slots().contains(&Slot::numeric(1))
    })
}

fn let_lam_diff_unopt() -> Rewrite<Rise> {
    let pat = "(let $1 (lam $2 ?body) ?e)";
    let outpat = "(lam $2 (let $1 ?body ?e))";
    Rewrite::new("let-lam-diff-unopt", pat, outpat)
}

fn let_const() -> Rewrite<Rise> {
    // is the const-detection at the same time as the baseline? probably not relevant.
    let pat = Pattern::parse("(let $1 ?c ?t)").unwrap();

    let rt: RewriteT<Rise, (), ()> = RewriteT {
        searcher: Box::new(|_| ()),
        applier: Box::new(move |(), eg| {
            for subst in ematch_all(eg, &pat) {
                if eg.enodes_applied(&subst["c"]).iter().any(|n| matches!(n, Rise::Symbol(_) | Rise::Number(_))) {
                    let orig = pattern_subst(eg, &pat, &subst);
                    eg.union_justified(&orig, &subst["c"], Some("let-const".to_string()));
                }
            }
        }),
    };
    rt.into()
}

/////////////////////

fn map_fusion() -> Rewrite<Rise> {
    let mfu = "$0";
    let pat = "(app (app map ?f) (app (app map ?g) ?arg))";
    let outpat = &format!("(app (app map (lam {mfu} (app ?f (app ?g (var {mfu}))))) ?arg)");
    Rewrite::new("map-fusion", pat, outpat)
}

fn map_fission() -> Rewrite<Rise> {
    let x = 0;
    let mfi = 1;

    let pat = &format!(
        "(app map (lam ${x} (app ?f ?gx)))"
    );

    let outpat = &format!(
        "(lam ${mfi} (app (app map ?f) (app (app map (lam ${x} ?gx)) (var ${mfi}))))"
    );

    Rewrite::new_if("map-fission", pat, outpat, move |subst, _| {
        !subst["f"].slots().contains(&Slot::numeric(x))
    })
}

fn remove_transpose_pair() -> Rewrite<Rise> {
    let pat = "(app transpose (app transpose ?x))";
    let outpat = "?x";
    Rewrite::new("remove-transpose-pair", pat, outpat)
}

fn slide_before_map() -> Rewrite<Rise> {
    let pat = "(app (app (app slide ?sz) ?sp) (app (app map ?f) ?y))";
    let outpat = "(app (app map (app map ?f)) (app (app (app slide ?sz) ?sp) ?y))";
    Rewrite::new("slide-before-map", pat, outpat)
}

fn map_slide_before_transpose() -> Rewrite<Rise> {
    let pat = "(app transpose (app (app map (app (app slide ?sz) ?sp)) ?y))";
    let outpat = "(app (app map transpose) (app (app (app slide ?sz) ?sp) (app transpose ?y)))";
    Rewrite::new("map-slide-before-transpose", pat, outpat)
}

fn slide_before_map_map_f() -> Rewrite<Rise> {
    let pat = "(app (app map (app map ?f)) (app (app (app slide ?sz) ?sp) ?y))";
    let outpat = "(app (app (app slide ?sz) ?sp) (app (app map ?f) ?y))";
    Rewrite::new("slide-before-map-map-f", pat, outpat)
}

fn separate_dot_vh_simplified() -> Rewrite<Rise> {
    let x = "$0";
    let sdvh = "$1";

    let pat = &format!(
        "(app (app (app reduce add) 0) (app (app map (lam {x} (app (app mul (app fst (var {x}))) (app snd (var {x})))))
         (app (app zip (app join weights2d)) (app join ?nbh))))
        ");
    let outpat = &format!(
        "(app (app (app reduce add) 0) (app (app map (lam {x} (app (app mul (app fst (var {x}))) (app snd (var {x})))))
         (app (app zip weightsH) (app (app map (lam {sdvh} (app (app (app reduce add) 0) (app (app map (lam {x} (app (app mul (app fst (var {x}))) (app snd (var {x})))))
         (app (app zip weightsV) (var {sdvh})))))) (app transpose ?nbh)))))
        ");
    Rewrite::new("separate-dot-vh-simplified", pat, outpat)
}

fn separate_dot_hv_simplified() -> Rewrite<Rise> {
    let x = "$0";
    let sdhv = "$1";

    let pat = &format!(
        "(app (app (app reduce add) 0) (app (app map (lam {x} (app (app mul (app fst (var {x}))) (app snd (var {x})))))
         (app (app zip (app join weights2d)) (app join ?nbh))))
        ");
    let outpat = &format!(
        "(app (app (app reduce add) 0) (app (app map (lam {x} (app (app mul (app fst (var {x}))) (app snd (var {x})))))
         (app (app zip weightsV) (app (app map (lam {sdhv} (app (app (app reduce add) 0) (app (app map (lam {x} (app (app mul (app fst (var {x}))) (app snd (var {x})))))
         (app (app zip weightsH) (var {sdhv})))))) ?nbh))))
        ");

    Rewrite::new("separate-dot-hv-simplified", pat, outpat)
}

// subst using extraction
fn beta_extr() -> Rewrite<Rise> {
    let pat = Pattern::parse("(app (lam $1 ?b) ?t)").unwrap();
    let s = Slot::numeric(1);

    let a = pat.clone();
    let a2 = pat.clone();

    let rt: RewriteT<Rise, (), Vec<(Subst, RecExpr<Rise>)>> = RewriteT {
        searcher: Box::new(move |eg| {
            let extractor = Extractor::<_, AstSize>::new(eg, AstSize);

            let mut out: Vec<(Subst, RecExpr<Rise>)> = Vec::new();
            for subst in ematch_all(eg, &a) {
                let b = extractor.extract(&subst["b"], eg);
                let t = extractor.extract(&subst["t"], eg);
                let res = re_subst(s, b, &t);
                out.push((subst, res));
            }
            out
        }),
        applier: Box::new(move |substs, eg| {
            for (subst, res) in substs {
                let orig = pattern_subst(eg, &pat, &subst);
                let res = eg.add_expr(res);
                eg.union_justified(&orig, &res, Some("beta-expr".to_string()));
            }
        }),
    };
    rt.into()
}

// why is this faster than beta_extr?
// Probably because it can extract smaller terms after more rewrites?
fn beta_extr_direct() -> Rewrite<Rise> {
    let pat = Pattern::parse("(app (lam $1 ?b) ?t)").unwrap();
    let s = Slot::numeric(1);

    let a = pat.clone();
    let a2 = pat.clone();

    let rt: RewriteT<Rise, (), ()> = RewriteT {
        searcher: Box::new(|_| ()),
        applier: Box::new(move |(), eg| {
            let extractor = Extractor::<_, AstSize>::new(eg, AstSize);

            let mut out: Vec<(Subst, RecExpr<Rise>)> = Vec::new();
            for subst in ematch_all(eg, &a) {
                let b = extractor.extract(&subst["b"], eg);
                let t = extractor.extract(&subst["t"], eg);
                let res = re_subst(s, b, &t);
                out.push((subst, res));
            }
            for (subst, res) in out {
                let orig = pattern_subst(eg, &pat, &subst);
                let res = eg.add_expr(res);
                eg.union_justified(&orig, &res, Some("betaoextr-direct".to_string()));
            }
        }),
    };
    rt.into()
}

fn re_subst(s: Slot, b: RecExpr<Rise>, t: &RecExpr<Rise>) -> RecExpr<Rise> {
    let new_node = match b.node {
        Rise::Var(s2) if s == s2 => return t.clone(),
        Rise::Lam(Bind { slot, .. }) if s == slot => panic!("This shouldn't be possible!"),
        Rise::Let(..) => panic!("This shouldn't be here!"),
        old => old,
    };

    let mut children = Vec::new();
    for child in b.children {
        children.push(re_subst(s, child, t));
    }

    RecExpr {
        node: new_node,
        children,
    }
}
