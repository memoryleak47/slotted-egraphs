use crate::*;
use std::any::Any;

pub struct RewriteT<L: Language, T: Any> {
    pub searcher: Box<dyn Fn(&EGraph<L>) -> T>,
    pub applier: Box<dyn Fn(T, &mut EGraph<L>)>,
}

impl<L: Language + 'static, T: 'static> RewriteT<L, T> {
    pub fn into(self) -> Rewrite<L> {
        let searcher = self.searcher;
        let applier = self.applier;
        Rewrite {
            searcher: Box::new(move |eg| Box::new((*searcher)(eg))),
            applier: Box::new(move |t, eg| (*applier)(any_to_t(t), eg))
        }
    }
}

pub type Rewrite<L> = RewriteT<L, Box<dyn Any>>;

fn any_to_t<T: Any>(t: Box<dyn Any>) -> T {
    *t.downcast().unwrap()
}

pub fn do_rewrites<L: Language>(eg: &mut EGraph<L>, rewrites: &[Rewrite<L>]) {
    let ts: Vec<Box<dyn Any>> = rewrites.iter().map(|rw| (*rw.searcher)(eg)).collect();
    for (rw, t) in rewrites.iter().zip(ts.into_iter()) {
        (*rw.applier)(t, eg);
    }
}

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

