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
        let subsrc = eq.side(/*src:*/ true, symm, graph);
        ctx.update_slot_map(&subsrc, &pos);

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
                let mut subdst = eq.side(/*src:*/ false, symm, graph);
                subdst.apply_slot_map(&ctx.slot_map);
                let dst = ctx.head.replace_subexpr(&pos, subdst);
                let step = Step {
                    dst: dst.clone(), rw_pos: pos, jus: jus.as_ref().unwrap().clone(), back: symm 
                };
                ctx.head = dst;
                vec![step]
            },            
        }
    }
}

impl ProvenEqRaw {

    // The `src` variable indicates whether we want the source of destination of the equation. That 
    // is, the left- or right-hand side modulo `symm`.
    fn side<L: Language, N: Analysis<L>>(&self, src: bool, symm: bool, graph: &EGraph<L, N>) -> RecExpr<L> {
        let Equation { l: lhs, r: rhs } = self.equ();
        let subdst_id = if src ^ symm { lhs } else { rhs };
        graph.get_syn_expr(&subdst_id)
    }
}

impl<L: Language> FlatteningContext<L> {

    fn update_slot_map(&mut self, subsrc: &RecExpr<L>, pos: &Pos) {
        let subhead = self.head.subexpr(pos);
        Self::update_slot_map_core(&mut self.slot_map, subhead, subsrc);
    }

    fn update_slot_map_core(map: &mut HashMap<Slot, Slot>, head: &RecExpr<L>, src: &RecExpr<L>) {
        let (head_op, head_children) = head.node.to_op();
        let (src_op, src_children)   = src.node.to_op();
        
        // `src` and `head` should be syntactically equal up to renaming of slots.
        if head_op != src_op || head_children.len() != src_children.len() {
            panic!("'FlatteningContext.update_slot_map_core' received distinct 'head' and 'src'.")
         }

        // Note: head_children.len() == src_children.len()
        let mut child_idx = 0;
        for idx in 0..head_children.len() {
            match (&head_children[idx], &src_children[idx]) {
                (Child::AppliedId(_), Child::AppliedId(_)) => {
                    Self::update_slot_map_core(map, &head.children[child_idx], &src.children[child_idx]);
                    child_idx += 1;
                },
                (Child::Slot(h), Child::Slot(s)) => {
                    if h != s { map.insert(*s, *h); }
                },
                _ => panic!("'FlatteningContext.update_slot_map_core' found distinct children.")
            }
        }
    }
}

impl<L: Language> RecExpr<L> {

    fn subexpr(&self, pos: &Pos) -> &RecExpr<L> {
        if let Some((next, subpos)) = pos.split_first() {
            Self::subexpr(&self.children[*next as usize], &subpos.to_vec())
        } else {
            self
        }
    }

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

    fn apply_slot_map(&mut self, m: &HashMap<Slot, Slot>) {
        for slot in self.node.all_slot_occurrences_mut().iter_mut() {
            **slot = Self::map_slot(**slot, m); 
        }
        for idx in 0..self.children.len() {
            Self::apply_slot_map(&mut self.children[idx], m);
        }
    }

    // Important: This will loop if the slot map contains a cycle!
    fn map_slot(s: Slot, m: &HashMap<Slot, Slot>) -> Slot {
        if let Some(&new) = m.get(&s) {
            Self::map_slot(new, m)
        } else {
            s
        }
    }
}
