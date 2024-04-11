use crate::*;

impl<L: Language> EGraph<L> {
    // creates a new eclass with slots "l.slots() cap r.slots()".
    // TODO get references here instead!
    // returns whether it actually did something.
    pub fn union(&mut self, l: AppliedId, r: AppliedId) -> bool {
        self.union_internal(l, r)
    }

    pub(in crate::egraph) fn union_internal(&mut self, l: AppliedId, r: AppliedId) -> bool {
        // normalize inputs
        let l = self.find_applied_id(l);
        let r = self.find_applied_id(r);

        // early return, if union should not be made.
        if l == r { return false; }

        if l.id == r.id {
            eprintln!("We reject self-unions for now!");
            return false;
        };

        // make the slots fresh.
        let all_slots = &l.slots() | &r.slots();
        let fresh_map = SlotMap::bijection_from_fresh_to(&all_slots).inverse();
        let l = l.apply_slotmap(&fresh_map);
        let r = r.apply_slotmap(&fresh_map);

        let slots = &l.slots() & &r.slots();
        let c_id = self.alloc_eclass(&slots);

        for lr in [l, r] {
            // We need to filter the ones out that are newly "redundant".
            let lr_m = lr.m.iter().filter(|(x, y)| slots.contains(y)).collect();
            self.merge_into_eclass(lr.id, c_id, &lr_m);
        }

        return true;
    }

    // merges the EClass `from` into `to`. This deprecates the EClass `from`.
    // map :: slots(from) -> slots(to)
    fn merge_into_eclass(&mut self, from: Id, to: Id, map: &SlotMap) {
        // Should hold here: self.inv();

        let mut future_unions = Vec::new();

        // X = slots(from)
        // Y = slots(to)
        // map :: X -> Y

        assert!(map.keys().is_subset(&self.classes[&from].slots));
        assert_eq!(self.classes[&to].slots, map.values());

        // 1. add unionfind entry 'from -> to'.
        self.unionfind.set(from, self.mk_applied_id(to, map.inverse()));

        // 2. move enodes from 'from' to 'to'.
        let from_enodes = self.classes.get(&from).unwrap().nodes.clone();
        for (sh, bij) in from_enodes {
            // SH = slots(sh)
            // bij :: SH -> X

            // out_bij :: SH -> Y
            let mut out_bij = bij.compose_partial(map);

            // map redundant slots too.
            for x in sh.slots() {
                if !out_bij.contains_key(x) {
                    out_bij.insert(x, Slot::fresh());
                }
            }

            self.raw_remove_from_class(from, (sh.clone(), bij.clone()));
            self.raw_add_to_class(to, (sh, out_bij));
        }

        let from_class = self.classes.get(&from).unwrap().clone();

        // 3. fix all ENodes that reference `from`.
        for sh in from_class.usages {
            let i = self.hashcons[&sh];
            let bij = self.classes[&i].nodes[&sh].clone();
            self.raw_remove_from_class(i, (sh.clone(), bij.clone()));
            let n = sh.apply_slotmap(&bij);
            let norm = self.find_enode(&n);
            let (norm_sh, norm_bij) = norm.shape();

            // Check whether `norm` makes a Slot redundant.
            let class_slots = self.classes[&i].slots.clone();
            let norm_slots = norm.slots();
            if !class_slots.is_subset(&norm_slots) {
                let l = self.mk_identity_applied_id(i);

                let sub = &class_slots & &norm_slots;

                // We union `i` with an empty EClass that is just missing a slot.
                let r = self.alloc_eclass_fresh(&sub);

                // TODO This is a dangerous solution.
                // The invariant that each e-class.slots subset e-node.slots doesn't hold until the corresponding future_union is handled.
                // In other words, we call union_internal, even though the e-graphs invariants are not upheld.
                future_unions.push((l, r));
            }

            // Check whether `norm` collides with something:
            if let Some(app_id) = self.lookup_internal(&norm) {
                // If there is a collision, we don't add it directly.
                // Instead, we union it together.
                let l = self.mk_identity_applied_id(i);
                let r = app_id;
                future_unions.push((l, r));
            } else {
                self.raw_add_to_class(i, norm.shape());
            }
        }

        for (x, y) in future_unions {
            self.union_internal(x, y);
        }

        // Should hold here: self.inv();
    }
}
