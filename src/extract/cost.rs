use crate::*;

use std::marker::PhantomData;

/// A cost function to guide extraction.
///
/// If you want to use your e-graph analysis in your cost function, then your cost function should hold a reference to the e-graph.
pub trait CostFunction<L: Language> {
    type Cost: Ord + Clone + Debug;
    fn cost<C>(&self, enode: &L, costs: C) -> Self::Cost where C: Fn(Id) -> Self::Cost;

    fn cost_rec(&self, expr: &RecExpr<L>) -> Self::Cost {
        let child_costs: Vec<Self::Cost> = expr.children.iter().map(|x| self.cost_rec(x)).collect();
        let c = |i: Id| child_costs[i.0].clone();
        let mut node = expr.node.clone();
        for (i, x) in node.applied_id_occurrences_mut().iter_mut().enumerate() {
            **x = AppliedId::new(Id(i), SlotMap::new());
        }
        self.cost(&node, c)
    }
}

/// The 'default' [CostFunction]. It measures the size of the abstract syntax tree of the corresponding term.
#[derive(Default)]
pub struct AstSize;

impl<L: Language> CostFunction<L> for AstSize {
    type Cost = u64;

    fn cost<C>(&self, enode: &L, costs: C) -> u64 where C: Fn(Id) -> u64 {
        let mut s: u64 = 1;
        for x in enode.applied_id_occurrences() {
            s = s.saturating_add(costs(x.id));
        }
        s
    }
}
