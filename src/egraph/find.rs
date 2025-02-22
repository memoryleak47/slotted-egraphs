use crate::*;

impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    fn unionfind_get_impl(&self, i: Id, map: &mut [ProvenAppliedId]) -> ProvenAppliedId {
        let entry = &mut map[i.0];

        if entry.elem.id == i {
            return entry.clone();
        }

        let entry = entry.clone();

        // entry.0.m :: slots(entry.0.id) -> slots(i)
        // entry_to_leader.0.m :: slots(leader) -> slots(entry.0.id)
        let entry_to_leader = self.unionfind_get_impl(entry.elem.id, map);
        let new = self.chain_pai(&entry, &entry_to_leader);

        map[i.0] = new.clone();
        new
    }

    pub(crate) fn unionfind_set(&self, i: Id, pai: ProvenAppliedId) {
        #[cfg(feature = "explanations")]
        if CHECKS {
            pai.proof.check(self);
            assert_eq!(i, pai.proof.l.id);
            assert_eq!(pai.elem.id, pai.proof.r.id);
        }

        let mut lock = self.unionfind.borrow_mut();
        if lock.len() == i.0 {
            lock.push(pai);
        } else {
            lock[i.0] = pai;
        }
    }

    pub(crate) fn proven_unionfind_get(&self, i: Id) -> ProvenAppliedId {
        let mut map = self.unionfind.borrow_mut();
        self.unionfind_get_impl(i, &mut *map)
    }

    pub(crate) fn unionfind_get(&self, i: Id) -> AppliedId {
        self.proven_unionfind_get(i).elem
    }

    /// Returns whether an id is still alive, or whether it was merged into another class.
    pub fn is_alive(&self, i: Id) -> bool {
        let map = self.unionfind.borrow();
        map[i.0].elem.id == i
    }

    pub(crate) fn unionfind_iter(&self) -> impl Iterator<Item = (Id, AppliedId)> {
        let mut map = self.unionfind.borrow_mut();
        let mut out = Vec::new();

        for x in (0..map.len()).map(Id) {
            let y = self.unionfind_get_impl(x, &mut *map).elem;
            out.push((x, y));
        }

        out.into_iter()
    }

    pub(crate) fn unionfind_len(&self) -> usize {
        self.unionfind.borrow().len()
    }

    pub(crate) fn find_enode(&self, enode: &L) -> L {
        self.proven_find_enode(enode).elem
    }

    pub(crate) fn proven_find_enode(&self, enode: &L) -> ProvenNode<L> {
        let pn = self.refl_pn(enode);
        self.proven_proven_find_enode(&pn)
    }

    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    pub(crate) fn proven_proven_find_enode(&self, enode: &ProvenNode<L>) -> ProvenNode<L> {
        self.chain_pn_map(enode, |_, pai| self.proven_proven_find_applied_id(&pai))
    }

    // normalize i.id
    //
    // Example 1:
    // 'find(c1(s10, s11)) = c2(s11, s10)', where 'c1(s0, s1) -> c2(s1, s0)' in unionfind.
    //
    // Example 2:
    // 'find(c1(s3, s7, s8)) = c2(s8, s7)', where 'c1(s0, s1, s2) -> c2(s2, s1)' in unionfind,
    pub fn find_applied_id(&self, i: &AppliedId) -> AppliedId {
        #[cfg(feature = "explanations")]
        let i = &self.synify_app_id(i.clone());

        self.proven_find_applied_id(i).elem
    }

    pub(crate) fn proven_find_applied_id(&self, i: &AppliedId) -> ProvenAppliedId {
        let pai = self.refl_pai(i);
        self.proven_proven_find_applied_id(&pai)
    }

    pub(crate) fn proven_proven_find_applied_id(&self, pai: &ProvenAppliedId) -> ProvenAppliedId {
        if CHECKS {
            self.check_pai(&pai);
        }

        let mut pai2 = self.proven_unionfind_get(pai.elem.id);

        pai2.elem.m = pai2.elem.m.compose_partial(&pai.elem.m);

        #[cfg(feature = "explanations")]
        {
            pai2.proof = prove_transitivity(pai.proof.clone(), pai2.proof, &self.proof_registry);
        }

        if CHECKS {
            self.check_pai(&pai);
        }

        pai2
    }

    pub(crate) fn find_id(&self, i: Id) -> Id {
        self.unionfind_get(i).id
    }

    pub fn ids(&self) -> Vec<Id> {
        let map = self.unionfind.borrow();
        (0..map.len())
            .map(Id)
            .filter(|x| map[x.0].elem.id == *x)
            .collect()
    }
}
