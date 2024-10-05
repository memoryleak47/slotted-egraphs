use crate::*;
use std::any::Any;

pub struct RewriteT<L: Language, T: Any> {
    pub searcher: Box<dyn Fn(&EGraph<L>) -> T>,
    pub applier: Box<dyn Fn(T, &mut EGraph<L>)>,
}

pub struct Rewrite<L: Language> {
    pub searcher: Box<dyn Fn(&EGraph<L>) -> Box<dyn Any>>,
    pub applier: Box<dyn Fn(Box<dyn Any>, &mut EGraph<L>)>,
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

fn any_to_t<T: Any>(t: Box<dyn Any>) -> T {
    *t.downcast().unwrap()
}

pub fn apply_rewrites<L: Language>(eg: &mut EGraph<L>, rewrites: &[Rewrite<L>]) {
    let ts: Vec<Box<dyn Any>> = rewrites.iter().map(|rw| (*rw.searcher)(eg)).collect();
    for (rw, t) in rewrites.iter().zip(ts.into_iter()) {
        (*rw.applier)(t, eg);
    }
}

impl<L: Language + 'static> Rewrite<L> {
    pub fn new_if(rule: &str, a: &str, b: &str, cond: impl Fn(&Subst) -> bool + 'static) -> Self {
        let a = Pattern::parse(a).unwrap();
        let b = Pattern::parse(b).unwrap();
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

    pub fn new(rule: &str, a: &str, b: &str) -> Self {
        Self::new_if(rule, a, b, |_| true)
    }
}
