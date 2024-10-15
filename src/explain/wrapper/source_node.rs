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

impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    pub fn refl_psn(&self, start: &L, syn_id: &AppliedId) -> (L, ProvenSourceNode) {
        let (sh, bij) = start.weak_shape();

        #[cfg(feature = "explanations")]
        let rfl = start.applied_id_occurences()
                       .into_iter()
                       .map(|x| self.refl_proof(x.id))
                       .collect();

        (sh, ProvenSourceNode {
            elem: bij,

            #[cfg(feature = "explanations")]
            proofs: rfl,

            pai: self.refl_pai(&syn_id),

            src_id: syn_id.id,
        })
    }
}
