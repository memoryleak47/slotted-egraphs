use crate::*;

// The "rename" action * on Applicable.
impl<M: SlotMapLike, T: Applicable> Mul<T> for SlotMapRef<M> {
    type Output = T;

    fn mul(self, mut t: T) -> T {
        t.apply_slotmap_inplace(self);
        t
    }
}

// Building Compose.
impl<M1: SlotMapLike, M2: SlotMapLike> Mul<SlotMapRef<M2>> for SlotMapRef<M1> {
    type Output = SlotMapRef<Compose<M1, M2>>;

    fn mul(self, other: SlotMapRef<M2>) -> Self::Output {
        SlotMapRef(Compose(self.0, other.0))
    }
}

// SlotMap * t = Applied(t)
impl<T> Mul<T> for SlotMap {
    type Output = Applied<T>;

    fn mul(self, t: T) -> Applied<T> {
        AppliedBy(self, t)
    }
}

// The "rename" action * on &HashSet<Slot>.
impl<'s, M: SlotMapLike> Mul<&'s HashSet<Slot>> for SlotMapRef<M> {
    type Output = HashSet<Slot>;

    fn mul(self, s: &'s HashSet<Slot>) -> HashSet<Slot> {
        s.iter().map(|x| self.0.map(*x).unwrap()).collect()
    }
}
