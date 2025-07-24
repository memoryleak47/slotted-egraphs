use std::collections::HashSet;

struct Unionfind {
    classes: Vec<Class>,
}

struct Class {
    leader: AppliedId,
    group: Group,
    arity: usize, // for now. will probably be stored somewhere else later.
}

struct AppliedId {
    id: Id,
    m: SlotMap,
}

struct SlotMap(Box<[Slot]>);

struct Id(usize);

struct Group(HashSet<SlotMap>);

struct Slot(u32);

impl Unionfind {
    pub fn add(&mut self, arity: usize) -> Id { // should this return AppliedId?
        let i = Id(self.classes.len());
        self.classes.push(Class {
            leader: todo!(),
            group: Group(HashSet::new()),
            arity,
        });
        i
    }

    pub fn union(&mut self, x: &AppliedId, y: &AppliedId) {
        todo!()
    }

    pub fn eq(&self, x: &AppliedId, y: &AppliedId) -> bool {
        todo!()
    }
}
