use crate::*;
use std::any::Any;

mod ematch;
pub use ematch::*;

mod pattern;
pub use pattern::*;

mod subst_method;
pub use subst_method::*;

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
/// Returns an indicator for whether the e-graph changed as a result.
pub fn apply_rewrites<L: Language, N: Analysis<L>>(eg: &mut EGraph<L, N>, rewrites: &[Rewrite<L, N>]) -> bool {
    let prog = eg.progress();
    
    let ts: Vec<Box<dyn Any>> = rewrites.iter().map(|rw| (*rw.searcher)(eg)).collect();
    for (rw, t) in rewrites.iter().zip(ts.into_iter()) {
        (*rw.applier)(t, eg);
    }
    
    prog != eg.progress()
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


#[derive(PartialEq, Eq)]
/// A Progress Measure to check saturation of an e-graph with.
pub struct ProgressMeasure {
    /// How many classes that were allocated in this e-graph. This measure is strictly growing.
    pub number_of_classes: usize,

    /// How many classes are still "live". If "number_of_classes" isn't changed, this can only decrease (by union).
    pub number_of_live_classes: usize,

    /// How many parameter-slots are still in the e-classes. If number_of_classes & number_of_live_classes isn't changed, this can only decrease (by proving a redundancy by union).
    pub sum_of_slots: usize,

    /// How many symmetries the egraphs knows. If number_of_classes & number_of_live_classes & sum_of_slots isn't changed, this can only increase (by proving a symmetry by union).
    pub sum_of_symmetries: usize,

}

impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    /// Computes the [ProgressMeasure] of this E-Graph.
    pub fn progress(&self) -> ProgressMeasure {
        let ids = self.ids();
        ProgressMeasure {
            number_of_classes: self.classes.len(),
            number_of_live_classes: ids.len(),
            sum_of_symmetries: ids.iter().map(|x| self.classes[x].group.count()).sum(),
            sum_of_slots: ids.iter().map(|x| self.slots(*x).len()).sum(),
        }
    }
}
