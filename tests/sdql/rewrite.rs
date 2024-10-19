use crate::*;

pub fn sdql_rules() -> Vec<Rewrite<Sdql>> {
    let pat = "(sum $x $y ?R (sing ?e1 ?e2))";
    let outpat = "(sing ?e1 (sum $x $y ?R ?e2))";

    vec![Rewrite::new_if("rule1", pat, outpat, |subst| {
        !subst["e1"].slots().contains(&Slot::named("x"))
        && !subst["e1"].slots().contains(&Slot::named("y"))
    })]

    //rw!("sum-fact-3";  "(sum ?R (sing ?e1 ?e2))"        => 
    //        { with_shifted_double_down(var("?e1"), var("?e1d"), 2, "(sing ?e1d (sum ?R ?e2))".parse::<Pattern<SDQL>>().unwrap()) }
    //        if and(neg(contains_ident(var("?e1"), Index(0))), neg(contains_ident(var("?e1"), Index(1))))),
}

#[test]
fn t1() {
    let input = &format!("(lambda $R (lambda $a (sum $i $j (var $R) (sing (var $a) (var $j)))))");

    let re: RecExpr<Sdql> = RecExpr::parse(input).unwrap();
    let rewrites = sdql_rules();

    let mut eg = EGraph::new();

    let id = eg.add_syn_expr(re.clone());

    apply_rewrites(&mut eg, &rewrites);
    let term = extract::<_, _, AstSize>(id.clone(), &eg);
    eprintln!("{}", re.to_string());
    eprintln!("{}", term.to_string());
}
