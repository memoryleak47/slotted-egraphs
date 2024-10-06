use crate::*;
use crate::i_rise::build::*;

fn assert_reaches(start: RecExpr<RiseENode>, goal: RecExpr<RiseENode>, steps: usize) {
    let rules = rise_rules(SubstMethod::SmallStep);
    dbg!(&start);
    dbg!(&goal);

    let mut eg = EGraph::new();
    let i1 = eg.add_expr(start.clone());
    for _ in 0..steps {
        apply_rewrites(&mut eg, &rules);
        dbg!(eg.total_number_of_nodes());
        if let Some(i2) = lookup_rec_expr(&goal, &eg) {
            if eg.eq(&i1, &i2) {
                dbg!(eg.total_number_of_nodes());
                eg.explain_equivalence(start, goal).show_expr(&eg);
                return;
            }
        }
    }

    dbg!(extract::<_, AstSizeNoLet>(i1, &eg));
    dbg!(&goal);
    assert!(false);
}

#[test]
fn test_reduction() {
    let a = "(app (lam s0 (app (lam s1 (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (var s1)))))))) (lam s2 (app (app sym_add (var s2)) num_1)))) (lam s3 (lam s4 (lam s5 (app (var s3) (app (var s4) (var s5)))))))";
    let b = "(lam s0 (app (app sym_add (app (app sym_add (app (app sym_add (app (app sym_add (app (app sym_add (app (app sym_add (app (app sym_add (var s0)) num_1)) num_1)) num_1)) num_1)) num_1)) num_1)) num_1))";
    let a = RecExpr::parse(a).unwrap();
    let b = RecExpr::parse(b).unwrap();
    assert_reaches(a, b, 40);
}

#[test]
fn test_fission() {
    let a = "(app sym_map (lam s42 (app sym_f5 (app sym_f4 (app sym_f3 (app sym_f2 (app sym_f1 (var s42))))))))";
    let b = "(lam s1 (app (app sym_map (lam s42 (app sym_f5 (app sym_f4 (app sym_f3 (var s42)))))) (app (app sym_map (lam s42 (app sym_f2 (app sym_f1 (var s42))))) (var s1))))";
    let a = RecExpr::parse(a).unwrap();
    let b = RecExpr::parse(b).unwrap();
    assert_reaches(a, b, 40);
}

// BINOMIAL //

fn binomial_re1() -> RecExpr<RiseENode> {
    let nbh = 0;
    let dt = dot2(
            join1(symb("weights2d")),
            join1(var(nbh)));
    let out = map2(
        map1(lam(nbh, dt)),
        map2(transpose0(),
            slide3(num(3), num(1), map2(slide2(num(3), num(1)), symb("input")))
        )
    );

    pattern_to_re(&out)
}

fn binomial_re2() -> RecExpr<RiseENode> {
    let nbhL = 0;
    let nbhH = 1;
    let nbhV = 2;

    let out = map2(
        lam(nbhL, map2(
            lam(nbhH, dot2(symb("weightsH"), var(nbhH))),
            slide3(num(3), num(1), map2(lam(nbhV, dot2(symb("weightsV"), var(nbhV))), transpose1(var(nbhL))))
        )),
        slide3(num(3), num(1), symb("input"))
    );

    pattern_to_re(&out)
}

#[test]
#[ignore]
pub fn test_binomial() {
    assert_reaches(binomial_re1(), binomial_re2(), 40);
}
