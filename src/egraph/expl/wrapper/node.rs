use crate::*;

// Should ProvenNode also contain the src-id?
pub struct ProvenNode<L>(L, Vec<ProvenEq>);
