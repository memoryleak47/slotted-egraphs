use crate::*;

#[derive(Clone, Debug)]
pub struct ProvenAppliedId {
    pub elem: AppliedId,

    // @ghost
    pub proof: ProvenEq,
}
