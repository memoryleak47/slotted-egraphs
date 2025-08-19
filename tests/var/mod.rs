#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

define_language! {
    pub enum Var {
        F(Slot, Slot) = "f",
    }
}

#[test]
fn xy_eq_yz_causes_redundancy() {
    let mut eg = EGraph::<Var>::default();
    let a = eg.add_expr(RecExpr::parse("(f $0 $1)").unwrap());
    let b = eg.add_expr(RecExpr::parse("(f $1 $2)").unwrap());
    eg.union(&a, &b);
    eg.dump();
    let ids = eg.ids();
    assert_eq!(ids.len(), 1);
    let id = ids[0];
    assert!(eg.slots(id).is_empty());
}
