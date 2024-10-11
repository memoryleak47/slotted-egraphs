use crate::*;

// the existance of a ProvenContains `pc` implies `eg.find_id(eg.lookup(pc.elem.elem).unwrap().id) == eg.find_id(pc.proof.r.id)`
pub struct ProvenContains<L> {
    // contains the proof that the e-node is equal to some app-ids syn-enode.
    // lhs of this proof is the syn-enode, rhs is the current e-node represented by this ProvenContains when explanations are off.
    pub elem: ProvenNode<L>,

    // proofs that this app-id is equal to our target app-id.
    // The lhs of this ProvenEq should be the class containing our syn-enode.
    // The rhs is the current state that we express.
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
