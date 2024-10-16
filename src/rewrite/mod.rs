use crate::*;
use std::any::Any;

mod enode_or;
pub use enode_or::*;

mod ematch;
pub use ematch::*;

mod pattern_subst;
pub use pattern_subst::*;

/// An equational rewrite rule.
pub struct Rewrite<L: Language, N: Analysis<L> = ()> {
    pub(crate) searcher: Box<dyn Fn(&EGraph<L, N>) -> Box<dyn Any>>,
    pub(crate) applier: Box<dyn Fn(Box<dyn Any>, &mut EGraph<L, N>)>,
}

/// Use this type when you want to build your own [Rewrite].
///
/// The type parameter `T` can be anything you want, as long as the `searcher` creates it, and the `applier` consumes it.
///
/// In most cases, `T` is a [Subst].
pub struct RewriteT<L: Language, N: Analysis<L>, T: Any> {
    pub searcher: Box<dyn Fn(&EGraph<L, N>) -> T>,
    pub applier: Box<dyn Fn(T, &mut EGraph<L, N>)>,
}


impl<L: Language + 'static, N: Analysis<L> + 'static, T: 'static> RewriteT<L, N, T> {
    /// Use this function to convert it to an actual [Rewrite].
    pub fn into(self) -> Rewrite<L, N> {
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

/// Applies each given rewrite rule to the E-Graph once.
pub fn apply_rewrites<L: Language, N: Analysis<L>>(eg: &mut EGraph<L, N>, rewrites: &[Rewrite<L, N>]) {
    let ts: Vec<Box<dyn Any>> = rewrites.iter().map(|rw| (*rw.searcher)(eg)).collect();
    for (rw, t) in rewrites.iter().zip(ts.into_iter()) {
        (*rw.applier)(t, eg);
    }
}

impl<L: Language + 'static, N: Analysis<L> + 'static> Rewrite<L, N> {
    /// Create a rewrite rule by specifing a left- and right-hand side of your equation.
    pub fn new(rule: &str, a: &str, b: &str) -> Self {
        Self::new_if(rule, a, b, |_| true)
    }

    /// Create a conditional rewrite rule.
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
}
