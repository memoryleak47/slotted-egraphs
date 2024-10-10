use crate::*;

impl<L: Language> EGraph<L> {
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

    pub fn unionfind_set(&self, i: Id, pai: ProvenAppliedId) {
        if CHECKS {
            pai.proof.check(self);
            assert_eq!(i, pai.proof.l.id);
            assert_eq!(pai.elem.id, pai.proof.r.id);
        }
        let mut lock = self.unionfind.try_lock().unwrap();
        if lock.len() == i.0 {
            lock.push(pai);
        } else {
            lock[i.0] = pai;
        }
    }

    pub fn proven_unionfind_get(&self, i: Id) -> ProvenAppliedId {
        let mut map = self.unionfind.try_lock().unwrap();
        self.unionfind_get_impl(i, &mut *map)
    }

    pub fn unionfind_get(&self, i: Id) -> AppliedId {
        self.proven_unionfind_get(i).elem
    }

    pub fn unionfind_iter(&self) -> impl Iterator<Item=(Id, AppliedId)> {
        let mut map = self.unionfind.try_lock().unwrap();
        let mut out = Vec::new();

        for x in (0..map.len()).map(Id) {
            let y = self.unionfind_get_impl(x, &mut *map).elem;
            out.push((x, y));
        }

        out.into_iter()
    }

    pub fn unionfind_len(&self) -> usize {
        self.unionfind.try_lock().unwrap().len()
    }

    pub fn find_enode(&self, enode: &L) -> L {
        self.proven_find_enode(enode).elem
    }

    pub fn proven_find_enode(&self, enode: &L) -> ProvenNode<L> {
        let mut v = Vec::new();
        let out = enode.map_applied_ids(|x| {
            let pai = self.proven_find_applied_id(&x);
            v.push(pai.proof);
            pai.elem
        });
        ProvenNode {
            elem: out,
            proofs: v
        }
    }

    // normalize i.id
    //
    // Example 1:
    // 'find(c1(s10, s11)) = c2(s11, s10)', where 'c1(s0, s1) -> c2(s1, s0)' in unionfind.
    //
    // Example 2:
    // 'find(c1(s3, s7, s8)) = c2(s8, s7)', where 'c1(s0, s1, s2) -> c2(s2, s1)' in unionfind,
    pub fn find_applied_id(&self, i: &AppliedId) -> AppliedId {
        self.proven_find_applied_id(i).elem
    }

    pub fn proven_find_applied_id(&self, i: &AppliedId) -> ProvenAppliedId {
        let mut pai = self.proven_unionfind_get(i.id);
        pai.proof.check(self);

        // I = self.slots(i.id);
        // A = self.slots(a.id);
        // i.m   :: I -> X
        // a.m   :: A -> I
        // out.m :: A -> X

        pai.elem = self.mk_sem_applied_id(
            pai.elem.id,
            pai.elem.m.compose_partial(&i.m), // This is partial if `i.id` had redundant slots.
        );
        pai
    }

    pub fn find_id(&self, i: Id) -> Id {
        self.unionfind_get(i).id
    }
}
