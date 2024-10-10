use crate::*;

fn assert_reaches(start: &str, goal: &str, steps: usize) {
    let start = RecExpr::parse(start).unwrap();
    let goal = RecExpr::parse(goal).unwrap();

    let rules = rise_rules(SubstMethod::SmallStep);

    let mut eg = EGraph::new();
    let i1 = eg.add_expr(start.clone());
    for _ in 0..steps {
        apply_rewrites(&mut eg, &rules);
        dbg!(eg.total_number_of_nodes());
        if let Some(i2) = lookup_rec_expr(&goal, &eg) {
            if eg.eq(&i1, &i2) {
                dbg!(eg.total_number_of_nodes());
                #[cfg(feature = "explanations")]
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
fn reduction() {
    let a = "(app (lam s0 (app (lam s1 (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (var s1)))))))) (lam s2 (app (app add (var s2)) 1)))) (lam s3 (lam s4 (lam s5 (app (var s3) (app (var s4) (var s5)))))))";
    let b = "(lam s0 (app (app add (app (app add (app (app add (app (app add (app (app add (app (app add (app (app add (var s0)) 1)) 1)) 1)) 1)) 1)) 1)) 1))";
    assert_reaches(a, b, 40);
}

#[test]
fn fission() {
    let a = "(app map (lam s42 (app f5 (app f4 (app f3 (app f2 (app f1 (var s42))))))))";
    let b = "(lam s1 (app (app map (lam s42 (app f5 (app f4 (app f3 (var s42)))))) (app (app map (lam s42 (app f2 (app f1 (var s42))))) (var s1))))";
    assert_reaches(a, b, 40);
}

#[test]
#[ignore] // takes too long
pub fn binomial() {
    let a = "(app (app map (app map (lam s0 (app (app (app reduce add) 0) (app (app map (lam s-1 (app (app mul (app fst (var s-1))) (app snd (var s-1))))) (app (app zip (app join weights2d)) (app join (var s0)))))))) (app (app map transpose) (app (app (app slide 3) 1) (app (app map (app (app slide 3) 1)) input))))";
    let b = "(app (app map (lam s0 (app (app map (lam s1 (app (app (app reduce add) 0) (app (app map (lam s-2 (app (app mul (app fst (var s-2))) (app snd (var s-2))))) (app (app zip weightsH) (var s1)))))) (app (app (app slide 3) 1) (app (app map (lam s2 (app (app (app reduce add) 0) (app (app map (lam s-3 (app (app mul (app fst (var s-3))) (app snd (var s-3))))) (app (app zip weightsV) (var s2)))))) (app transpose (var s0))))))) (app (app (app slide 3) 1) input))";
    assert_reaches(a, b, 40);
}


#[test]
fn small15() {
    let eg: &mut EGraph<Rise> = &mut EGraph::new();
    equate("(app a (var s0))", "(app b (var s0))", eg); // a(x) = b(x)

    // Removing this equation, makes it work.
    equate("(app s (app a (var s0)))", "c", eg); // s(a(x)) = c
    eg.dump();
    explain("(app s (app a (var s0)))", "(app s (app b (var s0)))", eg); // s(a(x)) = s(b(x))
}

#[test]
fn small14() {
    let eg: &mut EGraph<Rise> = &mut EGraph::new();
    equate("(app (var s0) (var s1))", "(app (var s1) (var s2))", eg);
    eg.dump();
    eg.check();
    explain("(app (app (var s0) (var s1)) x)", "(app (app (var s2) (var s3)) x)", eg);
}

#[test]
fn small13() {
    let eg: &mut EGraph<Rise> = &mut EGraph::new();
    equate("(app (var s0) (var s1))", "(app (var s1) (var s0))", eg);
    eg.dump();
    explain("(app (app (var s0) (var s1)) x)", "(app (app (var s1) (var s0)) x)", eg);
}

#[test]
fn small12() {
    let eg: &mut EGraph<Rise> = &mut EGraph::new();
    equate("(var s0)", "y", eg);
    eg.dump();
    explain("(lam s1 (var s1))", "(lam s0 (var s0))", eg);
    explain("(lam s1 (var s1))", "(lam s0 (var s2))", eg);
}

#[test]
fn small11() {
    let eg: &mut EGraph<Rise> = &mut EGraph::new();

    equate("(app (var s0) (var s1))", "(app (var s0) x)", eg);
    equate("(app (var s0) (var s1))", "(app (var s1) (var s0))", eg);
    eg.dump();
    explain("(app (var s0) (var s1))", "(app (var s3) (var s4))", eg);
}

#[test]
fn small10() {
    let eg: &mut EGraph<Rise> = &mut EGraph::new();
    equate("(app (var s0) (var s1))", "x", eg);
    eg.dump();
    explain("(app (var s0) (var s1))", "(app (var s1) (var s0))", eg);
}

#[test]
fn small9() {
    let eg: &mut EGraph<Rise> = &mut EGraph::new();
    equate("(app (var s0) x)", "y", eg);
    eg.dump();
    explain("(app (var s0) x)", "(app (var s1) x)", eg);
}

#[test]
fn small8() {
    let eg: &mut EGraph<Rise> = &mut EGraph::new();
    equate("(app (app (var s0) (var s1)) x)", "(app (app (var s1) (var s0)) x)", eg);
    equate("(app (app (var s0) (var s1)) y)", "(app (app (var s1) (var s0)) y)", eg);
    equate("(app (app (var s0) (var s1)) x)", "(app (app (var s0) (var s1)) y)", eg);
    eg.dump();
    explain("(app (app (var s0) (var s1)) x)", "(app (app (var s1) (var s0)) y)", eg);
}

#[test]
fn small7() {
    let eg: &mut EGraph<Rise> = &mut EGraph::new();
    equate("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s1) (var s0)) (var s2))", eg);
    equate("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s0) (var s2)) (var s1))", eg);
    eg.dump();
    explain("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s0) (var s1)) (var s2))", eg);
    explain("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s0) (var s2)) (var s1))", eg);
    explain("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s1) (var s0)) (var s2))", eg);
    explain("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s1) (var s2)) (var s0))", eg);
    explain("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s2) (var s0)) (var s1))", eg);
    explain("(app (app (var s0) (var s1)) (var s2))", "(app (app (var s2) (var s1)) (var s0))", eg);
}

#[test]
fn small6() {
    let eg: &mut EGraph<Rise> = &mut EGraph::new();
    equate("(app (var s0) (var s1))", "(app (var s1) (var s0))", eg);
    eg.dump();
    explain("(app (var s0) (var s1))", "(app (var s1) (var s0))", eg);
}

#[test]
fn small5() {
    let eg: &mut EGraph<Rise> = &mut EGraph::new();
    equate("(var s0)", "(app (var s0) x)", eg);
    equate("x", "y", eg);
    eg.dump();
    explain("(var s2)", "(app (var s2) y)", eg);
}


#[test]
fn small3() {
    let eg: &mut EGraph<Rise> = &mut EGraph::new();
    let x1 = id("x1", eg);
    let x2 = id("x2", eg);
    let x1x3 = term("(app x1 x3)", eg);
    let x2x3 = term("(app x2 x3)", eg);
    eg.union(&x1, &x2);
    eg.dump();
    dbg!(&x1x3);
    dbg!(&x2x3);
    #[cfg(feature = "explanations")]
    eg.explain_equivalence(x1x3, x2x3).show_expr(&eg);
}

#[test]
fn small2() {
    let p = |s| RecExpr::parse(s).unwrap();
    let x1 = p("x1");
    let x2 = p("x2");
    let x3 = p("x3");
    let x4 = p("x4");
    let mut eg: EGraph<Rise> = EGraph::new();
    let y1 = eg.add_expr(x1.clone());
    let y2 = eg.add_expr(x2.clone());
    let y3 = eg.add_expr(x3.clone());
    let y4 = eg.add_expr(x4.clone());
    eg.union(&y1, &y2);
    eg.union(&y3, &y4);
    eg.union(&y2, &y3);
    #[cfg(feature = "explanations")]
    eg.explain_equivalence(x1, x4).show_expr(&eg);
}
