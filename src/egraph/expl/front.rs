use crate::*;

#[track_caller]
pub fn prove_explicit(l: &AppliedId, r: &AppliedId, j: Option<String>, reg: &ProofRegistry) -> ProvenEq {
    let eq = Equation { l: l.clone(), r: r.clone() };
    ExplicitProof(j).check(&eq, reg)
}

#[track_caller]
pub fn prove_reflexivity(id: &AppliedId, reg: &ProofRegistry) -> ProvenEq {
    let eq = Equation { l: id.clone(), r: id.clone() };
    ReflexivityProof.check(&eq, reg)
}

#[track_caller]
pub fn prove_symmetry(x: ProvenEq, reg: &ProofRegistry) -> ProvenEq {
    let eq = Equation { l: x.r.clone(), r: x.l.clone() };
    SymmetryProof(x).check(&eq, reg)
}

#[track_caller]
pub fn prove_transitivity(x: ProvenEq, y: ProvenEq, reg: &ProofRegistry) -> ProvenEq {
    let eq1 = x.clone();
    let eq2 = y.clone();
    let theta = match_app_id(&eq2.l, &eq1.r);
    let a = eq1.l.clone();
    let c = eq2.r.apply_slotmap_fresh(&theta);
    let eq = Equation { l: a, r: c };

    TransitivityProof(x.clone(), y.clone()).check(&eq, reg)
}

impl<L: Language> EGraph<L> {
    fn semify_equation(&self, eq: &Equation) -> Equation {
        Equation {
            l: self.semify_app_id(eq.l.clone()),
            r: self.semify_app_id(eq.r.clone()),
        }
    }

    // peq seems to imply `goal` if we remove all the redundant slots.
    // now we want a proof that even makes the redundant slots work out.
    // We assume that `peq` is already maximally dis-associated. Hence we only need to re-associate some redundant slots to reach the goal.
    fn associate_necessaries(&self, goal: &Equation, peq: ProvenEq) -> ProvenEq {
        if CHECKS {
            assert_match_equation(&self.semify_equation(goal), &self.semify_equation(&peq));
        }
        let l_red = self.get_redundancy_proof(peq.l.id);

        // goal.l.m :: slots(goal.l.id) -> X
        // goal.r.m :: slots(goal.r.id) -> X
        // goal_associations :: slots(goal.l.id) -> slots(goal.r.id)
        let goal_associations = goal.l.m.compose_partial(&goal.r.m.inverse());

        let mut current = peq;
        let current_associations = current.l.m.compose_partial(&current.r.m.inverse());
        let open_association_keys = &goal_associations.keys() - &current_associations.keys();

        let l_red_slots = &self.syn_slots(current.l.id) - &self.slots(current.l.id);
        let intersection = &open_association_keys & &l_red_slots;
        if intersection.len() > 0 {
            let mut subgoal = current.equ();
            for x in intersection {
                let f = Slot::fresh();
                subgoal.l.m.insert(x, f);
                subgoal.r.m.insert(goal_associations[x], f);
            }
            current = TransitivityProof(l_red, current).check(&subgoal, &self.proof_registry);
        }

        let r_red = self.get_redundancy_proof(current.r.id);
        TransitivityProof(current, r_red).check(goal, &self.proof_registry)
    }

    fn disassociation_necessary(&self, peq: &ProvenEq) -> bool {
        let l_rev = peq.l.m.inverse();
        let r_rev = peq.r.m.inverse();
        let l_slots = self.slots(peq.l.id);
        let r_slots = self.slots(peq.r.id);
        for s in &peq.l.slots() & &peq.r.slots() {
            if !l_slots.contains(&l_rev[s]) { return true; }
            if !r_slots.contains(&r_rev[s]) { return true; }
        }

        false
    }

    fn get_redundancy_proof(&self, i: Id) -> ProvenEq {
        let (leader, prf) = self.proven_unionfind_get(i);
        let red_prf = self.classes[&leader.id].redundancy_proof.clone();
        let inv_prf = prove_symmetry(prf.clone(), &self.proof_registry);
        let out = prove_transitivity(prf, prove_transitivity(red_prf, inv_prf, &self.proof_registry), &self.proof_registry);
        out
    }

    pub fn disassociate_proven_eq(&self, peq: ProvenEq) -> ProvenEq {
        if self.disassociation_necessary(&peq) {
            let mut peq = peq;
            let x = self.get_redundancy_proof(peq.l.id);
            let y = self.get_redundancy_proof(peq.r.id);
            peq = prove_transitivity(x, peq, &self.proof_registry);
            peq = prove_transitivity(peq, y, &self.proof_registry);

            peq
        } else {
            peq
        }
    }
}

// This API should be ignoring the values of redundant slots.
// This means that whether you pre-randomize all the Slots mapped to redundant Slots in both goal&input-proofs before passing them to prove_X should not affect the outcome.
// Further it should always produce maximally disassociated output.
impl<L: Language> EGraph<L> {
    #[track_caller]
    pub fn prove_explicit(&self, l: &AppliedId, r: &AppliedId, j: Option<String>) -> ProvenEq {
        self.check_syn_applied_id(l);
        self.check_syn_applied_id(r);
        self.disassociate_proven_eq(prove_explicit(l, r, j, &self.proof_registry))
    }

