pub use std::hash::Hash;
pub use slotted_egraphs::*;

pub use symbol_table::GlobalSymbol as Symbol;

mod lamcalc;
pub use lamcalc::*;

mod i_arith;
pub use i_arith::*;

mod i_array;
pub use i_array::*;

mod i_lambda;
pub use i_lambda::*;

mod i_let;
pub use i_let::*;

mod i_main;
pub use i_main::*;

mod i_rise;
pub use i_rise::*;

mod i_symbol;
pub use i_symbol::*;

pub fn singleton_set<T: Eq + Hash>(t: T) -> HashSet<T> {
    [t].into_iter().collect()
}

// TODO remove those:
// Indirect rewrites.

pub fn mk_named_rewrite_if<L: Language + 'static>(rule: &str, a: Pattern<L>, b: Pattern<L>, cond: impl Fn(&Subst) -> bool + 'static) -> Rewrite<L> {
    let rule = rule.to_string();
    let a2 = a.clone();
    RewriteT {
        searcher: Box::new(move |eg| {
            let x: Vec<Subst> = ematch_all(eg, &a);
            x
        }),
        applier: Box::new(move |substs, eg| {
            for subst in substs {
                if cond(&subst) {
                    eg.union_instantiations(&a2, &b, &subst, Some(rule.to_string()));
                }
            }
        }),
    }.into()
}

pub fn mk_rewrite_if<L: Language + 'static>(a: Pattern<L>, b: Pattern<L>, cond: impl Fn(&Subst) -> bool + 'static) -> Rewrite<L> {
    mk_named_rewrite_if("<no rule name>", a, b, cond)
}

pub fn mk_named_rewrite<L: Language + 'static>(rule: &str, a: Pattern<L>, b: Pattern<L>) -> Rewrite<L> {
    mk_named_rewrite_if(rule, a, b, |_| true)
}

pub fn mk_rewrite<L: Language + 'static>(a: Pattern<L>, b: Pattern<L>) -> Rewrite<L> {
    mk_rewrite_if(a, b, |_| true)
}

// Direct rewrites.

pub fn rewrite_if<L: Language>(eg: &mut EGraph<L>, a: Pattern<L>, b: Pattern<L>, cond: impl Fn(&Subst) -> bool) {
    for subst in ematch_all(eg, &a) {
        if cond(&subst) {
            eg.union_instantiations(&a, &b, &subst, None);
        }
    }
}

pub fn rewrite<L: Language>(eg: &mut EGraph<L>, a: Pattern<L>, b: Pattern<L>) {
    rewrite_if(eg, a, b, |_| true);
}

pub fn rewrite_bi<L: Language>(eg: &mut EGraph<L>, a: Pattern<L>, b: Pattern<L>) {
    rewrite(eg, a.clone(), b.clone());
    rewrite(eg, b, a);
}

