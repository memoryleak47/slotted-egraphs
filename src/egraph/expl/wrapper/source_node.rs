use crate::*;

#[derive(Clone, Debug)]
pub struct ProvenSourceNode {
    pub elem: Bijection,

    // remembers the original AppliedId, where this came from
    // @ghost
    pub src_id: AppliedId,
}
