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

// Extracting from a class whose cheapest node names a slot the class does not.
// `(f $0 $1)` equated with `(g $1 $1)` leaves `$0` redundant, so the class keeps a node
// wider than itself -- which Def. 4 permits -- and extraction has to name that slot.
#[test]
fn extract_with_redundant_slot() {
    let eg: &mut EGraph<Fgh> = &mut EGraph::default();
    equate("(f $0 $1)", "(g $1 $1)", eg);

    let ids = eg.ids();
    assert_eq!(ids.len(), 1);

    // the class invoked on its own slots
    let id = AppliedId::new(ids[0], SlotMap::identity(&eg.slots(ids[0])));
    let out = ast_size_extract(&id, eg);
    // whichever node is cheapest, extraction must produce a term rather than panic
    assert!(!out.to_string().is_empty());
}
