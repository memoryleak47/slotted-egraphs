use std::ops::Index;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SlotMap(pub Box<[Slot]>);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot(pub usize);

impl SlotMap {
    pub fn identity(arity: usize) -> SlotMap {
        let v: Vec<_> = (0..arity).map(Slot).collect();
        SlotMap(v.into_boxed_slice())
    }

    pub fn compose(&self, other: &SlotMap) -> SlotMap {
        todo!()
    }

    pub fn inverse(&self) -> SlotMap {
        let v: Vec<_> = self.0.iter().copied().rev().collect();
        SlotMap(v.into_boxed_slice())
    }
}

impl Index<Slot> for SlotMap {
    type Output = Slot;

    fn index(&self, index: Slot) -> &Slot {
        &self.0[index.0]
    }
}
