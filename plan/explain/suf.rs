struct SufClass {
    leader: AppliedId, // corresponding equation needs to be in the registry; already includes redundancy
    syn_slots: Set<Slot>, // the original slot-set, required for reflexivity.
    group: Group, // corresponding equations need to be in the registry, equations are maximally redundant.
}

struct Suf {
    v: Vec<SufClass>,
    registry: Registry,
}

struct Registry(HashMap<Equation, ProofStep>);

impl Suf {
    fn new() -> Self;
    fn add(&mut self, s: Set<Slot>) -> Id;
    fn union(&mut self, Equation, j: Justification);
    fn is_equal(&self, eq: &Equation) -> Option<Registry> {
      // canonicalize both ids-sides, and check whether the group contains it.
      // return a new registry, containing all relevant proof steps
    }
}

struct Equation(Id, Id, /*partial*/ SlotMap);

enum ProofStep {
    Transitivity(Equation, Equation),
    Refl(Id), // generates syn refl
    Symmetry(Equation),
    Explicit(Justification),
}

struct Justification {
    s: String,
    children: Vec<Equation>
}
