use crate::*;

// Should ProvenNode also contain the src-id?
pub struct ProvenNode<L> {
    elem: L,
    proofs: Vec<ProvenEq>,
}
