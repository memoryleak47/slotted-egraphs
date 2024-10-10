use crate::*;

impl<L: Language> EGraph<L> {
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
                #[cfg(feature = "explanations_tmp")]
                proof,
                #[cfg(feature = "explanations_tmp")]
                reg: self.proof_registry.clone()
            };
            #[cfg(feature = "explanations_tmp")]
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

            #[cfg(feature = "explanations_tmp")]
            assert_eq!(proof.l.id, i);
        }

        let prf = ghost!({
            let flipped = prove_symmetry(proof.clone(), &self.proof_registry);
            let new_prf = prove_transitivity(proof, flipped, &self.proof_registry);

            let old_prf = self.proven_find_applied_id(&self.mk_syn_identity_applied_id(i)).proof;
            prove_transitivity(new_prf, old_prf, &self.proof_registry)
        });

        let elem = self.mk_syn_identity_applied_id(i).apply_slotmap_partial(&SlotMap::identity(cap));

        #[cfg(feature = "explanations_tmp")]
        if CHECKS {
            let eq = prf.equ();
            let elem2 = eq.r.apply_slotmap_partial(&eq.l.m.inverse());
            assert_eq!(elem, elem2);
        }

        self.unionfind_set(i, ProvenAppliedId {
            elem,

            #[cfg(feature = "explanations_tmp")]
            proof: prf,
        });
    }

    // We expect `from` to be on the lhs of this equation.
    fn shrink_slots(&mut self, from: &AppliedId, cap: &HashSet<Slot>, proof: ProvenEq) {
        #[cfg(feature = "explanations_tmp")]
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

            #[cfg(feature = "explanations_tmp")]
            let prf = self.disassociate_proven_eq(proven_perm.proof);
            let out = ProvenPerm {
                elem: perm,
                #[cfg(feature = "explanations_tmp")]
                proof: prf,
                #[cfg(feature = "explanations_tmp")]
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
            #[cfg(feature = "explanations_tmp")]
            assert_eq!(from.id, proof.l.id);
            #[cfg(feature = "explanations_tmp")]
            assert_eq!(to.id, proof.r.id);
        }
        // from.m :: slots(from.id) -> X
        // to.m :: slots(to.id) -> X
        let map = to.m.compose_partial(&from.m.inverse());
        self.assert_ty(&map, &self.slots(to.id), &self.slots(from.id));

        let app_id = self.mk_sem_applied_id(to.id, map.clone());
        let pai = ProvenAppliedId {
            elem: app_id,

            #[cfg(feature = "explanations_tmp")]
            proof,
        };
        self.unionfind_set(from.id, pai);

        // who updates the usages? raw_add_to_class & raw_remove_from_class do that.

        let from_nodes = self.classes.get(&from.id).unwrap().nodes.clone();
        let from_id = self.mk_sem_identity_applied_id(from.id);
        for (sh, psn) in from_nodes {
            let enode = sh.apply_slotmap(&psn.elem);
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
        #[cfg(feature = "explanations_tmp")]
        let prf = self.proven_find_applied_id(&from).proof;
        #[cfg(feature = "explanations_tmp")]
        let prf_rev = self.prove_symmetry(prf.clone());

        let change_proven_permutation_from_from_to_to = |proven_perm: ProvenPerm| {
            let new_perm = change_permutation_from_from_to_to(proven_perm.elem);
            #[cfg(feature = "explanations_tmp")]
            let new_proof = self.prove_transitivity(prf_rev.clone(), self.prove_transitivity(proven_perm.proof, prf.clone()));
            ProvenPerm {
                elem: new_perm,
                #[cfg(feature = "explanations_tmp")]
                proof: new_proof,
                #[cfg(feature = "explanations_tmp")]
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
        let psn = self.classes[&i].nodes[&sh].clone();
        let node = sh.apply_slotmap(&psn.elem);
        self.raw_remove_from_class(i, (sh.clone(), psn.elem));
        let app_i = self.mk_sem_identity_applied_id(i);

        let src_id = psn.src_id;
        self.semantic_add(&node, &app_i, src_id);
    }

    fn handle_shrink_in_upwards_merge(&mut self, src_id: Id) {
        let pai = self.proven_unionfind_get(src_id);
        let ProvenAppliedId { elem: leader, .. } = pai.clone();

        #[cfg(feature = "explanations_tmp")]
        let neg_leader_prf = prove_symmetry(pai.proof.clone(), &self.proof_registry);

        let src_syn_slots = self.syn_slots(src_id);

        let identity = self.mk_syn_identity_applied_id(src_id);
        let syn_enode = self.get_syn_node(&identity);
        assert!(syn_enode.slots().is_subset(&src_syn_slots));
        let pn = self.proven_find_enode(&syn_enode);
        let ProvenNode { elem: new_node, .. } = pn.clone();
        assert!(new_node.slots().is_subset(&src_syn_slots));

        #[cfg(feature = "explanations_tmp")]
        let mut combined = Vec::new();
        #[cfg(feature = "explanations_tmp")]
        for (app_id, prf) in new_node.applied_id_occurences().into_iter().zip(pn.proofs.into_iter()) {
            // each child-proof might "fix" a few slots, which are not witnessed to be redundant by it.
            let rev = prove_symmetry(prf.clone(), &self.proof_registry);
            let cycle = prove_transitivity(prf, rev, &self.proof_registry);

            combined.push(cycle);
        }
        #[cfg(feature = "explanations_tmp")]
        let cong = self.prove_congruence(src_id, src_id, &combined);
        #[cfg(feature = "explanations_tmp")]
        let prf = self.prove_transitivity(neg_leader_prf.clone(), self.prove_transitivity(cong, pai.proof.clone()));

        let leader_inv = leader.m.inverse();
        if CHECKS {
            let ty = self.syn_slots(src_id);
            assert!(leader_inv.keys().is_subset(&ty));
        }
        let cap = new_node.slots();

        #[cfg(feature = "explanations_tmp")]
        if CHECKS {
            let ghost_cap: HashSet<_> = prf.l.m.iter()
                 .filter_map(|(x, y)| {
                    if prf.r.m.values().contains(&y) { Some(x) } else { None }
                 }).collect();
            assert_eq!(ghost_cap, cap);
        }
        self.shrink_slots(&leader, &cap, ghost!(prf));
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
        let pai = self.proven_unionfind_get(src_id);
        let ProvenAppliedId { elem: leader, .. } = pai.clone();
        #[cfg(feature = "explanations_tmp")]
        let neg_leader_prf = self.prove_symmetry(pai.proof.clone());
        let i = leader.id;
        let leader_bij = leader.m;

        let syn_slots = self.syn_slots(src_id);
        let src_identity = AppliedId::new(src_id, Perm::identity(&syn_slots));
        let syn_node = self.get_syn_node(&src_identity);
        if CHECKS {
            assert_eq!(&syn_slots, &syn_node.slots());
        }

        let pn1 = self.proven_find_enode(&syn_node);
        let ProvenNode { elem: enode, .. } = pn1.clone();
        let (weak, bij) = enode.weak_shape();
        for pn2  in self.proven_get_group_compatible_variants(&enode) {
            let ProvenNode { elem: n, .. } = pn2.clone();
            let (weak2, bij2) = n.weak_shape();
            if weak == weak2 {
                // I'm looking for an equation like i == i * BIJ to add BIJ to the group.

                // - i == sh * bij == enode == n
                // - sh2 * bij2 == n
                // - sh2 == sh

                // simplify:
                // - i == sh * bij
                // - i == sh * bij2

                // -> i == i * bij^-1 * bij2

                let perm = bij.inverse().compose(&bij2);

                // no need to add the identity permutation.
                if perm.iter().all(|(x, y)| x == y) { continue; }

                if CHECKS { assert!(perm.is_perm()); }

                #[cfg(feature = "explanations_tmp")]
                let mut combined_prfs = Vec::new();
                #[cfg(feature = "explanations_tmp")]
                for (old_to_new_ids, perm_prf) in pn1.proofs.iter().zip(pn2.proofs.iter()) {
                    let new_to_old_ids = self.prove_symmetry(old_to_new_ids.clone());

                    let eq = self.prove_transitivity(self.prove_transitivity(old_to_new_ids.clone(), perm_prf.clone()), new_to_old_ids.clone());
                    let combined = TransitivityProof(self.prove_transitivity(old_to_new_ids.clone(), perm_prf.clone()), new_to_old_ids.clone()).check(&eq, &self.proof_registry);
                    combined_prfs.push(combined);
                }

                // src_id[...] == src_id[...]

                #[cfg(feature = "explanations_tmp")]
                let prf = self.prove_congruence(src_id, src_id, &combined_prfs);
                #[cfg(feature = "explanations_tmp")]
                if CHECKS {
                    assert_eq!(prf.l.id, src_id);
                    assert_eq!(prf.r.id, src_id);
                }

                // i[...] == i[...]
                #[cfg(feature = "explanations_tmp")]
                let prf = self.prove_transitivity(neg_leader_prf.clone(), self.prove_transitivity(prf, pai.proof.clone()));
                let perm = leader_bij.compose_partial(&perm.compose_partial(&leader_bij.inverse()));

                let slots = self.slots(i);
                let syn_slots = self.syn_slots(i);
                #[cfg(feature = "explanations_tmp")]
                if CHECKS {
                    assert_eq!(prf.l.id, i);
                    assert_eq!(prf.r.id, i);
                }
                let proven_perm = ProvenPerm {
                    elem: perm,
                    #[cfg(feature = "explanations_tmp")]
                    proof: prf,
                    #[cfg(feature = "explanations_tmp")]
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
        let a = &self.mk_syn_identity_applied_id(a);
        let a_node = self.get_syn_node(a);
        let (pn1, bij1) = self.proven_shape(&a_node);
        let ProvenNode { elem: sh1, .. } = pn1.clone();

        let t = (sh1.clone(), bij1.clone());
        let b = self.lookup_internal(&t).expect("handle_congruence should only be called on hashcons collision!");
        let psn = self.classes[&b.id].nodes[&t.0].clone();
        let ProvenSourceNode { elem: bij, src_id: c } = psn.clone();
        let c = &self.mk_syn_identity_applied_id(c);
        let c_node = self.get_syn_node(&c);
        let (pn2, bij2) = self.proven_shape(&c_node);
        let ProvenNode { elem: sh2, .. } = pn2.clone();
        let t2 = (sh2.clone(), bij2.clone());
        if CHECKS {
            assert_eq!(&t.0, &t2.0);
        }

        #[cfg(feature = "explanations_tmp")]
        let mut vec = Vec::new();
        #[cfg(feature = "explanations_tmp")]
        for (l, r) in pn1.proofs.into_iter().zip(pn2.proofs.into_iter()) {
            let r_inv = self.prove_symmetry(r);
            let l_to_r = self.prove_transitivity(l, r_inv);
            vec.push(l_to_r);
        }

        #[cfg(feature = "explanations_tmp")]
        let proven_eq = self.prove_congruence(a.id, c.id, &vec);
        let eq = proven_eq.equ();
        let l = eq.l;
        let r = eq.r;
        // TODO l and r currently depend on ghost code.
        self.union_internal(&l, &r, proven_eq);
    }

    // upon touching an e-class, you need to update all usages of it.
    fn touched_class(&mut self, i: Id) {
        for sh in &self.classes[&i].usages {
            self.pending.insert(sh.clone());
        }
    }

}
