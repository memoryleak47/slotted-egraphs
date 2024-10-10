use crate::*;

#[derive(Clone, Debug)]
pub struct ProvenSourceNode {
    pub elem: Bijection,

    // remembers the original AppliedId, where this came from
    #[cfg(feature = "explanations_tmp")]
    pub src_id: AppliedId,
}
