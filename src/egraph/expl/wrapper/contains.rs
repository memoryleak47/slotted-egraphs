use crate::*;

// the existance of a ProvenContains `pc` implies `eg.find_id(eg.lookup(pc.elem.elem).unwrap().id) == eg.find_id(pc.proof.r.id)`
pub struct ProvenContains<L> {
    // contains the proof that the e-node is equal to some app-ids syn-enode.
    // lhs of this proof is the syn-enode, rhs is the current e-node represented by this ProvenContains when explanations are off.
    pub elem: ProvenNode<L>,

    // proofs that this app-id is equal to our target app-id.
    // The lhs of this ProvenEq should be the class containing our syn-enode (i.e. the source-id).
    // The rhs is the current state that we express (i.e. the target-id).
    #[cfg(feature = "explanations")]
    pub proof: ProvenEq,
}

impl<L: Language> ProvenContains<L> {
    #[cfg(feature = "explanations")]
    fn src_id(&self) -> Id {
        self.proof.l.id
    }

    #[cfg(feature = "explanations")]
    fn target_id(&self) -> Id {
        self.proof.r.id
    }
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

    #[cfg(feature = "explanations")]
    pub fn pc_congruence(&self, a: &ProvenContains<L>, b: &ProvenContains<L>) -> ProvenEq {
        let prf_a = &*a.elem.proofs;
        let prf_b = &*b.elem.proofs;

        assert_eq!(prf_a.len(), prf_b.len());
        let n = prf_a.len();

        let mut vec = Vec::new();
        for (pa, pb) in prf_a.iter().zip(prf_b.iter()) {
            let pb_inv = self.prove_symmetry(pb.clone());
            let pa_to_pb = self.prove_transitivity(pa.clone(), pb_inv);
            vec.push(pa_to_pb);
        }

        // a.src -> b.src
        let prf = self.prove_congruence(a.src_id(), b.src_id(), &vec);

        // a.proof :: a.src -> a.target
        // b.proof :: b.src -> b.target

        let sym_a = prove_symmetry(a.proof.clone(), &self.proof_registry);
        let prf = prove_transitivity(sym_a, prf, &self.proof_registry);
        let prf = prove_transitivity(prf, b.proof.clone(), &self.proof_registry);

        // a.target -> b.target
        prf
    }
}
