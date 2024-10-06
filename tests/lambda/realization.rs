use crate::lambda::*;

const NO_ITERS: usize = 400;
const NO_ENODES: usize = 10000;

pub trait Realization {
    fn step(eg: &mut EGraph<Lambda>);
}

// stops when the desired output has been reached.
pub fn simplify_to_nf<R: Realization>(s: &str) -> String {
    let orig_re = RecExpr::parse(s).unwrap();
    let mut re = orig_re.clone();
    let mut eg = EGraph::new();
    let i = eg.add_syn_expr(re.clone());
    for _ in 0..NO_ITERS {
        R::step(&mut eg);

        re = extract_ast(&eg, i.clone());
        if lam_step(&re).is_none() {
            eg.explain_equivalence(orig_re, re.clone());
            return re.to_string();
        };

        if eg.total_number_of_nodes() > NO_ENODES {
            break;
        }
    }
    panic!("failed to reach NF! Or the beta-NF is just AstSize-suboptimal!");
}


pub fn simplify<R: Realization>(s: &str) -> String {
    let re = RecExpr::parse(s).unwrap();
    let mut eg = EGraph::new();
    let i = eg.add_syn_expr(re.clone());
    for _ in 0..NO_ITERS {
        R::step(&mut eg);
        if eg.total_number_of_nodes() > NO_ENODES {
            break;
        }
    }
    let out = extract_ast(&eg, i.clone());

    eg.explain_equivalence(re.clone(), out.clone());

    let out = out.to_string();

    out
}

// TODO the smallest term isn't necessarily the beta-NF.
pub fn check_simplify<R: Realization>(p: &str) {
    let out1 = simplify::<R>(p);
    let out2 = run(p);
    assert_alpha_eq(&*out1, &*out2);
}

pub fn check_simplify_to_nf<R: Realization>(p: &str) {
    let out1 = simplify_to_nf::<R>(p);
    let out2 = run(p);
    assert_alpha_eq(&*out1, &*out2);
}

// checks whether simplify has valid output, even though it might not be able to finish the whole computation.
pub fn check_simplify_incomplete<R: Realization>(p: &str) {
    let out1 = run(&simplify::<R>(p));
    let out2 = run(p);
    assert_alpha_eq(&*out1, &*out2);
}

pub fn check_eq<R: Realization>(s1: &str, s2: &str) {
    let s1 = RecExpr::parse(s1).unwrap();
    let s2 = RecExpr::parse(s2).unwrap();
    let mut eg = EGraph::new();
    let i1 = eg.add_syn_expr(s1.clone());
    let i2 = eg.add_syn_expr(s2.clone());
    for _ in 0..NO_ITERS {
        if eg.eq(&i1, &i2) {
            eg.explain_equivalence(s1.clone(), s2.clone());
            return;
        }

        R::step(&mut eg);

        if eg.total_number_of_nodes() > NO_ENODES {
            break;
        }
    }
    panic!("equality could not be found!");
}

// Non-Realization functions:

fn extract_ast(eg: &EGraph<Lambda>, i: AppliedId) -> RecExpr<Lambda> {
    extract::<_, AstSizeNoLet>(i, eg)
}

pub fn norm(s: &str) -> String {
    let s = RecExpr::parse(s).unwrap();
    let s = lam_normalize(&s);
    s.to_string()
}

pub fn run(s: &str) -> String {
    let s = RecExpr::parse(s).unwrap();
    let s = lam_run(&s);
    let s = lam_normalize(&s);
    s.to_string()
}

pub fn assert_alpha_eq(s1: &str, s2: &str) {
    assert_eq!(norm(s1), norm(s2));
}

pub fn assert_run_eq(s1: &str, s2: &str) {
    assert_eq!(run(s1), run(s2));
}
