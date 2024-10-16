use crate::*;

#[derive(Clone, Debug)]
pub(crate) struct ProvenSourceNode {
    pub elem: Bijection,

    pub pai: ProvenAppliedId,

    #[cfg(feature = "explanations")]
    pub proofs: Vec<ProvenEq>,
}

#[cfg(feature = "explanations")]
impl ProvenSourceNode {
    pub(crate) fn src_id(&self) -> Id {
        self.pai.proof.l.id
    }
}

impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    pub(crate) fn refl_psn(&self, start: &L, syn_id: &AppliedId) -> (L, ProvenSourceNode) {
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
        })
    }
}
