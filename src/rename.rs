use crate::*;

// The "rename" action *.
impl<'a, T: Applicable> Mul<T> for &'a SlotMap {
    type Output = T;

    fn mul(self, mut t: T) -> T {
        t.apply_slotmap_inplace(self);
        t
    }
}

impl Mul<Id> for SlotMap {
    type Output = AppliedId;

    fn mul(self, id: Id) -> AppliedId {
        Applied(self, id)
    }
}

impl Mul<HashSet<Slot>> for SlotMap {
    type Output = HashSet<Slot>;

    fn mul(self, s: HashSet<Slot>) -> HashSet<Slot> {
        s.iter().map(|x| self[*x]).collect()
    }
}


