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
    let a = "(app (lam s0 (app (lam s1 (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (var s1)))))))) (lam s2 (app (app add (var s2)) 1)))) (lam s3 (lam s4 (lam s5 (app (var s3) (app (var s4) (var s5)))))))";
    let b = "(lam s0 (app (app add (app (app add (app (app add (app (app add (app (app add (app (app add (app (app add (var s0)) 1)) 1)) 1)) 1)) 1)) 1)) 1))";
    assert_reaches(a, b, 40);
}

#[test]
fn test_fission() {
    let a = "(app map (lam s42 (app f5 (app f4 (app f3 (app f2 (app f1 (var s42))))))))";
    let b = "(lam s1 (app (app map (lam s42 (app f5 (app f4 (app f3 (var s42)))))) (app (app map (lam s42 (app f2 (app f1 (var s42))))) (var s1))))";
    assert_reaches(a, b, 40);
}

#[test]
pub fn test_binomial() {
    let a = "(app (app map (app map (lam s0 (app (app (app reduce add) 0) (app (app map (lam s-1 (app (app mul (app fst (var s-1))) (app snd (var s-1))))) (app (app zip (app join weights2d)) (app join (var s0)))))))) (app (app map transpose) (app (app (app slide 3) 1) (app (app map (app (app slide 3) 1)) input))))";
    let b = "(app (app map (lam s0 (app (app map (lam s1 (app (app (app reduce add) 0) (app (app map (lam s-2 (app (app mul (app fst (var s-2))) (app snd (var s-2))))) (app (app zip weightsH) (var s1)))))) (app (app (app slide 3) 1) (app (app map (lam s2 (app (app (app reduce add) 0) (app (app map (lam s-3 (app (app mul (app fst (var s-3))) (app snd (var s-3))))) (app (app zip weightsV) (var s2)))))) (app transpose (var s0))))))) (app (app (app slide 3) 1) input))";
    assert_reaches(a, b, 40);
}
