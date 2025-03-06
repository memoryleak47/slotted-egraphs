pub use slotted_egraphs::*;
pub use std::hash::Hash;
use std::num::Saturating;

pub type HashMap<K, V> = fxhash::FxHashMap<K, V>;
pub type HashSet<T> = fxhash::FxHashSet<T>;

mod arith;
pub use arith::*;

mod lambda;
pub use lambda::*;

mod rise;
pub use rise::*;

mod var;
pub use var::*;

mod fgh;
pub use fgh::*;

mod sdql;
pub use sdql::*;

mod array;
pub use array::*;

pub fn singleton_set<T: Eq + Hash>(t: T) -> HashSet<T> {
    [t].into_iter().collect()
}

pub fn id<L: Language>(s: &str, eg: &mut EGraph<L>) -> AppliedId {
    eg.check();
    let re = RecExpr::parse(s).unwrap();
    let out = eg.add_syn_expr(re.clone());
    eg.check();
    out
}

pub fn term<L: Language>(s: &str) -> RecExpr<L> {
    let re = RecExpr::parse(s).unwrap();
    re
}

pub fn equate<L: Language>(s1: &str, s2: &str, eg: &mut EGraph<L>) {
    eg.check();
    let s1 = id(s1, eg);
    eg.check();
    let s2 = id(s2, eg);
    eg.check();
    eg.union(&s1, &s2);
    eg.check();
}

pub fn explain<L: Language>(s1: &str, s2: &str, eg: &mut EGraph<L>) {
    eg.check();
    #[allow(unused)]
    let s1: RecExpr<L> = term(s1);
    #[allow(unused)]
    let s2: RecExpr<L> = term(s2);
    #[cfg(feature = "explanations")]
    println!("{}", eg.explain_equivalence(s1, s2).to_string(eg));
    eg.check();
}

fn reach_hook<'a, L, N, IterData>(
    start: &'a RecExpr<L>,
    goal: &'a RecExpr<L>,
    steps: usize,
) -> Box<dyn FnMut(&mut Runner<L, N, IterData>) -> Result<(), String>>
where
    L: Language + 'static,
    N: Analysis<L>,
    IterData: IterationData<L, N>,
{
    let start = start.clone();
    let goal = goal.clone();
    return Box::new(move |runner: &mut Runner<L, N, IterData>| {
        if let Some(i2) = lookup_rec_expr(&goal, &runner.egraph) {
            let i1 = lookup_rec_expr(&start, &runner.egraph).unwrap();
            if runner.egraph.eq(&i1, &i2) {
                #[cfg(feature = "explanations")]
                println!("{}", eg.explain_equivalence(start, goal).to_string(&eg));
                return Ok(());
            }
        }
        if runner.iterations.len() >= steps - 1 {
            let msg = format!(
                "Start term {} did not reach goal term {} in {} steps",
                start, goal, steps
            );
            return Err(msg.to_string());
        }
        Ok(())
    });
}

fn assert_reaches<L, N>(start: &str, goal: &str, rewrites: &[Rewrite<L, N>], steps: usize)
where
    L: Language + 'static,
    N: Analysis<L> + 'static,
{
    let start: RecExpr<L> = RecExpr::parse(start).unwrap();
    let goal: RecExpr<L> = RecExpr::parse(goal).unwrap();
    let steps = steps;
    let mut eg: EGraph<L, N> = EGraph::new();
    eg.add_expr(start.clone());

    let mut runner = Runner::<L, N, ()>::new()
        .with_expr(&start)
        .with_iter_limit(60)
        .with_iter_limit(steps)
        .with_hook(reach_hook(&start, &goal, steps));
    let report = runner.run(rewrites);

    if let StopReason::Other(_) = report.stop_reason {
        eg.dump();
        assert!(false);
    }
}
