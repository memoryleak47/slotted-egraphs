use crate::*;

// src_id: left.
// target_id: right

// the existance of a ProvenContains `pc` implies that `pc.node` is contained in the e-class `pai` (assuming we are in non-ghost mode).
#[derive(Clone, Debug)]
pub struct ProvenContains<L> {
    // contains the proof that the e-node is equal to some app-ids syn-enode.
    // lhs of this proof is the syn-enode, rhs is the current e-node represented by this ProvenContains when explanations are off.
    pub node: ProvenNode<L>,

    // proofs that this app-id is equal to our target app-id.
    // The lhs of this ProvenEq should be the class containing our syn-enode (i.e. the source-id).
    // The rhs is the current state that we express (i.e. the target-id).
    pub pai: ProvenAppliedId,
}

impl<L: Language> ProvenContains<L> {
    #[cfg(feature = "explanations")]
    pub fn src_id(&self) -> Id {
        self.pai.proof.l.id
    }

    pub fn target_id(&self) -> Id {
        self.pai.elem.id
    }
}

impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    pub fn check_pc(&self, pc: &ProvenContains<L>) {
        self.check_pai(&pc.pai);
        self.check_pn(&pc.node);

        #[cfg(feature = "explanations")]
        {
            let a: Vec<Id> = self.get_syn_node(&self.mk_syn_identity_applied_id(pc.pai.proof.l.id)).applied_id_occurences().iter().map(|x| x.id).collect();
            let b: Vec<Id> = pc.node.proofs.iter().map(|eq| eq.l.id).collect();
            assert_eq!(a, b);
        }
    }

    pub fn pc_from_src_id(&self, i: Id) -> ProvenContains<L> {
        let identity = self.mk_syn_identity_applied_id(i);
        let n = self.get_syn_node(&identity);
        let (sh, bij) = self.proven_shape(&n);

        let pai = self.proven_find_applied_id(&identity);

        let out = ProvenContains {
            node: ProvenNode {
                elem: sh.elem.apply_slotmap_fresh(&bij),

                #[cfg(feature = "explanations")]
                proofs: sh.proofs,
            },
            pai,
        };
        self.check_pc(&out);
        out
    }

    pub fn chain_pc_map(&self, start: &ProvenContains<L>, f: impl Fn(usize, ProvenAppliedId) -> ProvenAppliedId) -> ProvenContains<L> {
        let out = ProvenContains {
            node: self.chain_pn_map(&start.node, f),
            pai: start.pai.clone(),
        };
        self.check_pc(&out);
        out
    }

    pub fn chain_pc_eq(&self, start: &ProvenContains<L>, eq: ProvenEq) -> ProvenContains<L> {
        ProvenContains {
            node: start.node.clone(),
            pai: self.chain_pai_eq(&start.pai, eq),
        }
    }

    // After this fn, both PCs talk about the same e-node.
    // panics, if that's impossible.
    fn match_pcs(&self, a: &ProvenContains<L>, b: &ProvenContains<L>) -> (ProvenContains<L>, ProvenContains<L>) {
        let (sh1, bij1) = a.node.elem.weak_shape();
        let (sh2, bij2) = b.node.elem.weak_shape();
        assert_eq!(&sh1, &sh2);

        // bij1 :: SHAPE -> A
        // bij2 :: SHAPE -> B

        // m :: B -> A
        let m = bij2.inverse().compose_fresh(&bij1);

        // update b to be compatible with a.
        let b = ProvenContains {
            node: ProvenNode {
                elem: b.node.elem.apply_slotmap_fresh(&m),

                #[cfg(feature = "explanations")]
                proofs: b.node.proofs.clone(),
            },
            pai: ProvenAppliedId {
                elem: b.pai.elem.apply_slotmap_fresh(&m),

                #[cfg(feature = "explanations")]
                proof: b.pai.proof.clone(),
            },
        };

        (a.clone(), b)
    }

    pub fn pc_congruence(&self, a: &ProvenContains<L>, b: &ProvenContains<L>) -> (AppliedId, AppliedId, ProvenEq) {
        self.check_pc(a);
        self.check_pc(b);

        let (a, b) = self.match_pcs(a, b);

        self.check_pc(&a);
        self.check_pc(&b);

        let prf = ghost!({
            let prf_a = &*a.node.proofs;
            let prf_b = &*b.node.proofs;

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

            let sym_a = prove_symmetry(a.pai.proof.clone(), &self.proof_registry);
            let prf = prove_transitivity(sym_a, prf, &self.proof_registry);
            let prf = prove_transitivity(prf, b.pai.proof.clone(), &self.proof_registry);
            prf
        });

        // a.target -> b.target
        (a.pai.elem.clone(), b.pai.elem.clone(), prf)
    }
}
