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

/// Shrinking a class whose group is non-trivial used to panic in `build_ot`.
///
/// `restrict_proven` filters a permutation by its keys, so restricting the swap
/// `{$0->$1, $1->$0}` to `cap = {$1}` leaves `{$1->$0}`, which carries a surviving slot
/// out of `cap` and is therefore not a permutation of it. Composing it later indexes a
/// slot the map does not have:
///
/// ```text
/// SlotMap::index($f1): index missing!
/// ```
///
/// `shrink_slots` already computes `final_cap`, which drops the orbit of every newly
/// redundant slot, but used `cap`. Taking `final_cap` avoids the panic.
///
/// The surviving slot count is only asserted to be what it is, not to be optimal: one
/// slot is redundant and the class is symmetric in the two, so a stronger shrink may
/// well be justified. That is a separate question from the crash.
#[test]
fn shrink_with_symmetry() {
    let eg: &mut EGraph<Fgh> = &mut EGraph::default();
    equate("(f $0 $1)", "(f $1 $0)", eg);
    equate("(f $0 $1)", "(g $1 $1)", eg);

    let ids = eg.ids();
    assert_eq!(ids.len(), 1);
    assert_eq!(eg.slots(ids[0]).len(), 1);
}
