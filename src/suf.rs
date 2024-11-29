use crate::*;

pub struct SlottedUF<C>(Vec<SUFClass<C>>);

struct SUFClass<C> {
    leader: AppliedId,
    s: HashSet<Slot>,
    g: Group<SlotMap>,

    c: C,
}

impl<C> SlottedUF<C> {
    pub fn new() -> Self {
        SlottedUF(Vec::new())
    }

    pub fn add(&mut self, s: HashSet<Slot>, c: C) -> AppliedId {
        todo!()
    }

    pub fn union(&mut self, x: &AppliedId, y: &AppliedId) {
        todo!()
    }

    pub fn is_equal(&self, x: &AppliedId, y: &AppliedId) {
        todo!()
    }
}
