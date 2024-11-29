use crate::*;

/// Ids identify e-classes.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);
pub type AppliedId = Applied<Id>;

// under-applied Slots correspond to redundant slots. (bound slots don't exist anymore)
pub trait Applicable {
    fn access_slots_mut(&mut self) -> impl Iterator<Item=&mut Slot>;
    fn access_slots(&self) -> impl Iterator<Item=Slot>;

    fn slots(&self) -> HashSet<Slot> {
        self.access_slots().collect()
    }

    fn apply_slotmap(&self, m: &SlotMap) -> Self where Self: Clone {
        let mut c = self.clone();
        c.apply_slotmap_inplace(m);
        c
    }

    fn apply_slotmap_fresh(&self, m: &SlotMap) -> Self where Self: Clone {
        let mut c = self.clone();
        c.apply_slotmap_fresh_inplace(m);
        c
    }

    fn apply_slotmap_inplace(&mut self, m: &SlotMap) {
        for x in self.access_slots_mut() {
            *x = m[*x];
        }
    }

    fn apply_slotmap_fresh_inplace(&mut self, m: &SlotMap) {
        let mut m = m.clone();
        for x in self.access_slots_mut() {
            if let Some(y) = m.get(*x) {
                *x = y;
            } else {
                let y = Slot::fresh();
                m.insert(*x, y);
                *x = y;
            }
        }
    }
}

// m * t
#[derive(Clone, PartialEq, Eq)]
pub struct Applied<T>(pub SlotMap, pub T);

impl<T> Applicable for Applied<T> {
    fn access_slots_mut(&mut self) -> impl Iterator<Item=&mut Slot> {
        let Applied(m, t) = self;
        m.values_mut()
    }

    fn access_slots(&self) -> impl Iterator<Item=Slot> {
        let Applied(m, t) = self;
        m.values_immut().copied()
    }
}

impl Applicable for SlotMap {
    fn access_slots_mut(&mut self) -> impl Iterator<Item=&mut Slot> {
        self.values_mut()
    }

    fn access_slots(&self) -> impl Iterator<Item=Slot> {
        self.values_immut().copied()
    }
}

impl<T: Applicable> Applied<T> {
    fn apply(self) -> T {
        let Applied(m, mut t) = self;
        t.apply_slotmap_inplace(&m);
        t
    }
}

impl<'a, T: Applicable> Mul<T> for &'a SlotMap {
    type Output = T;

    fn mul(self, mut t: T) -> T {
        t.apply_slotmap_inplace(self);
        t
    }
}

impl<T: Applicable> Mul<T> for SlotMap {
    type Output = T;

    fn mul(self, mut t: T) -> T {
        t.apply_slotmap_inplace(&self);
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

impl AppliedId {
    pub fn m(&self) -> &SlotMap {
        let Applied(m, _) = self;
        m
    }

    pub fn id(&self) -> Id {
        let Applied(_, id) = self;
        *id
    }
}
