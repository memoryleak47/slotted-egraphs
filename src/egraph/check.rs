use vec_collections::AbstractVecSet;

use crate::*;

impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    // mk_sem_applied_id & friends.
    #[track_caller]
    pub(crate) fn mk_sem_applied_id(&self, i: Id, m: SlotMap) -> AppliedId {
        let app_id = AppliedId::new(i, m);

        if CHECKS {
            self.check_sem_applied_id(&app_id);
        }

        app_id
    }

    #[track_caller]
    pub(crate) fn mk_sem_identity_applied_id(&self, i: Id) -> AppliedId {
        self.mk_sem_applied_id(i, SlotMap::identity(&self.slots(i)))
    }

    #[track_caller]
    pub(crate) fn check_sem_applied_id(&self, app_id: &AppliedId) {
        app_id.check();
        assert_eq!(
            self.slots(app_id.id),
            app_id.m.keys(),
            "checking sem AppliedId failed: Wrong key-set, {app_id:?}"
        );
    }

    // mk_syn_applied_id & friends.
    #[track_caller]
    pub(crate) fn mk_syn_applied_id(&self, i: Id, m: SlotMap) -> AppliedId {
        let app_id = AppliedId::new(i, m);

        if CHECKS {
            self.check_syn_applied_id(&app_id);
        }

        app_id
    }

    #[track_caller]
    pub(crate) fn mk_syn_identity_applied_id(&self, i: Id) -> AppliedId {
        self.mk_syn_applied_id(i, SlotMap::identity(&self.syn_slots(i)))
    }

    #[track_caller]
    pub(crate) fn check_syn_applied_id(&self, app_id: &AppliedId) {
        app_id.check();
        assert_eq!(
            self.syn_slots(app_id.id),
            app_id.m.keys(),
            "checking syn AppliedId failed: Wrong key-set, {app_id:?}"
        );
    }

    pub fn check(&self) {
        // Checks whether the hashcons / usages are correct.
        // And also checks that each Shape comes up in at most one EClass!
        let mut hashcons = HashMap::default();
        let mut usages = HashMap::default();

        for (i, _) in &self.classes {
            usages.insert(*i, HashSet::default());
        }

        // redundancy-check for leaders.
        // TODO add a similar check for followers, using unionfind_get.
        for (i, _) in &self.classes {
            if !self.is_alive(*i) {
                continue;
            }

            // There should be no symmetries witnessed by the unionfind.
            // It would make stuff just weird.
            let sem = self.mk_sem_identity_applied_id(*i);
            let sem2 = self.find_applied_id(&sem);
            assert_eq!(sem, sem2);

            #[cfg(feature = "explanations")]
            {
                let c = &self.classes[i];
                let eq = self.proven_unionfind_get(*i).proof.equ();
                // eq.l.m :: slots(i) -> X
                // eq.r.m :: slots(i) -> X
                let tmp = eq.l.m.compose_partial(&eq.r.m.inverse());
                assert!(tmp.is_perm());
                assert_eq!(c.slots, tmp.keys());
                assert_eq!(c.slots, tmp.values());
            }
        }

        for (i, c) in &self.classes {
            for sh in c.nodes.keys() {
                assert!(!hashcons.contains_key(sh));
                hashcons.insert(sh.clone(), *i);

                for ref_id in sh.ids() {
                    usages.get_mut(&ref_id).unwrap().insert(sh.clone());
                }
            }
        }

        assert_eq!(hashcons, self.hashcons);
        for (i, c) in &self.classes {
            assert_eq!(usages[&i], c.usages);
        }

        for (_, c) in &self.classes {
            for p in c.group.all_perms() {
                p.check();
            }
        }

        // check that self.classes contains exactly these classes which point to themselves in the unionfind.
        let all_keys = self
            .unionfind_iter()
            .map(|(x, _)| x)
            .collect::<HashSet<_>>();
        let all_values = self
            .unionfind_iter()
            .map(|(_, x)| x.id)
            .collect::<HashSet<_>>();
        let all_classes = self.classes.keys().copied().collect::<HashSet<_>>();
        let all: HashSet<Id> = &(&all_keys | &all_values) | &all_classes;
        for i in all {
            // if they point to themselves, they should do it using the identity.
            if self.is_alive(i) {
                assert_eq!(self.unionfind_get(i), self.mk_sem_identity_applied_id(i));
            } else {
                assert!(self.classes[&i].nodes.is_empty());
                for sh in &self.classes[&i].usages {
                    assert_eq!(self.pending.get(&sh), Some(&PendingType::Full));
                }
            }
        }

        // check that no EClass has Slot(0) in its API.
        for (_, c) in &self.classes {
            assert!(!c.slots.contains(&Slot::numeric(0)));
        }

        // Check that the Unionfind has valid AppliedIds.
        for (_, app_id) in self.unionfind_iter() {
            check_internal_applied_id::<L, N>(self, &app_id);
        }

        // Check that all ENodes are valid.
        for (_, c) in &self.classes {
            for (sh, ProvenSourceNode { elem: bij, .. }) in &c.nodes {
                let real = sh.apply_slotmap(bij);
                assert!(real.slots().is_superset(&c.slots));

                if self.pending.get(&sh) == Some(&PendingType::Full) {
                    continue;
                }

                let (computed_sh, computed_bij) = self.shape(&real);
                assert_eq!(&computed_sh, sh);

                // computed_bij :: shape-slots -> slots(i)
                // bij :: shape-slots -> slots(i)
                let perm = computed_bij.inverse().compose_partial(&bij);
                assert!(c.group.contains(&perm));

                for x in real.applied_id_occurrences() {
                    check_internal_applied_id::<L, N>(self, &x);
                }
            }
        }

        fn check_internal_applied_id<L: Language, N: Analysis<L>>(
            eg: &EGraph<L, N>,
            app_id: &AppliedId,
        ) {
            // 1. the app_id needs to be normalized!
            let y = eg.find_applied_id(app_id);
            assert_eq!(app_id, &y);

            // 2. It needs to have exactly the same slots as the underlying EClass.
            assert_eq!(&app_id.m.keys(), &eg.classes[&app_id.id].slots);
        }
    }
}
