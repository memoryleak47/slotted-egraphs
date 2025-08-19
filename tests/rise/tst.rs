use crate::*;

#[test]
#[cfg_attr(any(feature = "checks", feature = "explanations"), ignore = "too slow")]
fn reduction() {
    let a = "(app (lam $0 (app (lam $1 (app (app (var $0) (var $1)) (app (app (var $0) (var $1)) (app (app (var $0) (var $1)) (app (app (var $0) (var $1)) (app (app (var $0) (var $1)) (app (app (var $0) (var $1)) (var $1)))))))) (lam $2 (app (app add (var $2)) 1)))) (lam $3 (lam $4 (lam $5 (app (var $3) (app (var $4) (var $5)))))))";
    let b = "(lam $0 (app (app add (app (app add (app (app add (app (app add (app (app add (app (app add (app (app add (var $0)) 1)) 1)) 1)) 1)) 1)) 1)) 1))";
    assert_reaches(a, b, &rise_rules(RiseSubstMethod::SmallStep)[..], 40);
}

#[test]
#[cfg_attr(any(feature = "checks", feature = "explanations"), ignore = "too slow")]
fn fission() {
    let a = "(app map (lam $42 (app f5 (app f4 (app f3 (app f2 (app f1 (var $42))))))))";
    let b = "(lam $1 (app (app map (lam $42 (app f5 (app f4 (app f3 (var $42)))))) (app (app map (lam $42 (app f2 (app f1 (var $42))))) (var $1))))";
    assert_reaches(a, b, &rise_rules(RiseSubstMethod::SmallStep)[..], 40);
}

#[test]
#[ignore = "too slow"]
pub fn binomial() {
    let a = "(app (app map (app map (lam $0 (app (app (app reduce add) 0) (app (app map (lam $m1 (app (app mul (app fst (var $m1))) (app snd (var $m1))))) (app (app zip (app join weights2d)) (app join (var $0)))))))) (app (app map transpose) (app (app (app slide 3) 1) (app (app map (app (app slide 3) 1)) input))))";
    let b = "(app (app map (lam $0 (app (app map (lam $1 (app (app (app reduce add) 0) (app (app map (lam $m2 (app (app mul (app fst (var $m2))) (app snd (var $m2))))) (app (app zip weightsH) (var $1)))))) (app (app (app slide 3) 1) (app (app map (lam $2 (app (app (app reduce add) 0) (app (app map (lam $m3 (app (app mul (app fst (var $m3))) (app snd (var $m3))))) (app (app zip weightsV) (var $2)))))) (app transpose (var $0))))))) (app (app (app slide 3) 1) input))";
    assert_reaches(a, b, &rise_rules(RiseSubstMethod::SmallStep)[..], 40);
}

#[test]
fn small15() {
    let eg: &mut EGraph<Rise> = &mut EGraph::default();
    equate("(app a (var $0))", "(app b (var $0))", eg); // a(x) = b(x)

    // Removing this equation, makes it work.
    equate("(app s (app a (var $0)))", "c", eg); // s(a(x)) = c
    eg.dump();
    explain("(app s (app a (var $0)))", "(app s (app b (var $0)))", eg); // s(a(x)) = s(b(x))
}

#[test]
fn small14() {
    let eg: &mut EGraph<Rise> = &mut EGraph::default();
    equate("(app (var $0) (var $1))", "(app (var $1) (var $2))", eg);
    eg.dump();
    eg.check();
    explain(
        "(app (app (var $0) (var $1)) x)",
        "(app (app (var $2) (var $3)) x)",
        eg,
    );
}

#[test]
fn small13() {
    let eg: &mut EGraph<Rise> = &mut EGraph::default();
    equate("(app (var $0) (var $1))", "(app (var $1) (var $0))", eg);
    eg.dump();
    explain(
        "(app (app (var $0) (var $1)) x)",
        "(app (app (var $1) (var $0)) x)",
        eg,
    );
}

#[test]
fn small12() {
    let eg: &mut EGraph<Rise> = &mut EGraph::default();
    equate("(var $0)", "y", eg);
    eg.dump();
    explain("(lam $1 (var $1))", "(lam $0 (var $0))", eg);
    explain("(lam $1 (var $1))", "(lam $0 (var $2))", eg);
}

#[test]
fn small11() {
    let eg: &mut EGraph<Rise> = &mut EGraph::default();

    equate("(app (var $0) (var $1))", "(app (var $0) x)", eg);
    equate("(app (var $0) (var $1))", "(app (var $1) (var $0))", eg);
    eg.dump();
    explain("(app (var $0) (var $1))", "(app (var $3) (var $4))", eg);
}

