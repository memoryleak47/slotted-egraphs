use crate::*;

use std::hash::{Hasher, Hash};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Equation {
    pub l: AppliedId,
    pub r: AppliedId,
}

#[derive(Clone, Debug)]
pub struct ExplicitProof(pub /*justification: */ Option<String>);
#[derive(Clone, Debug)]
pub struct ReflexivityProof;
#[derive(Clone, Debug)]
pub struct SymmetryProof(pub ProvenEq);
#[derive(Clone, Debug)]
pub struct TransitivityProof(pub ProvenEq, pub ProvenEq);
#[derive(Clone, Debug)]
pub struct CongruenceProof(pub Vec<ProvenEq>);

#[derive(Debug, Clone)]
pub enum Proof {
    Explicit(ExplicitProof),
    Reflexivity(ReflexivityProof),
    Symmetry(SymmetryProof),
    Transitivity(TransitivityProof),
    Congruence(CongruenceProof),

    // Both global renaming within equations and alpha-equivalence will be handled in the other rules too.
    // All equations will be understood as an arbitrary representative from its global renaming equivalence class.
    // So f(x, y) = g(x, y) is conceptually the same equation as f(a, b) = g(a, b).
    // In other words, whenever you use an equation, you always do it using "match_app_id".
}

pub type ProvenEq = Arc<ProvenEqRaw>;

#[derive(Debug, Clone)]
pub struct ProvenEqRaw {
    // fields are intentionally private so that only this module can construct instances for it.
    // These equations should always be fully "syn", i.e. they should not have any missing slot arguments, even redundant slots have to be passed explicitly.
    eq: Equation,
    proof: Proof,
}

impl ProvenEqRaw {
    pub fn null() -> ProvenEq {
        let app_id = AppliedId::new(Id(0), Default::default());
        Arc::new(ProvenEqRaw {
            eq: Equation { l: app_id.clone(), r: app_id.clone() },
            proof: Proof::Explicit(ExplicitProof(None)),
        })
    }

    pub fn equ(&self) -> Equation {
        (**self).clone()
    }

    pub fn check<L: Language, N: Analysis<L>>(&self, eg: &EGraph<L, N>) {
        let Equation { l, r } = self.equ();
        eg.check_syn_applied_id(&l);
        eg.check_syn_applied_id(&r);
    }
}

impl PartialEq for ProvenEqRaw {
    // TODO normalize slotnames before this?
    fn eq(&self, other: &Self) -> bool { self.eq == other.eq }
}

impl Eq for ProvenEqRaw { }

impl Hash for ProvenEqRaw {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        // TODO normalize slotnames before this?
        self.eq.hash(hasher);
    }
}


impl ExplicitProof {
    pub fn check(&self, eq: &Equation, reg: &ProofRegistry) -> ProvenEq {
        let eq = eq.clone();
        let proof = Proof::Explicit(self.clone());
        reg.insert(Arc::new(ProvenEqRaw { eq, proof }))
    }
}

impl ReflexivityProof {
    pub fn check(&self, eq: &Equation, reg: &ProofRegistry) -> ProvenEq {
        assert_eq!(eq.l, eq.r);

        let eq = eq.clone();
        let proof = Proof::Reflexivity(self.clone());
        reg.insert(Arc::new(ProvenEqRaw { eq, proof }))
    }
}

impl SymmetryProof {
    pub fn check(&self, eq: &Equation, reg: &ProofRegistry) -> ProvenEq {
        let SymmetryProof(x) = self;

        let flipped = Equation { l: x.r.clone(), r: x.l.clone() };
        assert_match_equation(eq, &flipped);

        let eq = eq.clone();
        let proof = Proof::Symmetry(self.clone());
        reg.insert(Arc::new(ProvenEqRaw { eq, proof }))
    }
}

impl TransitivityProof {
    pub fn check(&self, eq: &Equation, reg: &ProofRegistry) -> ProvenEq {
        let TransitivityProof(eq1, eq2) = self;

        let mut theta1 = {
            // eq1.l*theta1 == eq.l
            // -> theta1 == eq1.l^-1 * eq.l
            eq1.l.m.inverse().compose_partial(&eq.l.m)
        };
        let mut theta2 = {
            // eq2.r*theta2 == eq.r
            // -> theta2 == eq2.r^-1 * eq.r
            eq2.r.m.inverse().compose_partial(&eq.r.m)
        };

        let recompute_theta1 = |theta1: &mut SlotMap, theta2: &SlotMap| {
            // eq1.r*theta1 == eq2.l*theta2
            // -> theta1 == eq1.r^-1 * eq2.l * theta2
            *theta1 = theta1.try_union(&eq1.r.m.inverse().compose_partial(&eq2.l.m).compose_partial(theta2)).unwrap();
        };

        let recompute_theta2 = |theta1: &SlotMap, theta2: &mut SlotMap| {
            // eq1.r*theta1 == eq2.l*theta2
            // -> theta2 == eq2.l^-1 * eq1.r * theta2
            *theta2 = theta2.try_union(&eq2.l.m.inverse().compose_partial(&eq1.r.m).compose_partial(theta1)).unwrap();
        };

        recompute_theta1(&mut theta1, &theta2);
        recompute_theta2(&theta1, &mut theta2);

        for x in eq1.slots() {
            if !theta1.contains_key(x) { theta1.insert(x, Slot::fresh()); }
        }
        recompute_theta2(&theta1, &mut theta2);
        for x in eq2.slots() {
            if !theta2.contains_key(x) { theta2.insert(x, Slot::fresh()); }
        }

        let renamed_eq1 = eq1.apply_slotmap(&theta1);
        let renamed_eq2 = eq2.apply_slotmap(&theta2);

        assert_eq!(renamed_eq1.l, eq.l);
        assert_eq!(renamed_eq2.r, eq.r);
        assert_eq!(renamed_eq1.r, renamed_eq2.l);


        let eq = eq.clone();
        let proof = Proof::Transitivity(self.clone());
        reg.insert(Arc::new(ProvenEqRaw { eq, proof }))
    }
}

