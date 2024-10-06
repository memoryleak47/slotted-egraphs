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

// FISSION //

fn fchain(fs: impl Iterator<Item=usize>) -> Pattern<RiseENode> {
    let x = 42;
    let mut it = var(x);
    for i in fs {
        let f_i = symb(&format!("f{}", i));
        it = app(f_i, it);
    }
    lam(x, it)
}

fn fission_re1() -> RecExpr<RiseENode> {
    let out = app(symb("map"), fchain(1..=5));
    pattern_to_re(&out)
}

fn fission_re2() -> RecExpr<RiseENode> {
    let y = 1;

    let left = map1(fchain(3..=5));
    let right = map2(fchain(1..=2), var(y));

    let out = lam(y, app(left, right));

    pattern_to_re(&out)
}

#[test]
fn test_fission() {
    assert_reaches(fission_re1(), fission_re2(), 40);
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