#[test]
fn small10() {
    let eg: &mut EGraph<Rise> = &mut EGraph::default();
    equate("(app (var $0) (var $1))", "x", eg);
    eg.dump();
    explain("(app (var $0) (var $1))", "(app (var $1) (var $0))", eg);
}

#[test]
fn small9() {
    let eg: &mut EGraph<Rise> = &mut EGraph::default();
    equate("(app (var $0) x)", "y", eg);
    eg.dump();
    explain("(app (var $0) x)", "(app (var $1) x)", eg);
}

#[test]
fn small8() {
    let eg: &mut EGraph<Rise> = &mut EGraph::default();
    equate(
        "(app (app (var $0) (var $1)) x)",
        "(app (app (var $1) (var $0)) x)",
        eg,
    );
    equate(
        "(app (app (var $0) (var $1)) y)",
        "(app (app (var $1) (var $0)) y)",
        eg,
    );
    equate(
        "(app (app (var $0) (var $1)) x)",
        "(app (app (var $0) (var $1)) y)",
        eg,
    );
    eg.dump();
    explain(
        "(app (app (var $0) (var $1)) x)",
        "(app (app (var $1) (var $0)) y)",
        eg,
    );
}

#[test]
fn small7() {
    let eg: &mut EGraph<Rise> = &mut EGraph::default();
    equate(
        "(app (app (var $0) (var $1)) (var $2))",
        "(app (app (var $1) (var $0)) (var $2))",
        eg,
    );
    equate(
        "(app (app (var $0) (var $1)) (var $2))",
        "(app (app (var $0) (var $2)) (var $1))",
        eg,
    );
    eg.dump();
    explain(
        "(app (app (var $0) (var $1)) (var $2))",
        "(app (app (var $0) (var $1)) (var $2))",
        eg,
    );
    explain(
        "(app (app (var $0) (var $1)) (var $2))",
        "(app (app (var $0) (var $2)) (var $1))",
        eg,
    );
    explain(
        "(app (app (var $0) (var $1)) (var $2))",
        "(app (app (var $1) (var $0)) (var $2))",
        eg,
    );
    explain(
        "(app (app (var $0) (var $1)) (var $2))",
        "(app (app (var $1) (var $2)) (var $0))",
        eg,
    );
    explain(
        "(app (app (var $0) (var $1)) (var $2))",
        "(app (app (var $2) (var $0)) (var $1))",
        eg,
    );
    explain(
        "(app (app (var $0) (var $1)) (var $2))",
        "(app (app (var $2) (var $1)) (var $0))",
        eg,
    );
}

#[test]
fn small6() {
    let eg: &mut EGraph<Rise> = &mut EGraph::default();
    equate("(app (var $0) (var $1))", "(app (var $1) (var $0))", eg);
    eg.dump();
    explain("(app (var $0) (var $1))", "(app (var $1) (var $0))", eg);
}

#[test]
fn small5() {
    let eg: &mut EGraph<Rise> = &mut EGraph::default();
    equate("(var $0)", "(app (var $0) x)", eg);
    equate("x", "y", eg);
    eg.dump();
    explain("(var $2)", "(app (var $2) y)", eg);
}

#[test]
fn small3() {
    let eg: &mut EGraph<Rise> = &mut EGraph::default();
    let x1 = id("x1", eg);
    let x2 = id("x2", eg);
    let x1x3: RecExpr<Rise> = term("(app x1 x3)");
    let x2x3: RecExpr<Rise> = term("(app x2 x3)");
    eg.union(&x1, &x2);
    eg.dump();
    dbg!(&x1x3);
    dbg!(&x2x3);
    #[cfg(feature = "explanations")]
    println!("{}", eg.explain_equivalence(x1x3, x2x3).to_string(&eg));
}

#[test]
fn small2() {
    let p = |s| RecExpr::parse(s).unwrap();
    let x1 = p("x1");
    let x2 = p("x2");
    let x3 = p("x3");
    let x4 = p("x4");
    let mut eg: EGraph<Rise> = EGraph::default();
    let y1 = eg.add_expr(x1.clone());
    let y2 = eg.add_expr(x2.clone());
    let y3 = eg.add_expr(x3.clone());
    let y4 = eg.add_expr(x4.clone());
    eg.union(&y1, &y2);
    eg.union(&y3, &y4);
    eg.union(&y2, &y3);
    #[cfg(feature = "explanations")]
    println!("{}", eg.explain_equivalence(x1, x4).to_string(&eg));
}
