use crate::*;

pub fn array_rules(rules: &[&'static str]) -> Vec<Rewrite<ArrayENode>> {
    let mut rewrites = Vec::new();

    for r in rules {
        let rewrite = match *r {
            "beta" => {
                rewrites.push(my_let_unused());
                rewrites.push(let_var_same());
                rewrites.push(let_app());
                rewrites.push(let_lam_diff());
                beta()
            },
            "eta" => eta(),

            "map-fission" => map_fission(),
            "map-fusion" => map_fusion(),

            "transpose-maps" => rew("transpose-maps", "(m ?n1 (m ?n2 ?f))", "(o T (o (m ?n2 (m ?n1 ?f)) T))"),
            "split-map" => rew("split-map", "(m (* ?n1 ?n2) ?f)", "(o j (o (m ?n1 (m ?n2 ?f)) (s ?n2)))"),

            "o-map-fission" => rew("o-map-fission", "(m ?n (o ?f ?g))", "(o (m ?n ?f) (m ?n ?g))"),
            "o-map-fusion" => rew("o-map-fusion", "(o (m ?n ?f) (m ?n ?g))", "(m ?n (o ?f ?g))"),

            "assoc1" => rew("assoc1", "(o ?a (o ?b ?c))", "(o (o ?a ?b) ?c)"),
            "assoc2" => rew("assoc2", "(o (o ?a ?b) ?c)", "(o ?a (o ?b ?c))"),
            x => panic!("unknown rule: {x}"),
        };
        rewrites.push(rewrite);
    }

    rewrites
}


fn rew(name: &str, s1: &str, s2: &str) -> Rewrite<ArrayENode> {
    let pat = &array_parse_pattern(s1).to_string();
    let outpat = &array_parse_pattern(s2).to_string();

    Rewrite::new(name, pat, outpat)
}

//////////////////////

fn beta() -> Rewrite<ArrayENode> {
    let pat = "(app (lam s1 ?body) ?e)";
    let outpat = "(let s1 ?e ?body)";

    Rewrite::new("beta", pat, outpat)
}

fn eta() -> Rewrite<ArrayENode> {
    let pat = "(lam s1 (app ?f (var s1)))";
    let outpat = "?f";

    Rewrite::new_if("eta", pat, outpat, |subst| {
        !subst["f"].slots().contains(&Slot::new(1))
    })
}

fn my_let_unused() -> Rewrite<ArrayENode> {
    let pat = "(let s1 ?t ?b)";
    let outpat = "?b";
    Rewrite::new_if("my-let-unused", pat, outpat, |subst| {
        !subst["b"].slots().contains(&Slot::new(1))
    })
}

fn let_var_same() -> Rewrite<ArrayENode> {
    let pat = "(let s1 ?e (var s1))";
    let outpat = "?e";
    Rewrite::new("let-var-same", pat, outpat)
}

fn let_var_diff() -> Rewrite<ArrayENode> {
    let pat = "(let s1 ?e (var s2))";
    let outpat = "(var s2)";
    Rewrite::new("let-var-diff", pat, outpat)
}

fn let_app() -> Rewrite<ArrayENode> {
    let pat = "(let s1 ?e (app ?a ?b))";
    let outpat = "(app (let s1 ?e ?a) (let s1 ?e ?b))";
    Rewrite::new_if("let-app", pat, outpat, |subst| {
        subst["a"].slots().contains(&Slot::new(1)) || subst["b"].slots().contains(&Slot::new(1))
    })
}

fn let_lam_diff() -> Rewrite<ArrayENode> {
    let pat = "(let s1 ?e (lam s2 ?body))";
    let outpat = "(lam s2 (let s1 ?e ?body))";
    Rewrite::new_if("let-lam-diff", pat, outpat, |subst| {
        subst["body"].slots().contains(&Slot::new(1))
    })
}

/////////////////////

fn map_fusion() -> Rewrite<ArrayENode> {
    let mfu = "s0";
    let pat = "(app (app (app m ?nn) ?f) (app (app (app m ?nn) ?g) ?arg))";
    let outpat = &format!("(app (app (app m ?nn) (lam {mfu} (app ?f (app ?g (var {mfu}))))) ?arg)");
    Rewrite::new("map-fusion", pat, outpat)
}

fn map_fission() -> Rewrite<ArrayENode> {
    let x = 0;
    let mfi = 1;

    let pat = &format!(
        "(app (app m ?nn) (lam s{x} (app ?f ?gx)))"
    );

    let outpat = &format!(
        "(lam s{mfi} (app (app (app m ?nn) ?f) (app (app (app m ?nn) (lam s{x} ?gx)) (var s{mfi}))))"
    );

    Rewrite::new_if("map-fission", pat, outpat, move |subst| {
        !subst["f"].slots().contains(&Slot::new(x))
    })
}
