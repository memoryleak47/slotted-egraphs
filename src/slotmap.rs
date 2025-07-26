use std::ops::Index;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
// slotmap m, has m[x] = m.slice[x.n], we return Slot::missing for unmapped slots.
// invariant: m.slice is minimal: m[m.len()-1] != Slot:missing.
pub struct SlotMap {
    pub slice: Box<[Slot]>
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Slot {
    pub n: u32,
}

impl SlotMap {
    pub fn identity(arity: usize) -> SlotMap {
        let v: Box<[_]> = Slot::it().take(arity).collect();
        SlotMap { slice: v }
    }

    pub fn is_identity(&self) -> bool {
        self.slice.iter().enumerate().all(|(i, x)| i as u32 == x.n)
    }

    // corresponds to (self * other)[x] = self[other[x]], just like in the paper.
    pub fn compose(&self, other: &SlotMap) -> SlotMap {
        todo!()
    }

    pub fn inverse(&self) -> SlotMap {
        let ib = self.img_bound();
        let mut slice = vec![Slot::MISSING; ib.n as usize].into_boxed_slice();
        for (i, x) in self.slice.iter().enumerate() {
            // we know self[i] = x.
            // thus slice[x] = i.
            slice[x.n as usize] = Slot { n: i as u32 };
        }
        SlotMap { slice }
    }

    // exclusive upper bound of all domain (aka key) slots.
    fn dom_bound(&self) -> Slot {
        Slot { n: self.slice.len() as u32 }
    }

    // exclusive upper bound of all img (aka value) slots.
    fn img_bound(&self) -> Slot {
        let it = self.slice.iter().filter(|x| **x != Slot::MISSING);
        match it.max() {
            Some(Slot { n }) => Slot { n: n+1 },
            None => Slot { n: 0 },
        }
    }
}

impl Index<Slot> for SlotMap {
    type Output = Slot;

    fn index(&self, index: Slot) -> &Slot {
        self.slice.get(index.n as usize).unwrap_or(&MISSING_REF)
    }
}

pub static MISSING_REF: Slot = Slot::MISSING;

impl Slot {
    pub const MISSING: Slot = Slot { n: u32::MAX };

    pub fn it() -> impl Iterator<Item=Slot> {
        (0..).map(|n| Slot { n })
    }
}
