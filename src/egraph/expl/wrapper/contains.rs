use crate::*;

pub struct ProvenContains<L> {
    // contains the proof that the e-node is equal to some app-ids syn-enode.
    pub elem: ProvenNode<L>,

    // proofs that this app-id is equal to our target app-id.
    #[cfg(feature = "explanations")]
    pub proof: ProvenEq,
}

impl<L: Language> EGraph<L> {
    pub fn refl_pc(&self, i: Id) -> ProvenContains<L> {
        let id = self.mk_syn_identity_applied_id(i);
        let node = self.get_syn_node(&id);
        ProvenContains {
            elem: self.refl_pn(&node),
            #[cfg(feature = "explanations")]
            proof: prove_reflexivity(&id, &self.proof_registry),
        }
    }

    pub fn chain_pc_map(&self, start: &ProvenContains<L>, f: impl Fn(usize, ProvenAppliedId) -> ProvenAppliedId) -> ProvenContains<L> {
        ProvenContains {
            elem: self.chain_pn_map(&start.elem, f),
            #[cfg(feature = "explanations")]
            proof: start.proof.clone(),
        }
    }

    pub fn chain_pc_eq(&self, start: &ProvenContains<L>, eq: ProvenEq) -> ProvenContains<L> {
        ProvenContains {
            elem: start.elem.clone(),
            #[cfg(feature = "explanations")]
            proof: prove_transitivity(start.proof.clone(), eq.clone(), &self.proof_registry),
        }
    }

    pub fn pc_congruence(&self, a: &ProvenContains<L>, b: &ProvenContains<L>) -> ProvenEq {
        todo!()
    }
}
