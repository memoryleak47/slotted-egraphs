use crate::*;

use std::ops::Index;
use std::hash::{Hash, Hasher};

pub trait Permutation: Index<Slot, Output=Slot> + Clone + Eq + Hash {
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)>;
    fn compose(&self, other: &Self) -> Self;
    fn inverse(&self) -> Self;

    fn to_slotmap(&self) -> SlotMap {
        self.iter().collect()
    }
}

impl Permutation for Perm {
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)> { Self::iter(self) }
    fn compose(&self, other: &Self) -> Self { Self::compose(self, other) }
    fn inverse(&self) -> Self { Self::inverse(self) }
}

#[derive(Clone, Debug)]
pub struct ProvenPerm(pub Perm, pub ProvenEq, pub ProofRegistry);

impl PartialEq for ProvenPerm {
    fn eq(&self, other: &Self) -> bool { self.0 == other.0 }
}

impl Eq for ProvenPerm { }

impl Hash for ProvenPerm {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher);
    }
}

impl Permutation for ProvenPerm {
    fn iter(&self) -> impl Iterator<Item=(Slot, Slot)> { self.0.iter() }
    fn compose(&self, other: &Self) -> Self {
        self.check();
        other.check();
        if CHECKS {
            assert_eq!(self.1.l.id, self.1.r.id);
            assert_eq!(other.1.l.id, other.1.r.id);
            assert_eq!(self.1.l.id, other.1.l.id);
        }
        // TODO why is this the other way around?
        let map = self.0.compose(&other.0);
        let prf = prove_transitivity(other.1.clone(), self.1.clone(), &self.2);
        let out = ProvenPerm(map, prf, self.2.clone());
        out.check();
        out
    }

    fn inverse(&self) -> Self {
        self.check();
        let map = self.0.inverse();
        let prf = prove_symmetry(self.1.clone(), &self.2);
        let out = ProvenPerm(map, prf, self.2.clone());
        out.check();
        out
    }
}

impl ProvenPerm {
    pub fn identity(i: Id, slots: &HashSet<Slot>, syn_slots: &HashSet<Slot>, reg: ProofRegistry) -> Self {
        let map = Perm::identity(slots);

        let identity = SlotMap::identity(syn_slots);
        let app_id = AppliedId::new(i, identity);
        let prf = prove_reflexivity(&app_id, &reg);
        ProvenPerm(map, prf, reg)
    }

    fn to_string(&self) -> String {
        format!("{:?}", (&self.0, &**self.1))
    }

    pub fn check(&self) {
        let id = self.1.l.id;
        let slots = self.0.keys();
        let syn_slots = self.1.l.m.keys();

        assert_eq!(id, self.1.l.id);
        assert_eq!(id, self.1.r.id);
        assert_eq!(&self.1.l.m.keys(), &syn_slots);
        assert_eq!(&self.1.r.m.keys(), &syn_slots);
        assert!(self.0.is_perm());

        let eq = Equation { l: AppliedId::new(id, SlotMap::identity(&slots)), r: AppliedId::new(id, self.0.clone()) };
        assert_proves_equation(&self.1, &eq);
    }
}

impl Index<Slot> for ProvenPerm {
    type Output = Slot;

    fn index(&self, s: Slot) -> &Slot {
        self.0.index(s)
    }
}
