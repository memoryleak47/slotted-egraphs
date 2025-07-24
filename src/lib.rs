use std::collections::HashSet;

mod slotmap;
pub use slotmap::*;

mod group;
pub use group::*;

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

struct Id(usize);

impl Unionfind {
    pub fn add(&mut self, arity: usize) -> Id { // should this return AppliedId?
        let i = Id(self.classes.len());
        self.classes.push(Class {
            leader: todo!(),
            group: todo!(),
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
