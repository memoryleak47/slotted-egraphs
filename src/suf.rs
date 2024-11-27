use crate::*;

pub struct SlottedUF(Vec<SUFClass>);

struct SUFClass {
    leader: AppliedId,
    s: Set<Slot>,
    g: Group,
}

struct AppliedId(SlotMap, Id);

struct SlotMap(Vec<(Slot, Slot)>);

struct Group;

struct Slot(usize);
struct Id(usize);
