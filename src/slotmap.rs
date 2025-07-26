use std::ops::Index;

// Types of slotmaps:
// - unionfind slotmaps: equating different Ids.
// -- redundant slots possible! This corresponds to "holes" in the slotmap.
// -- But only in one direction! the unionfind only gets more redundant the further you go.
//
// - group slotmaps: expressing symmetries
// -- (keyset = valueset, perfect for positional perms)
//
// - shape slotmaps: expressing how shape slots relate to e-class slots.
// -- redundant slots are again a thing! But again, the class always has less (<=) slots than the shape.
//
// Exceptions:
// - When the user calls "union", we can introduce redundant slots on both sides, so no (<=) guarantee anymore.
// -- technically, if we give it two AppliedIds, we could get the (<=) back.

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
// slotmap m maps x to m.0[x.0] if it's not Slot::missing
// we require that a slotmap m has m.0 is a reordering of 0..m.len()
pub struct SlotMap(pub Box<[Slot]>);

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot(pub u32);

impl SlotMap {
    pub fn identity(arity: usize) -> SlotMap {
        let v: Box<[_]> = (0..arity).map(|x| Slot(x as _)).collect();
        SlotMap(v)
    }

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
