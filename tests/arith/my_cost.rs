use crate::*;

use std::cmp::Ordering;

impl CostFunction<Arith> for AstSizeNoLet {
    type Cost = MyCost;

    fn cost<C>(&self, enode: &Arith, costs: C) -> MyCost where C: Fn(Id) -> MyCost {
        if let Arith::Let(..) = enode {
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