// replaces 'private' slots with enumerated slot-names, like a shape.
pub fn alpha_normalize<L: Language>(n: &L) -> L {
    let (sh, bij) = n.weak_shape();
    if CHECKS {
        let all_slots: HashSet<_> = sh.all_slot_occurences().into_iter().collect();
        assert!(&bij.values().is_disjoint(&all_slots));
    }
    sh.apply_slotmap(&bij)
}

impl CongruenceProof {
    pub fn check<L: Language, N: Analysis<L>>(&self, eq: &Equation, eg: &EGraph<L, N>) -> ProvenEq {
        let CongruenceProof(child_proofs) = self;

        let l = alpha_normalize(&eg.get_syn_node(&eq.l));
        let r = alpha_normalize(&eg.get_syn_node(&eq.r));

        let null_l = nullify_app_ids(&l);
        let null_r = nullify_app_ids(&r);
        assert_eq!(null_l, null_r);

        let l_v = l.applied_id_occurences();
        let r_v = r.applied_id_occurences();

        assert_eq!(l_v.len(), child_proofs.len());
        assert_eq!(r_v.len(), child_proofs.len());

        let l_v = l_v.into_iter();
        let r_v = r_v.into_iter();

        let c_v = child_proofs.into_iter();
        for ((ll, rr), prf) in l_v.zip(r_v).zip(c_v) {
            let eq1 = &Equation { l: ll, r: rr };
            let eq2 = prf.deref();
            assert_match_equation(eq1, eq2);
        }

        let eq = eq.clone();
        let proof = Proof::Congruence(self.clone());
        eg.proof_registry.insert(Arc::new(ProvenEqRaw { eq, proof }))
    }
}

impl Equation {
    pub fn slots(&self) -> HashSet<Slot> {
        &self.l.slots() | &self.r.slots()
    }

    #[track_caller]
    pub fn apply_slotmap(&self, m: &SlotMap) -> Self {
        Equation {
            l: self.l.apply_slotmap(&m),
            r: self.r.apply_slotmap(&m),
        }
    }

    pub fn apply_slotmap_fresh(&self, m: &SlotMap) -> Self {
        let mut m = m.clone();
        for s in &self.l.slots() | &self.r.slots() {
            if !m.contains_key(s) {
                m.insert(s, Slot::fresh());
            }
        }
        Equation {
            l: self.l.apply_slotmap(&m),
            r: self.r.apply_slotmap(&m),
        }
    }
}

impl Deref for ProvenEqRaw {
    type Target = Equation;

    fn deref(&self) -> &Equation {
        &self.eq
    }
}

impl ProvenEqRaw {
    pub fn proof(&self) -> &Proof {
        &self.proof
    }
}

// returns the global renaming theta, s.t. a.apply_slotmap(theta) = b, if it exists.
#[track_caller]
pub fn match_app_id(a: &AppliedId, b: &AppliedId) -> SlotMap {
    assert_eq!(a.id, b.id);
    assert_eq!(a.m.keys(), b.m.keys(), "match_app_id failed: different set of arguments");

    // a.m :: slots(i) -> A
    // b.m :: slots(i) -> B
    // theta :: A -> B
    let theta = a.m.inverse().compose(&b.m);

    if CHECKS {
        assert_eq!(&a.apply_slotmap(&theta), b);
    }

    theta
}

// returns the bijective renaming theta, s.t. a.apply_slotmap(theta) = b, if it exists.
pub fn assert_match_equation(a: &Equation, b: &Equation) -> SlotMap {
    let theta_l = match_app_id(&a.l, &b.l);
    let theta_r = match_app_id(&a.r, &b.r);

    let theta = theta_l.try_union(&theta_r).unwrap_or_else(|| panic!("trying to union {theta_l:?} with {theta_r:?} while trying to match '{a:?}' against '{b:?}'"));
    assert!(theta.is_bijection(), "trying to unify {theta_l:?} with {theta_r:?}, in assert_match_equation(\n  {a:?},\n  {b:?}\n)");

    if CHECKS {
        assert_eq!(&a.apply_slotmap(&theta), b);
    }

    theta
}

pub fn assert_proves_equation(peq: &ProvenEq, eq: &Equation) {
    let mut e: Equation = (***peq).clone();

    for s in e.l.m.keys() {
        if !eq.l.m.contains_key(s) {
            e.l.m.remove(s);
        }
    }

    for s in e.r.m.keys() {
        if !eq.r.m.contains_key(s) {
            e.r.m.remove(s);
        }
    }

    assert_match_equation(&e, eq);
}
