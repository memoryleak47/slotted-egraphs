use crate::*;

pub struct SlottedUF<C>(Vec<SUFClass<C>>);

struct SUFClass<C> {
    leader: AppliedId,
    s: HashSet<Slot>,
    g: Group<SlotMap>,

    c: C,
}
