use crate::*;
use crate::i_rise::build::*;

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
    let a = "(app (lam s0 (app (lam s1 (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (app (app (var s0) (var s1)) (var s1)))))))) (lam s2 (app (app sym_add (var s2)) num_1)))) (lam s3 (lam s4 (lam s5 (app (var s3) (app (var s4) (var s5)))))))";
    let b = "(lam s0 (app (app sym_add (app (app sym_add (app (app sym_add (app (app sym_add (app (app sym_add (app (app sym_add (app (app sym_add (var s0)) num_1)) num_1)) num_1)) num_1)) num_1)) num_1)) num_1))";
    assert_reaches(a, b, 40);
}

#[test]
fn test_fission() {
    let a = "(app sym_map (lam s42 (app sym_f5 (app sym_f4 (app sym_f3 (app sym_f2 (app sym_f1 (var s42))))))))";
    let b = "(lam s1 (app (app sym_map (lam s42 (app sym_f5 (app sym_f4 (app sym_f3 (var s42)))))) (app (app sym_map (lam s42 (app sym_f2 (app sym_f1 (var s42))))) (var s1))))";
    assert_reaches(a, b, 40);
}

#[test]
pub fn test_binomial() {
    let a = "(app (app sym_map (app sym_map (lam s0 (app (app (app sym_reduce sym_add) num_0) (app (app sym_map (lam s-1 (app (app sym_mul (app sym_fst (var s-1))) (app sym_snd (var s-1))))) (app (app sym_zip (app sym_join sym_weights2d)) (app sym_join (var s0)))))))) (app (app sym_map sym_transpose) (app (app (app sym_slide num_3) num_1) (app (app sym_map (app (app sym_slide num_3) num_1)) sym_input))))";
    let b = "(app (app sym_map (lam s0 (app (app sym_map (lam s1 (app (app (app sym_reduce sym_add) num_0) (app (app sym_map (lam s-2 (app (app sym_mul (app sym_fst (var s-2))) (app sym_snd (var s-2))))) (app (app sym_zip sym_weightsH) (var s1)))))) (app (app (app sym_slide num_3) num_1) (app (app sym_map (lam s2 (app (app (app sym_reduce sym_add) num_0) (app (app sym_map (lam s-3 (app (app sym_mul (app sym_fst (var s-3))) (app sym_snd (var s-3))))) (app (app sym_zip sym_weightsV) (var s2)))))) (app sym_transpose (var s0))))))) (app (app (app sym_slide num_3) num_1) sym_input))";
    assert_reaches(a, b, 40);
}
