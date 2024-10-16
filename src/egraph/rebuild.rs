use crate::*;

impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    // proof.l should be i.
    // proof.r should be missing a few slots.
    fn record_redundancy_witness(&mut self, i: Id, cap: &HashSet<Slot>, proof: ProvenEq) {
        if CHECKS {
            assert!(self.is_alive(i));

            #[cfg(feature = "explanations")]
            assert_eq!(proof.l.id, i);
        }

        let prf = ghost!({
            let flipped = prove_symmetry(proof.clone(), &self.proof_registry);
            let new_prf = prove_transitivity(proof, flipped, &self.proof_registry);

            let old_prf = self.proven_find_applied_id(&self.mk_syn_identity_applied_id(i)).proof;
            prove_transitivity(new_prf, old_prf, &self.proof_registry)
        });

        let elem = self.mk_syn_identity_applied_id(i).apply_slotmap_partial(&SlotMap::identity(cap));

        #[cfg(feature = "explanations")]
        if CHECKS {
            let eq = prf.equ();
            let elem2 = eq.r.apply_slotmap_partial(&eq.l.m.inverse());
            assert_eq!(elem, elem2);
        }

        self.unionfind_set(i, ProvenAppliedId {
            elem,

            #[cfg(feature = "explanations")]
            proof: prf,
        });
    }

    // We expect `from` to be on the lhs of this equation.
    pub(crate) fn shrink_slots(&mut self, from: &AppliedId, cap: &HashSet<Slot>, proof: ProvenEq) {
        #[cfg(feature = "explanations")]
        if CHECKS {
            assert_eq!(from.id, proof.l.id);
        }

        let origcap = cap.iter().map(|x| from.m.inverse()[*x]).collect();
        self.record_redundancy_witness(from.id, &origcap, proof);

        let (id, cap) = {
            // from.m :: slots(from.id) -> X
            // cap :: set X

            // m_inv :: X -> slots(from.id)
            let m_inv = from.m.inverse();

            // cap :: set slots(from.id)
            let new_cap: HashSet<Slot> = cap.iter().map(|x| m_inv[*x]).collect();

            (from.id, new_cap)
        };

        // cap :: set slots(id)

        let syn_slots = &self.syn_slots(id);
        let c = self.classes.get_mut(&id).unwrap();
        let grp = &c.group;

        let mut final_cap = cap.clone();

        // d is a newly redundant slot.
        for d in &c.slots - &cap {
            // if d is redundant, then also the orbit of d is redundant.
            final_cap = &final_cap - &grp.orbit(d);
        }

        c.slots = cap.clone();
        let generators = c.group.generators();
        let _ = c;

        let restrict_proven = |proven_perm: ProvenPerm| {
            proven_perm.check();

            let perm = proven_perm.elem.into_iter()
                .filter(|(x, _)| cap.contains(x))
                .collect();

            #[cfg(feature = "explanations")]
            let prf = self.disassociate_proven_eq(proven_perm.proof);
            let out = ProvenPerm {
                elem: perm,
                #[cfg(feature = "explanations")]
                proof: prf,
                #[cfg(feature = "explanations")]
                reg: self.proof_registry.clone()
            };
            out.check();
            out
        };

        let generators = generators.into_iter().map(restrict_proven).collect();
        let identity = ProvenPerm::identity(id, &cap, syn_slots, self.proof_registry.clone());
        identity.check();
        let c = self.classes.get_mut(&id).unwrap();
        c.group = Group::new(&identity, generators);

        self.touched_class(from.id);
    }

    pub(crate) fn rebuild(&mut self) {
        if CHECKS { self.check(); }
        while let Some(sh) = self.pending.iter().cloned().next() {
            self.pending.remove(&sh);
            self.handle_pending(sh);

            if CHECKS { self.check(); }
        }
    }

    fn consider_pc(pc: &ProvenContains<L>) {
        println!("-------------------------------------------");
        dbg!(&pc.pai.elem);
        dbg!(&pc.node.elem);

        #[cfg(feature = "explanations")]
        dbg!(&pc.pai.proof.equ());
        #[cfg(feature = "explanations")]
        dbg!(&pc.node.proofs.iter().map(|x| x.equ()).collect::<Vec<_>>());
        println!("///////////////////////////////////////////");
    }

    fn handle_pending(&mut self, sh: L) {
        let i = self.hashcons[&sh];

        self.update_analysis(&sh, i);

        let psn = self.raw_remove_from_class(i, sh.clone());
        let pc = self.pc_from_psn((sh.clone(), psn.clone()));
        let mut pc = self.pc_find(&pc);

        while !pc.pai.elem.slots().is_subset(&pc.node.elem.slots()) {
            self.handle_shrink_in_upwards_merge(pc.clone());

            pc = self.pc_find(&pc);
        }

        // upwards merging found a match!
        let (sh, _) = pc.node.elem.weak_shape();
        if self.hashcons.contains_key(&sh) {
            self.handle_congruence(pc.clone());
            return;
        }

        let (sh, psn) = pc.weak_shape();
        self.raw_add_to_class(pc.target_id(), (sh, psn));

        self.determine_self_symmetries(pc.clone());
    }

    fn update_analysis(&mut self, sh: &L, i: Id) {
        let v = N::make(self, sh);

        let c = self.classes.get_mut(&i).unwrap();
        let old = c.analysis_data.clone();
        let new = N::merge(old.clone(), v);
        c.analysis_data = new.clone();

        if new != old {
            self.touched_class(i);
        }
    }

    fn handle_shrink_in_upwards_merge(&mut self, pc1: ProvenContains<L>) {
        let pc2 = self.chain_pc_map(&pc1, |_, pai| self.proven_proven_find_applied_id(&pai));

        let (a, b, prf) = self.pc_congruence(&pc1, &pc2);

        let cap = &a.slots() & &b.slots();

        self.shrink_slots(&a, &cap, prf);
    }

    // finds self-symmetries caused by the e-node `src_id`.
    fn determine_self_symmetries(&mut self, pc1: ProvenContains<L>) {
        let i = pc1.target_id();
        let weak = pc1.node.elem.weak_shape().0;
        for pn2 in self.proven_proven_get_group_compatible_variants(&pc1.node) {
            let pc2 = ProvenContains {
                pai: pc1.pai.clone(),
                node: pn2,
            };
            let (weak2, bij2) = pc2.node.elem.weak_shape();
            if weak == weak2 {
                self.check_pc(&pc1);
                self.check_pc(&pc2);
                assert_eq!(pc1.target_id(), pc2.target_id());
                let (a, b, proof) = self.pc_congruence(&pc1, &pc2);

                // or is it the opposite direction? (flip a with b)
                let perm = a.m.compose(&b.m.inverse());

                let proven_perm = ProvenPerm {
                    elem: perm,

                    #[cfg(feature = "explanations")]
                    proof,

                    #[cfg(feature = "explanations")]
                    reg: self.proof_registry.clone(),
                };

                if CHECKS {
                    proven_perm.check();
                }
                let grp = &mut self.classes.get_mut(&i).unwrap().group;
                grp.add(proven_perm);
            }
        }
    }

    pub(in crate::egraph) fn handle_congruence(&mut self, pc1: ProvenContains<L>) {
        let (sh, _) = self.shape(&pc1.node.elem);
        let pc2 = self.pc_from_shape(&sh);

        let (a, b, prf) = self.pc_congruence(&pc1, &pc2);
        self.union_internal(&a, &b, prf);
    }

    // upon touching an e-class, you need to update all usages of it.
    pub(crate) fn touched_class(&mut self, i: Id) {
        for sh in &self.classes[&i].usages {
            self.pending.insert(sh.clone());
        }
    }

    pub(crate) fn pc_from_shape(&self, sh: &L) -> ProvenContains<L> {
        let i = self.hashcons.get(&sh).expect("pc_from_shape should only be called if the shape exists in the e-graph!");
        let psn = self.classes[&i].nodes[&sh].clone();

        self.pc_from_psn((sh.clone(), psn))
    }
}
