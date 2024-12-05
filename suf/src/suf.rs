use crate::*;

struct Suf {
    v: Vec<Class>,
}

struct Class {
    leader: AppliedId,
    slots: HashSet<Slot>,
    group: Group,
}

