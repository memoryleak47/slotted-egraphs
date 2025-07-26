use std::ops::Index;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
// slotmap m, has m[x] = m.slice[x.n], we return Slot::MISSING for unmapped slots.
// invariant: m.slice is minimal: m[m.len()-1] != Slot:MISSING.
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
        self.iter_unfiltered().all(|(x, y)| x == y)
    }

    // corresponds to (self * other)[x] = self[other[x]], just like in the paper.
    pub fn compose(&self, other: &SlotMap) -> SlotMap {
        let ib = self.img_bound();
        let mut slice = vec![Slot::MISSING; ib.n as usize];

        for (x, y) in other.iter_unfiltered() {
            slice[x.n as usize] = self[y];
        }

        while slice.last() == Some(&Slot::MISSING) { slice.pop(); }
        let slice = slice.into_boxed_slice();
        SlotMap { slice }
    }

    pub fn inverse(&self) -> SlotMap {
        let ib = self.img_bound();
        let mut slice = vec![Slot::MISSING; ib.n as usize].into_boxed_slice();
        for (x, y) in self.iter() {
            // we know self[x] = y.
            // thus slice[y] = x.
            slice[y.n as usize] = x;
        }
        SlotMap { slice }
    }

    // exclusive upper bound of all domain (aka key) slots.
    fn dom_bound(&self) -> Slot {
        Slot { n: self.slice.len() as u32 }
    }

    // exclusive upper bound of all img (aka value) slots.
    fn img_bound(&self) -> Slot {
        match self.img().max() {
            Some(Slot { n }) => Slot { n: n+1 },
            None => Slot { n: 0 },
        }
    }

    // TODO impl into_iter
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)> {
        self.iter_unfiltered().filter(|(_, y)| *y != Slot::MISSING)
    }

    fn iter_unfiltered(&self) -> impl Iterator<Item=(Slot, Slot)> {
        self.slice.iter().enumerate()
                  .map(|(i, x)| (Slot { n: i as u32 }, *x))
    }

    fn dom(&self) -> impl Iterator<Item=Slot> {
        self.iter().map(|(x, _)| x)
    }

    fn img(&self) -> impl Iterator<Item=Slot> {
        self.iter().map(|(_, y)| y)
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
