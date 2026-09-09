use crate::*;

pub fn sdql_rules() -> Vec<Rewrite<Sdql>> {
    let pat = "(sum ?R $x $y (sing ?e1 ?e2))";
    let outpat = "(sing ?e1 (sum ?R $x $y ?e2))";

    vec![Rewrite::new_if("rule1", pat, outpat, |subst, _| {
        !subst["e1"].slots().contains(&Slot::named("x"))
            && !subst["e1"].slots().contains(&Slot::named("y"))
    })]

    //rw!("sum-fact-3";  "(sum ?R (sing ?e1 ?e2))"        =>
    //        { with_shifted_double_down(var("?e1"), var("?e1d"), 2, "(sing ?e1d (sum ?R ?e2))".parse::<Pattern<SDQL>>().unwrap()) }
    //        if and(neg(contains_ident(var("?e1"), Index(0))), neg(contains_ident(var("?e1"), Index(1))))),
}

#[test]
fn t1() {
    let input = &format!("(lambda $R (lambda $a (sum (var $R) $i $j (sing (var $a) (var $j)))))");

    let re: RecExpr<Sdql> = RecExpr::parse(input).unwrap();
    let rewrites = sdql_rules();

    let mut eg = EGraph::default();

    let id = eg.add_syn_expr(re.clone());
    let mut runner = Runner::<Sdql>::default().with_egraph(eg);
    let report = runner.run(&sdql_rules()[..]);
    let term = extract::<_, _, AstSize>(&id, &runner.egraph);
    eprintln!("{}", re.to_string());
    eprintln!("{}", term.to_string());
}

fn sum_merge_lhs() -> &'static str {
    "(sum (var $1) $2 $3 (sum (var $4) $5 $6 (ifthen (eq (var $3) (var $6)) (get (var $2) (var $5)))))"
}

fn sum_merge_rhs() -> &'static str {
    "(merge (var $1) (var $4) $2 $5 $3 (let (var $3) $6 (get (var $2) (var $5))))"
}

fn assert_sum_merge_does_not_identify_its_free_slots(eg: &EGraph<Sdql>, root: &AppliedId) {
    let root = eg.find_applied_id(root);
    let free = [Slot::named("1"), Slot::named("4")].into_iter().collect();
    assert_eq!(root.slots(), free);

    let swap = SlotMap::from_pairs(&[
        (Slot::named("1"), Slot::named("4")),
        (Slot::named("4"), Slot::named("1")),
    ]);
    assert!(!eg.eq(&root, &root.apply_slotmap(&swap)));
}

#[test]
fn weak_shape_restores_a_free_slot_shadowed_by_a_binder() {
    let input = "(let (var $x) $x (var $x))";
    let mut eg: EGraph<Sdql> = EGraph::default();
    let id = eg.add_syn_expr(RecExpr::parse(input).unwrap());

    assert_eq!(id.slots(), singleton_set(Slot::named("x")));
    let found = lookup_rec_expr(&RecExpr::parse(input).unwrap(), &eg).unwrap();
    assert!(eg.eq(&id, &found));
    eg.check();
}

#[test]
fn sum_merge_flattened() {
    let mut eg: EGraph<Sdql> = EGraph::default();
    let root = eg.add_syn_expr(RecExpr::parse(sum_merge_lhs()).unwrap());
    let pat: MultiPattern<Sdql> = MultiPattern::parse(
        "?p == (sum ?R $k1 $v1 ?t1), \
         ?t1 == (sum ?S $k2 $v2 ?t2), \
         ?t2 == (ifthen ?t3 ?body), \
         ?sl1 == (var $v1), \
         ?sl2 == (var $v2), \
         ?t3 == (eq ?sl1 ?sl2)",
    )
    .unwrap();
    let from = Pattern::parse("?p").unwrap();
    let to = Pattern::parse("(merge ?R ?S $k1 $k2 $v1 (let (var $v1) $v2 ?body))").unwrap();
    let matches = multi_ematch(&pat, &eg);
    assert!(!matches.is_empty());
    for subst in matches {
        eg.union_instantiations(&from, &to, &subst, Some("sum-merge".to_string()));
    }

    let lhs = lookup_rec_expr(&RecExpr::parse(sum_merge_lhs()).unwrap(), &eg).unwrap();
    let rhs = lookup_rec_expr(&RecExpr::parse(sum_merge_rhs()).unwrap(), &eg).unwrap();
    assert!(eg.eq(&lhs, &rhs));
    assert_sum_merge_does_not_identify_its_free_slots(&eg, &root);
    eg.check();
}
