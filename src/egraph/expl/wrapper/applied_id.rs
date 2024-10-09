use crate::*;

#[derive(Clone, Debug)]
pub struct ProvenAppliedId {
    pub elem: AppliedId,
    pub proof: ProvenEq,
}
