use crate::*;

#[derive(Clone, Debug)]
pub struct ProvenAppliedId {
    pub elem: AppliedId,

    // @ghost
    pub proof: ProvenEq,
}

impl<L: Language> EGraph<L> {
    // x=y & y=z make x=z
    // It will return "next.elem" but using the slots of "start". The proofs concatenate.
    // Assumes that "next.m.values() ~ slots(start.id)"
    pub fn chain_pai(&self, start: &ProvenAppliedId, next: &ProvenAppliedId) -> ProvenAppliedId {
        ProvenAppliedId {
            elem: next.elem.apply_slotmap(&start.elem.m),
            // @ghost
            proof: prove_transitivity(start.proof.clone(), next.proof.clone(), &self.proof_registry),
        }
    }
}
