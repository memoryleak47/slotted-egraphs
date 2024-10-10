use crate::*;

use std::hash::*;

// Should ProvenNode also contain the src-id?
#[derive(Clone)]
pub struct ProvenNode<L> {
    pub elem: L,

    // @ghost
    pub proofs: Vec<ProvenEq>,
}

impl<L: Language> PartialEq for ProvenNode<L> {
    fn eq(&self, other: &Self) -> bool { self.elem == other.elem }
}

impl<L: Language> Eq for ProvenNode<L> { }

impl<L: Language> Hash for ProvenNode<L> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.elem.hash(hasher);
    }
}

impl<L: Language> ProvenNode<L> {
    // checks that `proofs` brings us from `base` to `elem`.
    pub fn check_base(&self, base: &L) {
        let l = base.applied_id_occurences();
        let r = self.elem.applied_id_occurences();
        let n = self.proofs.len();
        assert_eq!(n, l.len());
        assert_eq!(n, r.len());
        for i in 0..n {
            let l = l[i].clone();
            let r = r[i].clone();
            let eq = Equation { l, r };
            assert_proves_equation(&self.proofs[i], &eq);
        }
    }

    pub fn weak_shape(&self) -> (Self, Bijection) {
        let (sh, bij) = self.elem.weak_shape();
        let pn = ProvenNode {
            elem: sh,

            // @ghost:
            proofs: self.proofs.clone(),
        };
        (pn, bij)
    }
}

impl<L: Language> EGraph<L> {
    pub fn refl_pn(&self, start: &L) -> ProvenNode<L> {
        let mut rfl = Vec::new();
        for x in start.applied_id_occurences() {
            rfl.push(self.refl_proof(x.id));
        }

        ProvenNode {
            elem: start.clone(),
            proofs: rfl,
        }
    }

    fn refl_proof(&self, i: Id) -> ProvenEq {
        let syn_slots = self.syn_slots(i);
        let identity = SlotMap::identity(&syn_slots);
        let app_id = AppliedId::new(i, identity);
        self.prove_reflexivity(&app_id)
    }

    pub fn chain_pn_map(&self, start: &ProvenNode<L>, f: impl Fn(usize, ProvenAppliedId) -> ProvenAppliedId) -> ProvenNode<L> {
        let mut pn = start.clone();
        let n = pn.proofs.len();

        let mut app_ids_mut: Vec<&mut AppliedId> = pn.elem.applied_id_occurences_mut();
        let mut proofs_mut: &mut [ProvenEq] = &mut pn.proofs;
        for i in 0..n {
            let old_app_id: &mut AppliedId = app_ids_mut[i];
            let old_proof: &mut ProvenEq = &mut proofs_mut[i];

            let tmp_pai = ProvenAppliedId {
                elem: old_app_id.clone(),
                proof: old_proof.clone(),
            };
            let ProvenAppliedId { elem: new_app_id, proof: new_proof } = f(i, tmp_pai);

            *old_app_id = new_app_id;
            *old_proof = new_proof;
        }
        pn
    }
}
