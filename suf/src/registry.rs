use crate::*;

pub struct Registry(HashMap<Equation, ProofStep>);

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
    Refl,
    Symmetry,
    Transitivity(AppliedId), // contains b if transitivity using a = b = c
    Explicit(String, /*depends on*/ Vec<Equation>),
}
