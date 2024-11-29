use crate::*;

/// Ids identify e-classes.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);
pub type AppliedId = Applied<Id>;

// under-applied Slots correspond to redundant slots. (bound slots don't exist anymore)
pub trait Applicable {
    fn access_slots_mut(&mut self) -> impl Iterator<Item=&mut Slot>;
    fn access_slots(&self) -> impl Iterator<Item=Slot>;

    fn apply_slotmap(&self, m: &SlotMap) -> Self where Self: Clone {
        let mut c = self.clone();
        for x in c.access_slots_mut() {
            *x = m[*x];
        }
        c
    }
}

// m * t
pub struct Applied<T> {
    m: SlotMap,
    t: T,
}

impl<T> Applicable for Applied<T> {
    fn access_slots_mut(&mut self) -> impl Iterator<Item=&mut Slot> {
        self.m.values_mut()
    }

    fn access_slots(&self) -> impl Iterator<Item=Slot> {
        self.m.values_immut().copied()
    }
}
