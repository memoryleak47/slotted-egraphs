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

            "transpose-maps" => rew("(m ?n1 (m ?n2 ?f))", "(o T (o (m ?n2 (m ?n1 ?f)) T))"),
            "split-map" => rew("(m (* ?n1 ?n2) ?f)", "(o j (o (m ?n1 (m ?n2 ?f)) (s ?n2)))"),

            "o-map-fission" => rew("(m ?n (o ?f ?g))", "(o (m ?n ?f) (m ?n ?g))"),
            "o-map-fusion" => rew("(o (m ?n ?f) (m ?n ?g))", "(m ?n (o ?f ?g))"),

            "assoc1" => rew("(o ?a (o ?b ?c))", "(o (o ?a ?b) ?c)"),
            "assoc2" => rew("(o (o ?a ?b) ?c)", "(o ?a (o ?b ?c))"),
            x => panic!("unknown rule: {x}"),
        };
        rewrites.push(rewrite);
    }

    rewrites
}

fn rew(s1: &str, s2: &str) -> Rewrite<ArrayENode> {
    let pat = array_parse_pattern(s1);
    let outpat = array_parse_pattern(s2);

    mk_rewrite(pat, outpat)
}

//////////////////////

fn beta() -> Rewrite<ArrayENode> {
    let pat = Pattern::parse("(app (lam s1 ?body) ?e)").unwrap();
    let outpat = Pattern::parse("(let s1 ?e ?body)").unwrap();

    mk_rewrite(pat, outpat)
}

fn eta() -> Rewrite<ArrayENode> {
    let pat = Pattern::parse("(lam s1 (app ?f (var s1)))").unwrap();
    let outpat = Pattern::parse("?f").unwrap();

    mk_rewrite_if(pat, outpat, |subst| {
        !subst["f"].slots().contains(&Slot::new(1))
    })
}

fn my_let_unused() -> Rewrite<ArrayENode> {
    let pat = Pattern::parse("(let s1 ?t ?b)").unwrap();
    let outpat = Pattern::parse("?b").unwrap();
    mk_rewrite_if(pat, outpat, |subst| {
        !subst["b"].slots().contains(&Slot::new(1))
    })
}

fn let_var_same() -> Rewrite<ArrayENode> {
    let pat = Pattern::parse("(let s1 ?e (var s1))").unwrap();
    let outpat = Pattern::parse("?e").unwrap();
    mk_rewrite(pat, outpat)
}

fn let_var_diff() -> Rewrite<ArrayENode> {
    let pat = Pattern::parse("(let s1 ?e (var s2))").unwrap();
    let outpat = Pattern::parse("(var s2)").unwrap();
    mk_rewrite(pat, outpat)
}

fn let_app() -> Rewrite<ArrayENode> {
    let pat = Pattern::parse("(let s1 ?e (app ?a ?b))").unwrap();
    let outpat = Pattern::parse("(app (let s1 ?e ?a) (let s1 ?e ?b))").unwrap();
    mk_rewrite_if(pat, outpat, |subst| {
        subst["a"].slots().contains(&Slot::new(1)) || subst["b"].slots().contains(&Slot::new(1))
    })
}

fn let_lam_diff() -> Rewrite<ArrayENode> {
    let pat = Pattern::parse("(let s1 ?e (lam s2 ?body))").unwrap();
    let outpat = Pattern::parse("(lam s2 (let s1 ?e ?body))").unwrap();
    mk_rewrite_if(pat, outpat, |subst| {
        subst["body"].slots().contains(&Slot::new(1))
    })
}

/////////////////////

fn map_fusion() -> Rewrite<ArrayENode> {
    let mfu = "s0";
    let pat = Pattern::parse("(app (app (app m ?nn) ?f) (app (app (app m ?nn) ?g) ?arg))").unwrap();
    let outpat = Pattern::parse(&format!("(app (app (app m ?nn) (lam {mfu} (app ?f (app ?g (var {mfu}))))) ?arg)")).unwrap();
    mk_rewrite(pat, outpat)
}

fn map_fission() -> Rewrite<ArrayENode> {
    let x = 0;
    let mfi = 1;

    let pat = Pattern::parse(&format!(
        "(app (app m ?nn) (lam s{x} (app ?f ?gx)))"
    )).unwrap();

    let outpat = Pattern::parse(&format!(
        "(lam s{mfi} (app (app (app m ?nn) ?f) (app (app (app m ?nn) (lam s{x} ?gx)) (var s{mfi}))))"
    )).unwrap();

    mk_rewrite_if(pat, outpat, move |subst| {
        !subst["f"].slots().contains(&Slot::new(x))
    })
}
