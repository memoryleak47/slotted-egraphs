use crate::lambda::*;

const NO_ITERS: usize = 400;
const NO_ENODES: usize = 10000;

pub trait Realization {
    fn get_rewrites() -> Vec<Rewrite<Lambda>>;
}

// stops when the desired output has been reached.
pub fn simplify_to_nf<R: Realization>(s: &str) -> String {
    let orig_re = RecExpr::parse(s).unwrap();
    let mut eg = EGraph::new();
    let i = eg.add_syn_expr(orig_re.clone());

    let hook = Box::new(move |runner: &mut Runner<Lambda>| {
        let re = extract_ast(&runner.egraph, &i.clone());
        if lam_step(&re).is_none() {
            #[cfg(feature = "explanations")]
            runner
                .egraph
                .explain_equivalence(orig_re.clone(), re.clone());
            return Err(re.to_string());
        };
        Ok(())
    });

    let mut runner = Runner::new()
        .with_egraph(eg)
        .with_node_limit(NO_ENODES)
        .with_iter_limit(NO_ITERS)
        .with_hook(hook);

    let report = runner.run(&R::get_rewrites()[..]);
    if let StopReason::Other(s) = report.stop_reason {
        return s;
    }

    panic!("failed to reach NF! Or the beta-NF is just AstSize-suboptimal!");
}

pub fn simplify<R: Realization>(s: &str) -> String {
    let re = RecExpr::parse(s).unwrap();
    let mut eg = EGraph::new();
    let i = eg.add_syn_expr(re.clone());

    let mut runner: Runner<Lambda> = Runner::new().with_egraph(eg).with_node_limit(NO_ENODES);
    runner.run(&R::get_rewrites()[..]);
    let out = extract_ast(&runner.egraph, &i);

    #[cfg(feature = "explanations")]
    runner.egraph.explain_equivalence(re.clone(), out.clone());

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

    let hook = Box::new(move |runner: &mut Runner<Lambda, (), (), ()>| {
        if runner.egraph.eq(&i1, &i2) {
            #[cfg(feature = "explanations")]
            runner.egraph.explain_equivalence(s1.clone(), s2.clone());
            return Err(());
        }
        Ok(())
    });
    let mut runner = Runner::new()
        .with_egraph(eg)
        .with_node_limit(NO_ENODES)
        .with_iter_limit(NO_ITERS)
        .with_hook(hook);
    let report = runner.run(&R::get_rewrites()[..]);
    if let StopReason::Other(()) = report.stop_reason {
        return;
    }
    panic!("equality could not be found!");
}

// Non-Realization functions:

fn extract_ast(eg: &EGraph<Lambda>, i: &AppliedId) -> RecExpr<Lambda> {
    extract::<_, _, AstSizeNoLet>(i, eg)
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
