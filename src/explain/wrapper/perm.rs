use crate::*;

use std::hash::{Hash, Hasher};
use std::ops::Index;

pub(crate) trait Permutation: Index<Slot, Output = Slot> + Clone + Eq + Hash {
    fn iter(&self) -> impl Iterator<Item = (Slot, Slot)>;
    fn compose(&self, other: &Self) -> Self;
    fn inverse(&self) -> Self;

    fn to_slotmap(&self) -> SlotMap {
        self.iter().collect()
    }
}

impl Permutation for Perm {
    fn iter(&self) -> impl Iterator<Item = (Slot, Slot)> {
        Self::iter(self)
    }
    fn compose(&self, other: &Self) -> Self {
        Self::compose(self, other)
    }
    fn inverse(&self) -> Self {
        Self::inverse(self)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ProvenPerm {
    pub elem: Perm,

    #[cfg(feature = "explanations")]
    pub proof: ProvenEq,

    #[cfg(feature = "explanations")]
    pub reg: ProofRegistry,
}

impl PartialEq for ProvenPerm {
    fn eq(&self, other: &Self) -> bool {
        self.elem == other.elem
    }
}

impl Eq for ProvenPerm {}

impl Hash for ProvenPerm {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.elem.hash(hasher);
    }
}

impl Permutation for ProvenPerm {
    fn iter(&self) -> impl Iterator<Item = (Slot, Slot)> {
        self.elem.iter()
    }
    fn compose(&self, other: &Self) -> Self {
        if CHECKS {
            self.check();
            other.check();
        }
        #[cfg(feature = "explanations")]
        if CHECKS {
            assert_eq!(self.proof.l.id, self.proof.r.id);
            assert_eq!(other.proof.l.id, other.proof.r.id);
            assert_eq!(self.proof.l.id, other.proof.l.id);
        }
        // TODO why is this the other way around?
        let map = self.elem.compose(&other.elem);
        #[cfg(feature = "explanations")]
        let prf = prove_transitivity(other.proof.clone(), self.proof.clone(), &self.reg);
        let out = ProvenPerm {
            elem: map,
            #[cfg(feature = "explanations")]
            proof: prf,
            #[cfg(feature = "explanations")]
            reg: self.reg.clone(),
        };
        if CHECKS {
            out.check();
        }
        out
    }

    fn inverse(&self) -> Self {
        if CHECKS {
            self.check();
        }
        let map = self.elem.inverse();
        #[cfg(feature = "explanations")]
        let prf = prove_symmetry(self.proof.clone(), &self.reg);
        let out = ProvenPerm {
            elem: map,
            #[cfg(feature = "explanations")]
            proof: prf,
            #[cfg(feature = "explanations")]
            reg: self.reg.clone(),
        };
        if CHECKS {
            out.check();
        }
        out
    }
}

impl ProvenPerm {
    pub(crate) fn identity(
        #[allow(unused)] i: Id,
        slots: &SmallHashSet<Slot>,
        #[allow(unused)] syn_slots: &SmallHashSet<Slot>,
        #[allow(unused)] reg: ProofRegistry,
    ) -> Self {
        let map = Perm::identity(slots);

        #[cfg(feature = "explanations")]
        let identity = SlotMap::identity(syn_slots);
        #[cfg(feature = "explanations")]
        let app_id = AppliedId::new(i, identity);
        #[cfg(feature = "explanations")]
        let prf = prove_reflexivity(&app_id, &reg);
        ProvenPerm {
            elem: map,
            #[cfg(feature = "explanations")]
            proof: prf,
            #[cfg(feature = "explanations")]
            reg,
        }
    }

    pub(crate) fn check(&self) {
        assert!(self.elem.is_perm());

        #[cfg(feature = "explanations")]
        {
            let id = self.proof.l.id;
            let slots = self.elem.keys();
            let syn_slots = self.proof.l.m.keys();

            assert_eq!(id, self.proof.l.id);
            assert_eq!(id, self.proof.r.id);
            assert_eq!(&self.proof.l.m.keys(), &syn_slots);
            assert_eq!(&self.proof.r.m.keys(), &syn_slots);

            let eq = Equation {
                l: AppliedId::new(id, SlotMap::identity(&slots)),
                r: AppliedId::new(id, self.elem.clone()),
            };
            assert_proves_equation(&self.proof, &eq);
        }
    }
}

impl Index<Slot> for ProvenPerm {
    type Output = Slot;

    fn index(&self, s: Slot) -> &Slot {
        self.elem.index(s)
    }
}
