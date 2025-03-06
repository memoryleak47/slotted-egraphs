use crate::*;

// syntactic add:
impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    pub fn add_syn_expr(&mut self, re: RecExpr<L>) -> AppliedId {
        let mut n = re.node;
        let mut refs: Vec<&mut AppliedId> = n.applied_id_occurrences_mut();
        if CHECKS {
            assert_eq!(re.children.len(), refs.len());
        }
        for (i, child) in re.children.into_iter().enumerate() {
            *(refs[i]) = self.add_syn_expr(child);
        }
        self.add_syn(n)
    }

    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    pub fn add_syn(&mut self, enode: L) -> AppliedId {
        #[cfg(not(feature = "explanations"))]
        {
            self.add(enode)
        }

        #[cfg(feature = "explanations")]
        {
            let enode = self.synify_enode(enode);

            self.add(enode.clone());

            if let Some(x) = self.lookup_syn(&enode) {
                if CHECKS {
                    assert_eq!(enode.slots(), x.slots());
                }
                return x;
            }

            let old_slots = enode.slots();
            let fresh_to_old = Bijection::bijection_from_fresh_to(&old_slots);
            let old_to_fresh = fresh_to_old.inverse();
            let new_enode = enode.apply_slotmap(&old_to_fresh);
            let c = self.alloc_eclass(&old_to_fresh.values(), new_enode.clone());

            let pc = self.pc_find(&self.refl_pc(c));

            self.handle_congruence(pc);

            let c_a = self.mk_syn_applied_id(c, fresh_to_old.clone());
            if CHECKS {
                assert_eq!(enode.slots(), c_a.slots());
            }

            c_a
        }
    }

    #[cfg(feature = "explanations")]
    fn lookup_syn(&self, enode: &L) -> Option<AppliedId> {
        let (sh, bij) = enode.weak_shape();
        let i = self.syn_hashcons.get(&sh)?;

        // bij :: SHAPE -> X
        // i :: slots(i.id) -> SHAPE
        let i = i.apply_slotmap(&bij);
        Some(i)
    }
}

// semantic add:
impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    pub fn add_expr(&mut self, re: RecExpr<L>) -> AppliedId {
        let mut n = re.node;
        let mut refs: Vec<&mut AppliedId> = n.applied_id_occurrences_mut();
        if CHECKS {
            assert_eq!(re.children.len(), refs.len());
        }
        for (i, child) in re.children.into_iter().enumerate() {
            *(refs[i]) = self.add_expr(child);
        }
        self.add(n)
    }

    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    pub fn add(&mut self, enode: L) -> AppliedId {
        self.add_internal(self.shape_called_from_add(enode))
    }

    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    fn shape_called_from_add(&self, enode: L) -> (L, Bijection) {
        self.shape(&enode)
    }

    // self.add(x) = y implies that x.slots() is a superset of y.slots().
    // x.slots() - y.slots() are redundant slots.
    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    pub(in crate::egraph) fn add_internal(&mut self, t: (L, SlotMap)) -> AppliedId {
        if let Some(x) = self.lookup_internal(&t) {
            return x;
        }

        // TODO this code is kinda exactly what add_syn is supposed to do anyways. There's probably a way to write this more concisely.
        // We convert the enode to "syn" so that semantic_add will compute the necessary redundancy proofs.
        let enode = t.0.refresh_private().apply_slotmap(&t.1);
        let enode = self.synify_enode(enode);

        let syn = self.mk_singleton_class(enode);
        self.semify_app_id(syn)
    }

    pub fn lookup(&self, n: &L) -> Option<AppliedId> {
        self.lookup_internal(&self.shape(n))
    }

    pub(in crate::egraph) fn lookup_internal(
        &self,
        (shape, n_bij): &(L, Bijection),
    ) -> Option<AppliedId> {
        let i = self.hashcons.get(&shape)?;
        let c = &self.classes[i];
        let cn_bij = &c.nodes[&shape].elem;

        // X = shape.slots()
        // Y = n.slots()
        // Z = c.slots()
        // n_bij :: X -> Y
        // cn_bij :: X -> Z
        // out :: Z -> Y
        let out = cn_bij.inverse().compose(&n_bij);

        // Note that ENodes in an EClass can have redundant slots.
        // They shouldn't come up in the AppliedId.
        let out = out.iter().filter(|(x, _)| c.slots.contains(x)).collect();

        let app_id = self.mk_sem_applied_id(*i, out);

        if CHECKS {
            assert_eq!(&c.slots, &app_id.m.keys());
        }

        Some(app_id)
    }
}

impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    // returns a syn applied id.
    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    fn mk_singleton_class(&mut self, syn_enode: L) -> AppliedId {
        let old_slots = syn_enode.slots();

        let fresh_to_old = Bijection::bijection_from_fresh_to(&old_slots);
        let old_to_fresh = fresh_to_old.inverse();

        // allocate new class & slot set.
        let fresh_slots = old_to_fresh.values();
        let syn_enode_fresh = syn_enode.apply_slotmap_fresh(&old_to_fresh);
        let i = self.alloc_eclass(&fresh_slots, syn_enode_fresh.clone());

        // we use semantic_add so that the redundancy, symmetry and congruence checks run on it.
        let t = syn_enode_fresh.weak_shape();
        self.raw_add_to_class(i, t.clone(), i);
        self.pending.insert(t.0, PendingType::Full);
        self.rebuild_called_from_add();

        self.mk_syn_applied_id(i, fresh_to_old)
    }

    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    fn rebuild_called_from_add(&mut self) {
        self.rebuild();
    }

    // adds (sh, bij) to the eclass `id`.
    // TODO src_id should be optional!
    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    pub(in crate::egraph) fn raw_add_to_class(
        &mut self,
        id: Id,
        (sh, bij): (L, Bijection),
        src_id: Id,
    ) {
        let psn = ProvenSourceNode { elem: bij, src_id };

        let tmp1 = self
            .classes
            .get_mut(&id)
            .unwrap()
            .nodes
            .insert(sh.clone(), psn);
        let tmp2 = self.hashcons.insert(sh.clone(), id);
        if CHECKS {
            assert!(tmp1.is_none());
            assert!(tmp2.is_none());
        }
        for ref_id in sh.ids() {
            let usages = &mut self.classes.get_mut(&ref_id).unwrap().usages;
            usages.insert(sh.clone());
        }
    }

    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    pub(in crate::egraph) fn raw_remove_from_class(&mut self, id: Id, sh: L) -> ProvenSourceNode {
        let opt_psn = self.classes.get_mut(&id).unwrap().nodes.remove(&sh);
        let opt_id = self.hashcons.remove(&sh);
        if CHECKS {
            assert!(opt_psn.is_some());
            assert!(opt_id.is_some());
        }
        for ref_id in sh.ids() {
            let usages = &mut self.classes.get_mut(&ref_id).unwrap().usages;
            usages.remove(&sh);
        }

        opt_psn.unwrap()
    }
}

impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    // TODO make the public API auto "fresh" slots.
    pub fn alloc_empty_eclass(&mut self, _slots: &HashSet<Slot>) -> Id {
        panic!("Can't use alloc_empty_eclass if explanations are enabled!");
    }

    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    pub(in crate::egraph) fn alloc_eclass(&mut self, slots: &HashSet<Slot>, syn_enode: L) -> Id {
        let c_id = Id(self.unionfind_len()); // Pick the next unused Id.

        let syn_slots = syn_enode.slots();
        let proven_perm =
            ProvenPerm::identity(c_id, &slots, &syn_slots, self.proof_registry.clone());

        let c = EClass {
            nodes: HashMap::default(),
            group: Group::identity(&proven_perm),
            slots: slots.clone(),
            usages: HashSet::default(),
            syn_enode: syn_enode.clone(),
            analysis_data: N::make(&self, &syn_enode),
        };
        self.classes.insert(c_id, c);

        {
            // add syn_enode to the hashcons.
            let (sh, bij) = syn_enode.weak_shape();

            if CHECKS {
                assert!(!self.syn_hashcons.contains_key(&sh));
            }

            let app_id = self.mk_syn_applied_id(c_id, bij.inverse());
            self.syn_hashcons.insert(sh, app_id);
        }

        let syn_app_id = self.mk_syn_identity_applied_id(c_id);
        let pai = self.refl_pai(&syn_app_id);
        self.unionfind_set(c_id, pai);

        c_id
    }
}
