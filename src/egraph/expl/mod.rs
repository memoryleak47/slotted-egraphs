use crate::*;

mod proof;
pub use proof::*;

mod front;
pub use front::*;

mod registry;
pub use registry::*;

mod wrapper;
pub use wrapper::*;

impl<L: Language> EGraph<L> {
    pub fn explain_equivalence(&mut self, t1: RecExpr<L>, t2: RecExpr<L>) -> ProvenEq {
        let i1 = self.add_syn_expr(t1);
        let i2 = self.add_syn_expr(t2);

        if !self.eq(&i1, &i2) { panic!("Can't explain an equivalence that does not hold!"); }

        let pai1 = self.proven_find_applied_id(&i1);
        let ProvenAppliedId { elem: l1, proof: prf1 } = &pai1;

        let pai2 = self.proven_find_applied_id(&i2);
        let ProvenAppliedId { elem: l2, proof: prf2 } = &pai2;

        if CHECKS {
            assert_eq!(l1.id, l2.id);
        }
        let id = l1.id;

        let bij = l2.m.compose(&l1.m.inverse());
        let symmetry_prf = &self.classes[&id].group.proven_contains(&bij).unwrap();
        let (l1, prf1) = self.apply_proven_perm(pai1, symmetry_prf);

        let prf2 = self.prove_symmetry(prf2.clone());

        let final_eq = Equation { l: i1, r: i2 };
        let p = TransitivityProof(prf1, prf2.clone()).check(&final_eq, &self.proof_registry);

        if CHECKS {
            assert_proves_equation(&p, &final_eq);
        }

        p
    }
}


type ShowMap = HashMap<*const ProvenEqRaw, (usize, String)>;

impl ProvenEqRaw {
    pub fn show(&self) {
        self.show_impl(&|i| format!("{i:?}"))
    }

    pub fn show_expr<L: Language>(&self, eg: &EGraph<L>) {
        self.show_impl(&|i| {
            eg.get_syn_expr(i).to_string()
        })
    }

    pub fn show_impl(&self, f: &impl Fn(&AppliedId) -> String) {
        let mut map = Default::default();
        self.show_impl2(&mut map, f);

        let mut map_sorted: Vec<_> = map.into_iter().collect();
        map_sorted.sort_by_key(|(_, (i, _))| *i);
        for (_, (_, s)) in map_sorted {
            println!("{}", s);
        }
    }

    fn subproofs(&self) -> Vec<&ProvenEq> {
        match self.proof() {
            Proof::Explicit(ExplicitProof(j)) => vec![],
            Proof::Reflexivity(ReflexivityProof) => vec![],
            Proof::Symmetry(SymmetryProof(x)) => vec![x],
            Proof::Transitivity(TransitivityProof(x1, x2)) => vec![x1, x2],
            Proof::Congruence(CongruenceProof(xs)) => xs.iter().collect(),
        }
    }

    pub fn show_impl2(&self, v: &mut ShowMap, f: &impl Fn(&AppliedId) -> String) {
        let mut stack: Vec<&ProvenEqRaw> = vec![self];

        'outer: while let Some(x) = stack.last().cloned() {
            let mut ids = Vec::new();
            for sub in x.subproofs() {
                let subptr = (&**sub) as *const ProvenEqRaw;
                if let Some(o) = v.get(&subptr) {
                    ids.push(o.0.to_string());
                } else {
                    stack.push(sub);
                    continue 'outer;
                }
            }
            let prf_string = match x.proof() {
                Proof::Explicit(ExplicitProof(j)) => format!("{j:?}"),
                Proof::Reflexivity(ReflexivityProof) => format!("refl"),
                Proof::Symmetry(SymmetryProof(_)) => format!("symmetry({})", ids[0]),
                Proof::Transitivity(TransitivityProof(_, _)) => {
                    format!("transitivity({}, {})", ids[0], ids[1])
                },
                Proof::Congruence(CongruenceProof(xs)) => {
                    let s = ids.join(", ");
                    format!("congruence({s})")
                },
            };

            let i = v.len();
            let Equation { l, r } = &**x;
            let out = format!("lemma{i}: '{} = {}'", f(l), f(r));
            let out = format!("{out}\n  by {prf_string}\n");
            v.insert(x as *const ProvenEqRaw, (i, out));
            assert_eq!(stack.pop(), Some(x));
        }

    }
}
