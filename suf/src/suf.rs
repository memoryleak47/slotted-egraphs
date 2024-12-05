use crate::*;

struct Suf {
    v: Vec<Class>,
}

struct Class {
    leader: AppliedId,
    slots: HashSet<Slot>,
    group: Group,
}

impl Suf {
    pub fn new() -> Self {
        Self { v: Vec::new() }
    }

    pub fn add(&mut self, slots: HashSet<Slot>) -> Id {
        let i = Id(self.v.len());
        let leader = AppliedId(SlotMap::identity(&slots), i);
        let group = Group::new(slots.clone(), Default::default());
        self.v.push(Class {
            leader,
            slots,
            group,
        });
        i
    }
}
