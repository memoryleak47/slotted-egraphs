use crate::*;

impl<L: Language> EGraph<L> {
    // We lazily semify the entries, only when we encounter them.
    fn unionfind_semify_entry(&self, entry: &mut (AppliedId, ProvenEq)) {
        if entry.0.m.keys().len() > self.slots(entry.0.id).len() {
            entry.0 = self.semify_app_id(entry.0.clone());
        }

        // maybe I want something like this to disassociate?
        // but definitely only when necessary.
        // entry.1 = prove_transitivity(entry.1.clone(), self.classes[&entry.0.id].redundancy_proof.clone());
    }

    fn unionfind_get_impl(&self, i: Id, map: &mut [(AppliedId, ProvenEq)]) -> (AppliedId, ProvenEq) {
        let entry = &mut map[i.0];

        if entry.0.id == i {
            self.unionfind_semify_entry(entry);
            return entry.clone();
        }

        let entry = entry.clone();

        // entry.0.m :: slots(entry.0.id) -> slots(i)
        // entry_to_leader.0.m :: slots(leader) -> slots(entry.0.id)
        let entry_to_leader = self.unionfind_get_impl(entry.0.id, map);
        let new = (
            entry_to_leader.0.apply_slotmap(&entry.0.m),
            prove_transitivity(entry.1, entry_to_leader.1, &self.proof_registry),
        );

        map[i.0] = new.clone();
        new
    }

    pub fn unionfind_set(&self, i: Id, app: AppliedId, proof: ProvenEq) {
        if CHECKS {
            proof.check(self);
            assert_eq!(i, proof.l.id);
            assert_eq!(app.id, proof.r.id);
        }
        let mut lock = self.unionfind.try_lock().unwrap();
        if lock.len() == i.0 {
            lock.push((app, proof));
        } else {
            lock[i.0] = (app, proof);
        }
    }

    pub fn proven_unionfind_get(&self, i: Id) -> (AppliedId, ProvenEq) {
        let mut map = self.unionfind.try_lock().unwrap();
        let (app_id, peq) = self.unionfind_get_impl(i, &mut *map);
        std::mem::drop(map);

        // We can directly access the redundancy_proof here, because we know that 'app_id.id' is a leader.
        let red = self.classes[&app_id.id].redundancy_proof.clone();
        let peq = prove_transitivity(peq, red, &self.proof_registry);
        (app_id, peq)
    }

    pub fn unionfind_get(&self, i: Id) -> AppliedId {
        self.proven_unionfind_get(i).0
    }

    pub fn unionfind_iter(&self) -> impl Iterator<Item=(Id, AppliedId)> {
        let mut map = self.unionfind.try_lock().unwrap();
        let mut out = Vec::new();

        for x in (0..map.len()).map(Id) {
            let y = self.unionfind_get_impl(x, &mut *map).0;
            out.push((x, y));
        }

        out.into_iter()
    }

    pub fn unionfind_len(&self) -> usize {
        self.unionfind.try_lock().unwrap().len()
    }

    pub fn find_enode(&self, enode: &L) -> L {
        self.proven_find_enode(enode).0
    }

    pub fn proven_find_enode(&self, enode: &L) -> (L, Vec<ProvenEq>) {
        let mut v = Vec::new();
        let out = enode.map_applied_ids(|x| {
            let (app, prf) = self.proven_find_applied_id(&x);
            v.push(prf);
            app
        });
        (out, v)
    }

    // normalize i.id
    //
    // Example 1:
    // 'find(c1(s10, s11)) = c2(s11, s10)', where 'c1(s0, s1) -> c2(s1, s0)' in unionfind.
    //
    // Example 2:
    // 'find(c1(s3, s7, s8)) = c2(s8, s7)', where 'c1(s0, s1, s2) -> c2(s2, s1)' in unionfind,
    pub fn find_applied_id(&self, i: &AppliedId) -> AppliedId {
        self.proven_find_applied_id(i).0
    }

    pub fn proven_find_applied_id(&self, i: &AppliedId) -> (AppliedId, ProvenEq) {
        let (a, prf) = self.proven_unionfind_get(i.id);
        prf.check(self);

        // I = self.slots(i.id);
        // A = self.slots(a.id);
        // i.m   :: I -> X
        // a.m   :: A -> I
        // out.m :: A -> X

        let out = self.mk_sem_applied_id(
            a.id,
            a.m.compose_partial(&i.m), // This is partial if `i.id` had redundant slots.
        );
        (out, prf)
    }

    pub fn find_id(&self, i: Id) -> Id {
        self.unionfind_get(i).id
    }
}
