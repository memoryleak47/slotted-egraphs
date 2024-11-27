use crate::*;

trait Annotation {
    fn apply_slotmap(m: &SlotMap) -> Self;
    fn compose(&self, other: &Self) -> Self;
    fn inverse(&self) -> Self;
    fn refl(x: AppliedId) -> Self;
}

pub struct SlottedUF<A: Annotation>(Vec<SUFClass<A>>);

struct SUFClass<A: Annotation> {
    leader: AppliedId,
    annotation_to_leader: A,

    s: Set<Slot>,
    g: Group<A>,
}

struct Group<A: Annotation>(A);
