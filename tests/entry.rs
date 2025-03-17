pub use slotted_egraphs::*;
pub use std::hash::Hash;

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
