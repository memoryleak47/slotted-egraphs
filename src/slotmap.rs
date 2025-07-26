use std::ops::Index;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
// slotmap m, has m[x] = m.0[x.0], we return Slot::missing for unmapped slots.
pub struct SlotMap(pub Box<[Slot]>);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot(pub u32);

impl SlotMap {
    pub fn identity(arity: usize) -> SlotMap {
        let v: Box<[_]> = (0..arity).map(|x| Slot(x as _)).collect();
        SlotMap(v)
    }

    // corresponds to (self * other)[x] = self[other[x]], just like in the paper.
    pub fn compose(&self, other: &SlotMap) -> SlotMap {
        todo!()
    }

    pub fn inverse(&self) -> SlotMap {
        // TODO handle missing entries.
        let v: Box<[_]> = self.0.iter().copied().rev().collect();
        SlotMap(v)
    }
}

static MISSING_REF: Slot = Slot::missing();

impl Index<Slot> for SlotMap {
    type Output = Slot;

    fn index(&self, index: Slot) -> &Slot {
        self.0.get(index.0 as usize).unwrap_or(&MISSING_REF)
    }
}

impl Slot {
    pub const fn missing() -> Slot {
        Slot(u32::MAX)
    }
}
