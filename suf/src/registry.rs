use crate::*;

pub struct Registry(HashMap<(Id, Id, SlotMap), ProofStep>);

// trait
pub trait WithRegistry {
    fn with(&mut self, f: impl FnOnce(&mut Registry));
}

impl WithRegistry for () {
    fn with(&mut self, f: impl FnOnce(&mut Registry)) {}
}

impl WithRegistry for Registry {
    fn with(&mut self, f: impl FnOnce(&mut Registry)) {
        f(self)
    }
}

enum ProofStep {
    Refl, // never stored in the Suf, but might be returned from `explain_equivalence`.
    Symmetry,
    Transitivity(Id, SlotMap), // contains b if transitivity using a = b = c
    Explicit(String, /*depends on*/ Vec<(Id, Id, SlotMap)>),
}
