use crate::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// Slots represent the concept of Variable Names.
///
/// Internally, they are just a number.
//
// An ENode contains three kinds of slots:
// - free / exposed
// - lambda
// - internal (not really part of the ENode API, it's rather the exposed slots of its children)
//
// A slot is "flexible" if it's free or lambda.
pub struct Slot(i64);

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AppliedId {
    pub id: Id,

    // m is always a bijection!
    // m maps the slots from `id` (be it ENode::slots() in a RecExpr, or EGraph::slots(Id) for eclasses) to the slots that we insert into it.
    // m.keys() == id.slots
    pub m: SlotMap,
}

// The AppliedIds in `node` are ignored (any typically set to AppliedId::null()). They are replaced by the children RecExpr.
// A non-fancy version of RecExpr that uses the slots as "names".
#[derive(Clone, PartialEq, Eq)]
pub struct RecExpr<L: Language> {
    pub node: L,
    pub children: Vec<RecExpr<L>>,
}


impl AppliedId {
    pub fn new(id: Id, m: SlotMap) -> Self {
        let s = AppliedId { id, m };
        s.check();
        s
    }

    pub(crate) fn check(&self) {
        assert!(self.m.is_bijection());
    }

    #[track_caller]
    pub fn apply_slotmap(&self, m: &SlotMap) -> AppliedId {
        if CHECKS {
            assert!(m.keys().is_superset(&self.slots()), "AppliedId::apply_slotmap: The SlotMap doesn't map all free slots!");
        }
        self.apply_slotmap_partial(m)
    }

    pub fn apply_slotmap_partial(&self, m: &SlotMap) -> AppliedId {
        AppliedId::new(
            self.id,
            self.m.compose_partial(m),
        )
    }

    pub fn apply_slotmap_fresh(&self, m: &SlotMap) -> AppliedId {
        AppliedId::new(
            self.id,
            self.m.compose_fresh(m),
        )
    }

    pub fn slots(&self) -> HashSet<Slot> {
        self.m.values()
    }

    // ordered!
    pub fn slots_mut(&mut self) -> Vec<&mut Slot> {
        self.m.values_mut().collect()
    }

    pub fn null() -> Self {
        AppliedId { id: Id(0), m: SlotMap::new() }
    }
}

impl Slot {
    /// Generates a fresh slot (with negative sign).
    /// The only way to create an equivalent Slot is by copying this one.
    /// Hence we can rule out any form of naming collisions with this Slot.
    // (In theory, another thread could also create the same Slot, but we don't do multithreading for now so it's fine)
    pub fn fresh() -> Self {
        use std::cell::RefCell;

        // We choose ThreadLocal here, so that tests (that run in parallel threads) don't interfere.
        // There were situations, where different Slot-names did affect hashmap ordering, and with that actually changed the output of the algorithm.
        // Using this, all tests should run deterministically.

        thread_local! {
            static CTR: RefCell<i64> = RefCell::new(-1);
        }

        let u = CTR.with_borrow(|v| *v);

        CTR.with_borrow_mut(|v| *v -= 1);

        Slot(u)
    }

    /// creates the slot `s_u`.
    /// These slots can never collide with the Slots returned from `Slot::fresh()` due to their sign.
    pub fn new(u: usize) -> Slot {
        Slot(u as i64)
    }

    pub fn to_string(&self) -> String {
        format!("s{}", self.0)
    }

    pub fn new_unchecked(i: i64) -> Slot {
        Slot(i)
    }
}
