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
