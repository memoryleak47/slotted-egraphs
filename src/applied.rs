use crate::*;

/// Ids identify e-classes.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);
pub type AppliedId = Applied<Id>;

// under-applied Slots correspond to redundant slots. (bound slots don't exist anymore)
pub trait Applicable: Access<Slot> {
    fn slots(&self) -> HashSet<Slot> {
        struct SlotsHandler;

        impl<'a> Handler<&'a Slot> for SlotsHandler {
            type R = HashSet<Slot>;
            fn call(self, it: impl Iterator<Item=&'a Slot>) -> HashSet<Slot> {
                it.copied().collect()
            }
        }

        self.access(SlotsHandler)
    }

    fn apply_slotmap(&self, m: SlotMapRef<impl SlotMapLike>) -> Self where Self: Clone {
        let mut c = self.clone();
        c.apply_slotmap_inplace(m);
        c
    }

    fn apply_slotmap_inplace<M: SlotMapLike>(&mut self, m: SlotMapRef<M>) {
        struct ApplySlotMapHandler<M> { m: SlotMapRef<M> }

        impl<'a, M: SlotMapLike> Handler<&'a mut Slot> for ApplySlotMapHandler<M> {
            type R = ();
            fn call(self, it: impl Iterator<Item=&'a mut Slot>) {
                for x in it {
                    *x = self.m.0.map(*x).unwrap();
                }
            }
        }

        self.access_mut(ApplySlotMapHandler { m })
    }

    fn apply_slotmap_fresh_inplace(&mut self, m: SlotMap) {
        struct ApplySlotMapFreshHandler { m: SlotMap }

        impl<'a, 'm> Handler<&'a mut Slot> for ApplySlotMapFreshHandler {
            type R = ();
            fn call(mut self, it: impl Iterator<Item=&'a mut Slot>) {
                for x in it {
                    if let Some(y) = self.m.get(*x) {
                        *x = y;
                    } else {
                        let y = Slot::fresh();
                        self.m.insert(*x, y);
                        *x = y;
                    }
                }
            }
        }

        self.access_mut(ApplySlotMapFreshHandler { m: m.clone() })
    }
}

impl<T: Access<Slot>> Applicable for T {}

// m * t
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AppliedBy<M, T>(pub M, pub T);
pub type Applied<T> = AppliedBy<SlotMap, T>;

impl<T> Access<Slot> for Applied<T> {
    fn access_mut<'a, H: Handler<&'a mut Slot>>(&'a mut self, h: H) -> H::R {
        let AppliedBy(m, _) = self;
        m.access_mut(h)
    }

    fn access<'a, H: Handler<&'a Slot>>(&'a self, h: H) -> H::R {
        let AppliedBy(m, _) = self;
        m.access(h)
    }

    fn into_access<H: Handler<Slot>>(self, h: H) -> H::R {
        let AppliedBy(m, _) = self;
        m.into_access(h)
    }
}

impl Access<Slot> for SlotMap {
    fn access_mut<'a, H: Handler<&'a mut Slot>>(&'a mut self, h: H) -> H::R {
        h.call(self.values_mut())
    }

    fn access<'a, H: Handler<&'a Slot>>(&'a self, h: H) -> H::R {
        h.call(self.values_immut())
    }

    fn into_access<H: Handler<Slot>>(self, h: H) -> H::R {
        h.call(self.into_iter().map(|(_, y)| y))
    }
}

impl<T: Applicable> Applied<T> {
    fn apply(self) -> T {
        let AppliedBy(m, mut t) = self;
        t.apply_slotmap_inplace(m.rf());
        t
    }
}

impl AppliedId {
    pub fn m(&self) -> SlotMapRef<&SlotMap> {
        let AppliedBy(m, _) = self;
        m.rf()
    }

    pub fn id(&self) -> Id {
        let AppliedBy(_, id) = self;
        *id
    }
}
