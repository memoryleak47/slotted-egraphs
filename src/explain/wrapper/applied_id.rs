use crate::*;

#[derive(Clone, Debug)]
pub(crate) struct ProvenAppliedId {
    pub elem: AppliedId,

    #[cfg(feature = "explanations")]
    pub proof: ProvenEq,
}

impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    pub(crate) fn check_pai(&self, pai: &ProvenAppliedId) {
        #[cfg(feature = "explanations")]
        {
            assert_eq!(pai.proof.r.id, pai.elem.id);
            self.check_syn_applied_id(&pai.proof.l);
            self.check_syn_applied_id(&pai.proof.r);
        }
    }

    // x=y & y=z make x=z
    // It will return "next.elem" but using the slots of "start". The proofs concatenate.
    // Assumes that "next.m.values() ~ slots(start.id)"
    pub(crate) fn chain_pai(&self, start: &ProvenAppliedId, next: &ProvenAppliedId) -> ProvenAppliedId {
        ProvenAppliedId {
            elem: next.elem.apply_slotmap(&start.elem.m),

            #[cfg(feature = "explanations")]
            proof: prove_transitivity(start.proof.clone(), next.proof.clone(), &self.proof_registry),
        }
    }

    pub(crate) fn refl_pai(&self, app_id: &AppliedId) -> ProvenAppliedId {
        ProvenAppliedId {
            elem: app_id.clone(),

            #[cfg(feature = "explanations")]
            proof: {
                // should this already be synified before calling this?
                let syn = self.synify_app_id(app_id.clone());
                prove_reflexivity(&syn, &self.proof_registry)
            }
        }
    }

    pub(crate) fn chain_pai_pp(&self, pai: &ProvenAppliedId, pp: &ProvenPerm) -> ProvenAppliedId {
        ProvenAppliedId {
            elem: self.mk_sem_applied_id(pai.elem.id, pp.elem.compose(&pai.elem.m)),

            #[cfg(feature = "explanations")]
            proof: self.prove_transitivity(pai.proof.clone(), pp.proof.clone()),
        }
    }

    // doesn't do anything if explanations are off.
    pub(crate) fn chain_pai_eq(&self, pai: &ProvenAppliedId, peq: ProvenEq) -> ProvenAppliedId {
        ProvenAppliedId {
            elem: pai.elem.clone(),

            #[cfg(feature = "explanations")]
            proof: self.prove_transitivity(pai.proof.clone(), peq),
        }
    }
}
