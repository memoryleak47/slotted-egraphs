use crate::*;

use std::cmp::Ordering;

impl CostFunction<RiseENode> for AstSizeNoLet {
    type Cost = MyCost;

    fn cost<C>(enode: &RiseENode, costs: C) -> MyCost where C: Fn(Id) -> MyCost {
        if let RiseENode::Let(..) = enode {
            MyCost::Infinite
        } else {
            let mut s = MyCost::Finite(1);
            for x in enode.applied_id_occurences() {
                s = s.add(&costs(x.id));
            }
            s
        }
    }
}
