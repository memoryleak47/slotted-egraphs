use crate::*;

pub struct SlottedUF(Vec<SUFClass>);

struct SUFClass {
    leader: AppliedId,
    s: Set<Slot>,
    g: Group,
}

struct Group;
