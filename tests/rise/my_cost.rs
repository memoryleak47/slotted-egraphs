use crate::*;

use std::cmp::Ordering;

impl CostFunction<Rise> for AstSizeNoLet {
    type Cost = MyCost;

    fn cost<C>(&self, enode: &Rise, costs: C) -> MyCost
    where
        C: Fn(Id) -> MyCost,
    {
        if let Rise::Let(..) = enode {
            MyCost::Infinite
        } else {
            let mut s = MyCost::Finite(1);
            for x in enode.applied_id_occurrences() {
                s = s.add(&costs(x.id));
            }
            s
        }
    }
}
