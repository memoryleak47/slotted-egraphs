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
        if let Some((next, subpos)) = self.rw_pos.split_first() {
            let (op, children) = self.dst.node.to_op();
            let mut str = format!("{}", op);

            if children.is_empty() { return str; }

            let mut child_node_idx = 0;
            for child in children.into_iter() {
                match child {
                    Child::AppliedId(_) => {
                        let child_node = self.dst.children[child_node_idx].clone();
                        if child_node_idx == (*next as usize) {
                            let substep = Step { 
                                dst: child_node, 
                                rw_pos: subpos.to_vec(), 
                                jus: self.jus.clone(), 
                                back: self.back 
                            };
                            str = format!("{} {}", str, substep.to_string());
                        } else {
                            str = format!("{} {}", str, child_node);
                        }
                        child_node_idx += 1;
                    },
                    Child::Slot(slot) => {
                        str = format!("{} {}", str, slot);
                    }
                }
            }
            format!("({})", str)
        } else {
            let dir_str = if self.back { "<=" } else { "=>" };
            format!("(Rewrite{} {} {})", dir_str, self.jus, self.dst)
        }
    }
}

impl ProvenEqRaw {

    /// Returns a string representation of a flattened explanation.
    pub fn to_flat_string<L: Language, N: Analysis<L>>(&self, graph: &EGraph<L, N>) -> String {
        let start = graph.get_syn_expr(&self.l);

        let mut init_ctx = FlatteningContext { head: start.clone(), slot_map: Default::default() };
        let steps = Self::to_steps(graph, self, vec![], false, &mut init_ctx);     
        
        let mut result = start.to_string();
        for step in steps { result = format!("{}\n{}", result, step.to_string()); }
        result
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
                let step = Step { 
                    dst: dst.clone(), rw_pos: pos, jus: jus.as_ref().unwrap().clone(), back: symm 
                };
                ctx.head = dst;
                vec![step]
            },            
        }
    }
}

impl<L: Language> RecExpr<L> {

    fn replace_subexpr(&self, pos: &Pos, e: RecExpr<L>) -> RecExpr<L> {
        if let Some((next, subpos)) = pos.split_first() {
            let mut children = self.children.clone();
            let child_idx = *next as usize;
            children[child_idx] = children[child_idx].replace_subexpr(&subpos.to_vec(), e);
            RecExpr { node: self.node.clone(), children }
        } else {
            e
        }
    }
}