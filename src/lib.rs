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
    leader_id: Id,
    leader_m: SlotMap,
}

#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Id { n: usize }

impl Unionfind {
    pub fn add(&mut self, arity: usize) -> Id {
        let i = Id { n: self.classes.len() };
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
