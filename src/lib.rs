use std::collections::HashSet;

struct Unionfind {
    classes: Vec<Class>,
}

struct Class {
    leader: AppliedId,
    group: Group,
}

struct AppliedId {
    id: Id,
    m: SlotMap,
}

// SlotMap m maps x to m.0[x]
struct SlotMap(Box<[Slot]>);

struct Id(usize);

struct Group(HashSet<SlotMap>);

struct Slot(u32);
