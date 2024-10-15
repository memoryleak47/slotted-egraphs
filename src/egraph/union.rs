use crate::*;

impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    pub fn union(&mut self, l: &AppliedId, r: &AppliedId) -> bool {
        self.union_justified(l, r, None)
    }

    pub fn union_justified(&mut self, l: &AppliedId, r: &AppliedId, j: Option<String>) -> bool {
        let subst = [(String::from("a"), l.clone()),
                     (String::from("b"), r.clone())]
                        .into_iter().collect();
        let a = Pattern::parse("?a").unwrap();
        let b = Pattern::parse("?b").unwrap();

        self.union_instantiations(&a, &b, &subst, j)
    }

    pub fn union_instantiations(&mut self, from_pat: &Pattern<L>, to_pat: &Pattern<L>, subst: &Subst, justification: Option<String>) -> bool {
        let a = pattern_subst(self, from_pat, subst);
        let b = pattern_subst(self, to_pat, subst);

        let syn_a = self.synify_app_id(a.clone());
        let syn_b = self.synify_app_id(b.clone());

        let proof = ghost!(self.prove_explicit(&syn_a, &syn_b, justification));

        let out = self.union_internal(&a, &b, proof);
        self.rebuild();
        out
    }

    pub(in crate::egraph) fn union_internal(&mut self, l: &AppliedId, r: &AppliedId, proof: ProvenEq) -> bool {
        // normalize inputs
        let pai_l = self.proven_find_applied_id(&l);
        let pai_r = self.proven_find_applied_id(&r);

        let proof = ghost!({
            if CHECKS {
                pai_l.proof.check(self);
                pai_r.proof.check(self);
            }

            let a = self.prove_symmetry(pai_l.proof);
            let a = self.prove_transitivity(a, proof);
            let a = self.prove_transitivity(a, pai_r.proof);
            if CHECKS {
                assert_eq!(a.l.id, pai_l.elem.id);
                assert_eq!(a.r.id, pai_r.elem.id);
            }
            a
        });
        self.union_leaders(pai_l.elem, pai_r.elem, proof)
    }

    fn union_leaders(&mut self, l: AppliedId, r: AppliedId, proof: ProvenEq) -> bool {
        // early return, if union should not be made.
        if self.eq(&l, &r) { return false; }

        let cap = &l.slots() & &r.slots();

        if l.slots() != cap {
            self.shrink_slots(&l, &cap, proof.clone());
            self.union_internal(&l, &r, proof);
            return true;
        }

        if r.slots() != cap {
            let flipped_proof = ghost!(self.prove_symmetry(proof.clone()));
            self.shrink_slots(&r, &cap, flipped_proof);
            self.union_internal(&l, &r, proof);
            return true;
        }

        if l.id == r.id {
            let id = l.id;

            // l.m :: slots(id) -> X
            // r.m :: slots(id) -> X
            // perm :: slots(id) -> slots(id)
            let perm = l.m.compose(&r.m.inverse());
            if CHECKS {
                assert!(perm.is_perm());
                assert_eq!(&perm.keys(), &self.classes[&id].slots);
            }

            let proven_perm = ProvenPerm {
                elem: perm,
                #[cfg(feature = "explanations")]
                proof,
                #[cfg(feature = "explanations")]
                reg: self.proof_registry.clone()
            };
            #[cfg(feature = "explanations")]
            assert_eq!(proven_perm.proof.l.id, id);

            proven_perm.check();
            let grp = &mut self.classes.get_mut(&id).unwrap().group;
            if grp.contains(&proven_perm.to_slotmap()) { return false; }

            grp.add(proven_perm);

            self.touched_class(id);

            true
        } else {
            // sort, s.t. size(l) >= size(r).
            let size = |i| {
                let c = &self.classes[&i];
                c.nodes.len() + c.usages.len()
            };

            if size(l.id) < size(r.id) {
                self.move_to(&l, &r, proof)
            } else {
                let proof = ghost!(self.prove_symmetry(proof));
                self.move_to(&r, &l, proof)
            }

            true
        }
    }

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
    fn shrink_slots(&mut self, from: &AppliedId, cap: &HashSet<Slot>, proof: ProvenEq) {
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

    fn assert_ty(&self, m: &SlotMap, keys: &HashSet<Slot>, values: &HashSet<Slot>) {
        if CHECKS {
            assert!(m.keys().is_subset(&keys));
            assert!(m.values().is_subset(&values));
        }
    }

    // moves everything from `from` to `to`.
    fn move_to(&mut self, from: &AppliedId, to: &AppliedId, proof: ProvenEq) {
        if CHECKS {
            assert_eq!(from.slots(), to.slots());
            #[cfg(feature = "explanations")]
            assert_eq!(from.id, proof.l.id);
            #[cfg(feature = "explanations")]
            assert_eq!(to.id, proof.r.id);
        }
        // from.m :: slots(from.id) -> X
        // to.m :: slots(to.id) -> X
        let map = to.m.compose_partial(&from.m.inverse());
        self.assert_ty(&map, &self.slots(to.id), &self.slots(from.id));

        let app_id = self.mk_sem_applied_id(to.id, map.clone());
        let pai = ProvenAppliedId {
            elem: app_id,

            #[cfg(feature = "explanations")]
            proof,
        };
        self.unionfind_set(from.id, pai);

        // who updates the usages? raw_add_to_class & raw_remove_from_class do that.

        let from_nodes = self.classes.get(&from.id).unwrap().nodes.clone();
        let from_id = self.mk_sem_identity_applied_id(from.id);
        for (sh, psn) in from_nodes {
            self.raw_remove_from_class(from.id, (sh.clone(), psn.elem.clone()));
            // if `sh` contains redundant slots, these won't be covered by 'map'.
            // Thus we need compose_fresh.
            let new_bij = psn.elem.compose_fresh(&map.inverse());

            let src_id = psn.src_id;

            self.raw_add_to_class(to.id, (sh.clone(), new_bij), src_id);
            self.pending.insert(sh);
        }

        // re-add the group equations as well.

        // This basically calls self.union(from, from * perm) for each perm generator in the group of from.
        // from.m :: slots(from.id) -> C
        // to.m :: slots(to.id) -> C

        // f :: slots(from.id) -> slots(to.id)
        // Note that f is a partial map, because some slots might have become redundant.
        let f = from.m.compose_partial(&to.m.inverse());

        let change_permutation_from_from_to_to = |x: Perm| -> Perm {
            let perm: Perm = x.iter().filter_map(|(x, y)| {
                if f.contains_key(x) && f.contains_key(y) {
                    Some((f[x], f[y]))
                } else { None }
            }).collect();

            if CHECKS {
                assert!(perm.is_perm());
                assert_eq!(perm.keys(), self.classes[&to.id].slots);
            }

            perm
        };
        #[cfg(feature = "explanations")]
        let prf = self.proven_find_applied_id(&from).proof;
        #[cfg(feature = "explanations")]
        let prf_rev = self.prove_symmetry(prf.clone());

        let change_proven_permutation_from_from_to_to = |proven_perm: ProvenPerm| {
            let new_perm = change_permutation_from_from_to_to(proven_perm.elem);
            #[cfg(feature = "explanations")]
            let new_proof = self.prove_transitivity(prf_rev.clone(), self.prove_transitivity(proven_perm.proof, prf.clone()));
            ProvenPerm {
                elem: new_perm,
                #[cfg(feature = "explanations")]
                proof: new_proof,
                #[cfg(feature = "explanations")]
                reg: self.proof_registry.clone(),
            }
        };

        let set = self.classes[&from.id].group.generators()
            .into_iter()
            .map(change_proven_permutation_from_from_to_to)
            .collect();
        self.classes.get_mut(&to.id).unwrap().group.add_set(set);

        // touched because the group might have grown.
        self.touched_class(to.id);

        // touched because the class is now dead and no e-nodes should point to it.
        self.touched_class(from.id);
    }

    pub fn rebuild(&mut self) {
        if CHECKS { self.check(); }
        while let Some(sh) = self.pending.iter().cloned().next() {
            self.pending.remove(&sh);
            self.handle_pending(sh);

            if CHECKS { self.check(); }
        }
    }

    fn handle_pending(&mut self, sh: L) {
        let i = self.hashcons[&sh];

        self.update_analysis(&sh, i);

        let psn = self.classes[&i].nodes[&sh].clone();
        let node = sh.apply_slotmap(&psn.elem);
        self.raw_remove_from_class(i, (sh.clone(), psn.elem));
        let app_i = self.mk_sem_identity_applied_id(i);

        let src_id = psn.src_id;
        self.semantic_add(&node, &app_i, src_id);
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

    fn handle_shrink_in_upwards_merge(&mut self, src_id: Id) {
        let pc1 = self.pc_from_src_id(src_id);
        let pc2 = self.chain_pc_map(&pc1, |_, pai| self.proven_proven_find_applied_id(&pai));

        let (a, b, prf) = self.pc_congruence(&pc1, &pc2);

        let cap = &a.slots() & &b.slots();

        self.shrink_slots(&a, &cap, prf);
    }

    // TODO get rid of semantic_add, in favor of "handle_pending".
    pub fn semantic_add(&mut self, enode: &L, i_orig: &AppliedId, src_id: Id) {
        let mut enode = self.find_enode(&enode);
        let mut i = self.find_applied_id(i_orig);
        // i.m :: slots(i) -> X
        // i_orig.m :: slots(i_orig) -> X
        let theta = i_orig.m.compose(&i.m.inverse());
        if !i.slots().is_subset(&enode.slots()) {
            self.handle_shrink_in_upwards_merge(src_id);

            enode = self.find_enode(&enode);
            i = self.find_applied_id(&i);
        }

        let t = self.shape(&enode);

        // upwards merging found a match!
        if self.lookup_internal(&t).is_some() {
            self.handle_congruence(src_id);
            return;
        }

        let (sh, bij) = t;
        let mut m = i.m.inverse();

        for x in bij.values() {
            if !m.contains_key(x) {
                m.insert(x, Slot::fresh());
            }
        }
        let bij = bij.compose(&m);
        let t = (sh, bij);
        self.raw_add_to_class(i.id, t.clone(), src_id);

        self.determine_self_symmetries(src_id);
    }

    // finds self-symmetries caused by the e-node `src_id`.
    fn determine_self_symmetries(&mut self, src_id: Id) {
        let pc1 = self.pc_from_src_id(src_id);

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

    pub(in crate::egraph) fn handle_congruence(&mut self, a: Id) {
        let pc1 = self.pc_from_src_id(a);

        let a_identity = self.mk_syn_identity_applied_id(a);
        let a_node = self.get_syn_node(&a_identity);
        let (pn1, bij1) = self.proven_shape(&a_node);
        let t = (pn1.elem.clone(), bij1);
        let b = self.lookup_internal(&t).expect("One should only call handle_congruence if there is a hashcons collision!").id;
        let b = self.classes[&b].nodes[&pn1.elem].src_id;
        let pc2 = self.pc_from_src_id(b);

        let (a, b, prf) = self.pc_congruence(&pc1, &pc2);
        self.union_internal(&a, &b, prf);
    }

    // upon touching an e-class, you need to update all usages of it.
    fn touched_class(&mut self, i: Id) {
        for sh in &self.classes[&i].usages {
            self.pending.insert(sh.clone());
        }
    }
}
