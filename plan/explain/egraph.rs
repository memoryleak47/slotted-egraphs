// hashcons collisions etc. are all checked manually for now.
// there's no hashing tricks in this conceptual design.

struct ProvenContains {
    node: L,
    parent_proof: Equation, 
    child_proofs: Vec<Equation>,
}

struct Class {
    syn_node: L,
    active_nodes: Vec<ProvenContains>,
}

struct EGraph {
    suf: Suf,
    classes: Vec<Class>,
}

impl EGraph {
    fn add(&mut self, n: L) -> AppliedId {
        if let m*x = n for some x, i with classes[i].active_nodes.contains(x) {
            return m*i
        } else {
            let i = suf.add(n.slots());
            classes.insert(i, Class {
                syn_node: n, // really?
                active_nodes: vec![n],
            });
            while n ==_cong m*n for some m {
                suf.groups[i].extend(m);
            }
            return identity * i;
        }
    }

    fn union(&mut self, x: AppliedId, y: AppliedId) {
        suf.union(x, y);

        if y.id deprecated {
            moves nodes from y to x
        }
    }

    fn explain_equivalence(&mut self, x: AppliedId, y: AppliedId) -> Option<Proof> {
        let registry = self.suf.is_equal(x, y)?;

        let mut map: HashMap<Equation, LemmaId> = HashMap::new();
        let mut lemmas = Vec::new();

        for (Equation(l, r, m), step) in registry {
            let lemma_id = lemmas.len();
            map.insert(eq, lemma_id);

            let slotmap = mapping syn_slots(l) u syn_slots(r) -> {$0, ...} while slotmap[x] = slotmap[m[x]];
            let lemma = Lemma {
                lhs: self.syn_term(slotmap * l),
                rhs: self.syn_term(slotmap * r),
                by: self.port_step(step, &map),
            };
            lemmas.push(lemma);
        }
        Some(Proof(lemmas))
    }

    fn port_step(&self, step: ProofStep, map: &HashMap<Equation, LemmaId>) -> ProofStep2 {
        match step {
            Transitivity(Equation, Equation) => { ... }
            Refl(Id) => { ... }
            Symmetry(Equation) => { ... }
            Explicit(j) => {
                if j.is_congruence() {
                    ProofStep2::Congruence(...)
                } else {
                    ProofStep2::Explicit(...)
                }
            }
        }
    }

    fn syn_term(&self, i: /*syn*/ AppliedId) -> Term {
        todo!()
    }
}

struct Proof {
    // the last lemma is the goal
    Vec<Lemma>, // indexed by LemmaId
}

// drawn as:
// lemma3($0, $1, $2, $3):
//   foo($0) = bar($1, $2, $3)
//   by thing(lemma2($1), lemma4($2, $3))
struct Lemma {
    lhs: Term,
    rhs: Term,
    by: ProofStep2,
}

// or Applied<LemmaId>
struct AppliedLemma {
    lemma_id: usize,
    application: SlotMap,
}

enum ProofStep2 {
    Reflexivity,
    Symmetry(AppliedLemma),
    Transitivity(AppliedLemma, AppliedLemma),
    Congruence(Vec<AppliedLemma>),
    Explicit(/*justification*/ String),
}
