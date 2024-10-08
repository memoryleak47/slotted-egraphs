use crate::*;

#[derive(Clone, Debug)]
pub struct ProvenAppliedId {
    pub elem: AppliedId,

    #[cfg(feature = "explanations")]
    pub proof: ProvenEq,
}

impl<L: Language> EGraph<L> {
    // x=y & y=z make x=z
    // It will return "next.elem" but using the slots of "start". The proofs concatenate.
    // Assumes that "next.m.values() ~ slots(start.id)"
    pub fn chain_pai(&self, start: &ProvenAppliedId, next: &ProvenAppliedId) -> ProvenAppliedId {
        ProvenAppliedId {
            elem: next.elem.apply_slotmap(&start.elem.m),

            #[cfg(feature = "explanations")]
            proof: prove_transitivity(start.proof.clone(), next.proof.clone(), &self.proof_registry),
        }
    }

    pub fn refl_pai(&self, app_id: &AppliedId) -> ProvenAppliedId {
        ProvenAppliedId {
            elem: app_id.clone(),

            #[cfg(feature = "explanations")]
            proof: prove_reflexivity(app_id, &self.proof_registry),
        }
    }

    pub fn chain_pai_pp(&self, pai: &ProvenAppliedId, pp: &ProvenPerm) -> ProvenAppliedId {
        ProvenAppliedId {
            elem: self.mk_sem_applied_id(pai.elem.id, pp.elem.compose(&pai.elem.m)),

            #[cfg(feature = "explanations")]
            proof: self.prove_transitivity(pai.proof.clone(), pp.proof.clone()),
        }
    }
}
