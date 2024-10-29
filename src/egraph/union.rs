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

            if CHECKS {
                #[cfg(feature = "explanations")]
                assert_eq!(proven_perm.proof.l.id, id);

                proven_perm.check();
            }
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

    fn assert_ty(&self, m: &SlotMap, keys: &HashSet<Slot>, values: &HashSet<Slot>) {
        assert!(m.keys().is_subset(&keys));
        assert!(m.values().is_subset(&values));
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

        let analysis_from = self.analysis_data(from.id).clone();
        let analysis_to = self.analysis_data_mut(to.id);
        *analysis_to = N::merge(analysis_from, analysis_to.clone());

        // from.m :: slots(from.id) -> X
        // to.m :: slots(to.id) -> X
        let map = to.m.compose_partial(&from.m.inverse());
        if CHECKS {
            self.assert_ty(&map, &self.slots(to.id), &self.slots(from.id));
        }

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
            self.raw_remove_from_class(from.id, sh.clone());
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

        if self.classes.get_mut(&to.id).unwrap().group.add_set(set) {
            self.touched_class(to.id);
        }

        // touched because the class is now dead and no e-nodes should point to it.
        self.touched_class(from.id);
    }
}
