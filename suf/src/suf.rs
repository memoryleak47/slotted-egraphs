use crate::*;

struct Suf {
    v: Vec<Class>,
}

struct Class {
    leader: (SlotMap, Id),
    slots: HashSet<Slot>,
    group: Group,
}

impl Suf {
    pub fn new() -> Self {
        Self { v: Vec::new() }
    }

    pub fn add(&mut self, slots: HashSet<Slot>) -> Id {
        let i = Id(self.v.len());
        let leader = (SlotMap::identity(&slots), i);
        let group = Group::new(slots.clone(), Default::default());
        self.v.push(Class {
            leader,
            slots,
            group,
        });
        i
    }

    fn find(&mut self, x: &(SlotMap, Id)) -> (SlotMap, Id) {
        todo!()
    }

    fn compose(&mut self, eq1: &(Id, Id, SlotMap), eq2: &(Id, Id, SlotMap)) -> (Id, Id, SlotMap) {
        todo!()
    }

    pub fn union(&mut self, eq: &(Id, Id, SlotMap)) {
        // simplify
        // 1. add redundancies
        todo!()
    }
}
