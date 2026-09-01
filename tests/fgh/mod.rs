#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;
use slotted_egraphs::*;

define_language! {
    pub enum Fgh {
        F(Slot, Slot) = "f",
        G(Slot, Slot) = "g",
        H(Slot, Slot) = "h",
    }
}

#[test]
fn transitive_symmetry() {
    let eg: &mut EGraph<Fgh> = &mut EGraph::default();
    equate("(f $1 $2)", "(g $2 $1)", eg);
    equate("(g $1 $2)", "(h $1 $2)", eg);
    eg.dump();
    explain("(f $1 $2)", "(h $2 $1)", eg);
}

// bug found by oflatt-claude.
#[test]
fn shrink_with_symmetry() {
    let eg: &mut EGraph<Fgh> = &mut EGraph::default();
    equate("(f $0 $1)", "(f $1 $0)", eg);
    equate("(f $0 $1)", "(g $1 $1)", eg);

    let ids = eg.ids();
    eg.dump();
    assert_eq!(ids.len(), 1);

    // here I disagree with claude.
    // There should be 0 slots, instead of 1.
    // After all, as (f $0 $1) is self-symmetric, making $0 redundant should make $1 redundant too.
    assert_eq!(eg.slots(ids[0]).len(), 0);
}
