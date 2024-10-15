use crate::*;

#[derive(Clone, Debug)]
pub(crate) struct ProvenSourceNode {
    pub elem: Bijection,

    // remembers the original Id, where this came from
    // TODO make ghost.
    pub src_id: Id,
}