    #[track_caller]
    pub fn prove_reflexivity(&self, id: &AppliedId) -> ProvenEq {
        self.check_syn_applied_id(id);
        self.disassociate_proven_eq(prove_reflexivity(id, &self.proof_registry))
    }

    #[track_caller]
    pub fn prove_symmetry(&self, x: ProvenEq) -> ProvenEq {
        self.disassociate_proven_eq(prove_symmetry(x, &self.proof_registry))
    }

    #[track_caller]
    pub fn prove_transitivity(&self, x: ProvenEq, y: ProvenEq) -> ProvenEq {
        self.disassociate_proven_eq(prove_transitivity(x, y, &self.proof_registry))
    }

    fn assert_sem_congruence(&self, l: &AppliedId, r: &AppliedId, child_proofs: &[ProvenEq]) {
        // check that the congruence makes sense in "sem".
        let l_node = alpha_normalize(&self.semify_enode(self.get_syn_node(l)));
        let r_node = alpha_normalize(&self.semify_enode(self.get_syn_node(r)));

        let null_l = nullify_app_ids(&l_node);
        let null_r = nullify_app_ids(&r_node);
        assert_eq!(null_l, null_r);

        let n = l_node.applied_id_occurences().len();
        for i in 0..n {
            let lhs = &self.semify_equation(&child_proofs[i]);
            let rhs = &Equation {
                l: l_node.applied_id_occurences()[i].clone(),
                r: r_node.applied_id_occurences()[i].clone(),
            };
            assert_match_equation(lhs, rhs);
        }
    }

    fn lift_sem_congruence(&self, l: AppliedId, r: AppliedId, child_proofs: &[ProvenEq]) -> ProvenEq {
        self.assert_sem_congruence(&l, &r, &child_proofs);

        let l_node = alpha_normalize(&self.get_syn_node(&l));
        let r_node = alpha_normalize(&self.get_syn_node(&r));

        let n = child_proofs.len();

        let mut final_child_proofs = Vec::new();
        for i in 0..n {
            let li = l_node.applied_id_occurences()[i].clone();
            let ri = r_node.applied_id_occurences()[i].clone();

            let goal = Equation { l: li, r: ri };
            let old_prf = child_proofs[i].clone();
            let new_proof = self.associate_necessaries(&goal, old_prf);
            final_child_proofs.push(new_proof);
        }

        let eq = Equation { l: l.clone(), r: r.clone() };
        let cong = CongruenceProof(final_child_proofs).check(&eq, self);

        self.disassociate_proven_eq(cong)
    }

    pub fn prove_congruence(&self, l: Id, r: Id, child_proofs: &[ProvenEq]) -> ProvenEq {
        // pretty sure this is unnecessary:
        let child_proofs: Vec<_> = child_proofs.iter().map(|x| self.disassociate_proven_eq(x.clone())).collect();

        let l_id = self.mk_syn_identity_applied_id(l);
        let r_id = self.mk_syn_identity_applied_id(r);
        let l_node = alpha_normalize(&self.get_syn_node(&l_id));
        let r_node = alpha_normalize(&self.get_syn_node(&r_id));

        let mut map = SlotMap::new();
        for (l, r) in l_node.private_slot_occurences().iter().zip(r_node.private_slot_occurences().iter()) {
            try_insert(*l, *r, &mut map);
        }

        let n = l_node.applied_id_occurences().len();
        for i in 0..n {
            let eq = Equation {
                l: l_node.applied_id_occurences()[i].clone(),
                r: r_node.applied_id_occurences()[i].clone(),
            };
            // eq.l.m :: syn_slots(eq.l.id) -> syn_slots(l_id)
            // eq.r.m :: syn_slots(eq.r.id) -> syn_slots(r_id)

            let child_eq = child_proofs[i].equ();
            // child_eq.l.m :: syn_slots(eq.l.id) -> X
            // child_eq.r.m :: syn_slots(eq.r.id) -> X

            for (k, v) in child_eq.l.m.compose_partial(&child_eq.r.m.inverse()).iter() {
                let Some(k) = eq.l.m.get(k) else { continue };
                let Some(v) = eq.r.m.get(v) else { continue };
                try_insert(k, v, &mut map);
            }
        }

        // map = map.iter().filter(|(x, y)| l_id.slots().contains(&x) && r_id.slots().contains(&y)).collect();
        // assert!(map.keys().is_subset(&l_id.slots()));
        // assert!(map.values().is_subset(&r_id.slots()));

        // we want to prove l2_id = r_id.
        let l2_id = l_id.apply_slotmap_fresh(&map);
        let l2_node = self.get_syn_node(&l2_id);

        self.lift_sem_congruence(l2_id, r_id, &child_proofs)
    }
}

fn try_insert(k: Slot, v: Slot, map: &mut SlotMap) {
    if map.get(k) == Some(v) { return; }

    if map.get(k).is_some() { panic!(); }

    map.insert(k, v);
}
