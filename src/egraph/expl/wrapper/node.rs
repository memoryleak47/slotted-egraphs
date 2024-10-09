use crate::*;

use std::hash::*;

// Should ProvenNode also contain the src-id?
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

    pub fn chain_pn(&self, start: &ProvenNode<L>, next: &ProvenNode<L>) -> ProvenNode<L> {
        todo!()
    }

    fn refl_proof(&self, i: Id) -> ProvenEq {
        let syn_slots = self.syn_slots(i);
        let identity = SlotMap::identity(&syn_slots);
        let app_id = AppliedId::new(i, identity);
        self.prove_reflexivity(&app_id)
    }
}
