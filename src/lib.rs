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

    // TODO add path compression.
    fn find(&self, x: Id) -> AppliedId {
        match &self.classes[x.n] {
            Class::Leader(c) => AppliedId {
                id: x,
                m: SlotMap::identity(c.group.arity),
            },
            Class::Follower(c) => {
                let AppliedId { m, id } = &c.leader;
                let AppliedId { m: m2, id: id2 } = self.find(*id);
                AppliedId {
                    id: id2,
                    m: m.compose(&m2),
                }
            },
        }
    }

    // m maps from slots(x) -> slots(y).
    pub fn eq(&self, x: Id, y: Id, m: &SlotMap) -> bool {
        todo!()
    }
}
