use crate::*;

type Pos = Vec<u8>;

struct FlatteningContext<L: Language> {
    head:     RecExpr<L>,
    slot_map: HashMap<Slot, Slot>
}

struct Step<L: Language> {
    dst:    RecExpr<L>,
    rw_pos: Pos,
    jus:    String,
    back:   bool
}

impl<L: Language> Step<L> {
    
    fn to_string(&self) -> String {
        // Print this in the egg format. That is, in the form 
        // "(app (const foo) (Rewrite <= add0 (lit 12)))".
        todo!()
    }
}

impl ProvenEqRaw {

    /// Returns a string representation of a flattened explanation.
    pub fn to_flat_string<L: Language, N: Analysis<L>>(&self, graph: &EGraph<L, N>) -> String {
        let mut init_ctx = FlatteningContext {
            head:     graph.get_syn_expr(&self.l),
            slot_map: Default::default()
        };

        Self::to_steps(graph, self, vec![], false, &mut init_ctx)
            .into_iter()
            .fold("".to_string(), |res, step| format!("{}{}\n", res, step.to_string()))
    }

    fn to_steps<L: Language, N: Analysis<L>>(
        graph: &EGraph<L, N>, eq: &ProvenEqRaw, pos: Pos, symm: bool, ctx: &mut FlatteningContext<L>
    ) -> Vec<Step<L>> {
        match eq.proof() {
            Proof::Reflexivity(ReflexivityProof) => {
                vec![]
            },
            Proof::Symmetry(SymmetryProof(e)) => {
                Self::to_steps(graph, e, pos, !symm, ctx)
            },
            Proof::Transitivity(TransitivityProof(e1, e2)) => {
                let mut result = vec![];
                if symm {
                    result.extend(Self::to_steps(graph, e2, pos.clone(), symm, ctx));
                    result.extend(Self::to_steps(graph, e1, pos, symm, ctx));
                } else {
                    result.extend(Self::to_steps(graph, e1, pos.clone(), symm, ctx));
                    result.extend(Self::to_steps(graph, e2, pos, symm, ctx));
                }
                result
            },
            Proof::Congruence(CongruenceProof(es)) => {
                let mut result = vec![];
                for (idx, e) in es.iter().enumerate() {
                    let mut subpos = pos.clone();
                    subpos.push(idx.try_into().unwrap());
                    result.extend(Self::to_steps(graph, e, subpos, symm, ctx));  
                }
                result
            },
            Proof::Explicit(ExplicitProof(jus)) => {
                let Equation { l: lhs, r: rhs } = eq.equ();
                let subdst_id = if symm { lhs } else { rhs };
                let subdst = graph.get_syn_expr(&subdst_id);
                let dst = ctx.head.replace_subexpr(&pos, subdst);
                // TODO: Apply the slot map to dst.
                vec![Step { dst, rw_pos: pos, jus: jus.as_ref().unwrap().clone(), back: symm }]
            },            
        }
    }
}

impl<L: Language> RecExpr<L> {

    fn replace_subexpr(&self, pos: &Pos, e: RecExpr<L>) -> RecExpr<L> {
        todo!()
    }
}