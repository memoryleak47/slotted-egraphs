use crate::*;

// Should ProvenNode also contain the src-id?
pub struct ProvenContains<L> {
    // contains the proof that the e-node is equal to some app-ids syn-enode.
    pub elem: ProvenNode<L>,

    // proofs that this app-id is equal to our target app-id.
    // @ghost
    pub proof: ProvenEq,
}
