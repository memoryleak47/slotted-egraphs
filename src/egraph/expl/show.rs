use crate::*;

type ShowMap = HashMap<*const ProvenEqRaw, (usize, String)>;

impl ProvenEqRaw {
    pub fn show(&self) {
        println!("{}", self.to_string());
    }

    pub fn show_expr<L: Language>(&self, eg: &EGraph<L>) {
        println!("{}", self.to_string_expr(eg));
    }

    // to string API:

    pub fn to_string(&self) -> String {
        self.show_impl(&|i| format!("{i:?}"))
    }

    pub fn to_string_expr<L: Language>(&self, eg: &EGraph<L>) -> String {
        self.show_impl(&|i| {
            eg.get_syn_expr(i).to_string()
        })
    }

    // internals:
    pub fn show_impl(&self, f: &impl Fn(&AppliedId) -> String) -> String {
        let mut map = Default::default();
        self.show_impl2(&mut map, f);

        let mut map_sorted: Vec<_> = map.into_iter().collect();
        map_sorted.sort_by_key(|(_, (i, _))| *i);
        let mut out = String::new();
        for (_, (_, s)) in map_sorted {
            out.extend(s.chars());
            out.push('\n');
        }
        out
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

