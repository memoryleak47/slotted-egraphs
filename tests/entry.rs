pub use std::hash::Hash;
pub use slotted_egraphs::*;

pub use symbol_table::GlobalSymbol as Symbol;

mod lamcalc;
pub use lamcalc::*;

mod arith;
pub use arith::*;

mod array;
pub use array::*;

mod lambda;
pub use lambda::*;

mod lambda_let;
pub use lambda_let::*;

mod rise;
pub use rise::*;

mod symbol;
pub use symbol::*;

mod var;
pub use var::*;

mod fgh;
pub use fgh::*;

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

pub fn term<L: Language>(s: &str, eg: &mut EGraph<L>) -> RecExpr<L> {
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
    let s1 = term(s1, eg);
    let s2 = term(s2, eg);
    eg.explain_equivalence(s1, s2).show_expr(eg);
    eg.check();
}

