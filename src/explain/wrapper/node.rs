use crate::*;

use std::hash::*;

#[derive(Clone, Debug)]
pub(crate) struct ProvenNode<L> {
    pub elem: L,

    // These proofs have as 'lhs' the base that is situation dependent.
    // And the 'rhs' is 'elem'.
    #[cfg(feature = "explanations")]
    pub proofs: Vec<ProvenEq>,
}

impl<L: Language> PartialEq for ProvenNode<L> {
    fn eq(&self, other: &Self) -> bool {
        self.elem == other.elem
    }
}

impl<L: Language> Eq for ProvenNode<L> {}

impl<L: Language> Hash for ProvenNode<L> {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.elem.hash(hasher);
    }
}

impl<L: Language> ProvenNode<L> {
    // checks that `proofs` brings us from `base` to `elem`.
    #[cfg(feature = "explanations")]
    #[allow(unused)]
    pub(crate) fn check_base(&self, base: &L) {
        let l = base.applied_id_occurrences();
        let r = self.elem.applied_id_occurrences();
        let n = self.proofs.len();
        assert_eq!(n, l.len());
        assert_eq!(n, r.len());
        for i in 0..n {
            let l = l[i].clone();
            let r = r[i].clone();
            let eq = Equation { l, r };
            assert_proves_equation(&self.proofs[i], &eq);
        }
    }

    pub(crate) fn weak_shape(&self) -> (Self, Bijection) {
        let (sh, bij) = self.elem.weak_shape();
        let pn = ProvenNode {
            elem: sh,

            #[cfg(feature = "explanations")]
            proofs: self.proofs.clone(),
        };
        (pn, bij)
    }
}

impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    pub(crate) fn check_pn(&self, #[allow(unused)] pn: &ProvenNode<L>) {
        #[cfg(feature = "explanations")]
        {
            let a = &pn.proofs;
            let b = &pn.elem.applied_id_occurrences();
            assert_eq!(a.len(), b.len());
            for (x, y) in a.iter().zip(b.iter()) {
                assert_eq!(x.r.id, y.id);
            }
        }
    }

    #[cfg(feature = "explanations")]
    // If we take the `proofs` to go backward from `elem`, where do we land?
    #[allow(unused)]
    pub(crate) fn pn_source_node(&self, pn: &ProvenNode<L>) -> L {
        todo!()
    }

    pub(crate) fn refl_pn(&self, start: &L) -> ProvenNode<L> {
        #[cfg(feature = "explanations")]
        let rfl = start
            .applied_id_occurrences()
            .into_iter()
            .map(|x| self.refl_proof(x.id))
            .collect();
        ProvenNode {
            elem: start.clone(),
            #[cfg(feature = "explanations")]
            proofs: rfl,
        }
    }

    #[cfg(feature = "explanations")]
    fn refl_proof(&self, i: Id) -> ProvenEq {
        let syn_slots = self.syn_slots(i);
        let identity = SlotMap::identity(&syn_slots);
        let app_id = AppliedId::new(i, identity);
        self.prove_reflexivity(&app_id)
    }

    pub(crate) fn chain_pn_map(
        &self,
        start: &ProvenNode<L>,
        f: impl Fn(usize, ProvenAppliedId) -> ProvenAppliedId,
    ) -> ProvenNode<L> {
        let mut pn = start.clone();
        let n = pn.elem.applied_id_occurrences().len();

        let mut app_ids_mut: Vec<&mut AppliedId> = pn.elem.applied_id_occurrences_mut();

        #[cfg(feature = "explanations")]
        let proofs_mut: &mut [ProvenEq] = &mut pn.proofs;

        for i in 0..n {
            let old_app_id: &mut AppliedId = app_ids_mut[i];
            #[cfg(feature = "explanations")]
            let old_proof: &mut ProvenEq = &mut proofs_mut[i];

            let tmp_pai = ProvenAppliedId {
                elem: old_app_id.clone(),
                #[cfg(feature = "explanations")]
                proof: old_proof.clone(),
            };
            let pai = f(i, tmp_pai);

            *old_app_id = pai.elem;

            #[cfg(feature = "explanations")]
            {
                *old_proof = pai.proof;
            }
        }
        pn
    }
}
