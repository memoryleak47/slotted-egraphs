use crate::*;

trait Annotation {
    fn apply_slotmap(m: &SlotMap) -> Self;
    fn compose(&self, other: &Self) -> Self;
    fn inverse(&self) -> Self;
    fn refl(x: AppliedId) -> Self; // there's a ProofSystem::refl(AppliedId) -> ProvenEq function that doesn't require &self, because we don't store those proofs in the registry.
}

pub struct SlottedUF<A: Annotation>(Vec<SUFClass<A>>);

struct SUFClass<A: Annotation> {
    leader: AppliedId,
    annotation_to_leader: A,

    s: Set<Slot>,
    g: Group<A>,
}

struct Group<A: Annotation>(A);
