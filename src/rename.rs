use crate::*;

// The "rename" action * on Applicable.
impl<'a, T: Applicable> Mul<T> for &'a SlotMap {
    type Output = T;

    fn mul(self, mut t: T) -> T {
        t.apply_slotmap_inplace(self);
        t
    }
}

impl<L: SlotMapLike, R: SlotMapLike, T: Applicable> Mul<T> for Compose<L, R> {
    type Output = T;

    fn mul(self, mut t: T) -> T {
        t.apply_slotmap_inplace(self);
        t
    }
}

// Building Compose.
impl<'a, 'b> Mul<&'b SlotMap> for &'a SlotMap {
    type Output = Compose<&'a SlotMap, &'b SlotMap>;

    fn mul(self, other: &'b SlotMap) -> Self::Output {
        Compose(self, other)
    }
}

impl<'b, L: SlotMapLike, R: SlotMapLike> Mul<&'b SlotMap> for Compose<L, R> {
    type Output = Compose<Compose<L, R>, &'b SlotMap>;

    fn mul(self, other: &'b SlotMap) -> Self::Output {
        Compose(self, other)
    }
}

// SlotMap * Id = AppliedId
impl Mul<Id> for SlotMap {
    type Output = AppliedId;

    fn mul(self, id: Id) -> AppliedId {
        AppliedBy(self, id)
    }
}

// The "rename" action * on &HashSet<Slot>.
impl<'a, 's> Mul<&'s HashSet<Slot>> for &'a SlotMap {
    type Output = HashSet<Slot>;

    fn mul(self, s: &'s HashSet<Slot>) -> HashSet<Slot> {
        s.iter().map(|x| self[*x]).collect()
    }
}

impl<'s, L: SlotMapLike, R: SlotMapLike> Mul<&'s HashSet<Slot>> for Compose<L, R> {
    type Output = HashSet<Slot>;

    fn mul(self, s: &'s HashSet<Slot>) -> HashSet<Slot> {
        s.iter().map(|x| self.map(*x).unwrap()).collect()
    }
}
