use std::collections::HashSet;

mod slotmap;
pub use slotmap::*;

mod group;
pub use group::*;

struct Unionfind {
    classes: Vec<Class>,
}

enum Class {
    Leader(LeaderClass),
    Follower(FollowerClass),
}

struct LeaderClass {
    // NOTE: arity of the class is stored in the group.
    group: Group,
}

struct FollowerClass {
    leader: AppliedId,
}

struct AppliedId {
    id: Id,
    m: SlotMap,
}

struct Id(usize);

impl Unionfind {
    pub fn add(&mut self, arity: usize) -> Id { // should this return AppliedId?
        let i = Id(self.classes.len());
        self.classes.push(Class::Leader(LeaderClass {
            group: Group::trivial(arity),
        }));
        i
    }

    // m maps from slots(x) -> slots(y).
    pub fn union(&mut self, x: Id, y: Id, m: &SlotMap) {
        todo!()
    }

    // m maps from slots(x) -> slots(y).
    pub fn eq(&self, x: Id, y: Id, m: &SlotMap) -> bool {
        todo!()
    }
}
