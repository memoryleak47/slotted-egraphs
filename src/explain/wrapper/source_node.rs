use crate::*;

#[derive(Clone, Debug)]
pub(crate) struct ProvenSourceNode {
    pub elem: Bijection,

    pub pai: ProvenAppliedId,

    #[cfg(feature = "explanations")]
    pub proofs: Vec<ProvenEq>,

    // TODO remove.
    pub src_id: Id,
}
