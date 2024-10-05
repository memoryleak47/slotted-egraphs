use crate::*;

// We write this as pattern[subst] for short.
pub fn pattern_subst<L: Language>(eg: &mut EGraph<L>, pattern: &Pattern<L>, subst: &Subst) -> AppliedId {
    match &pattern.node {
        ENodeOrPVar::ENode(n) => {
            let mut n = n.clone();
            let mut refs: Vec<&mut _> = n.applied_id_occurences_mut();
            assert_eq!(pattern.children.len(), refs.len());
            for i in 0..refs.len() {
                *(refs[i]) = pattern_subst(eg, &pattern.children[i], subst);
            }
            eg.add_syn(n)
        },
        ENodeOrPVar::PVar(v) => {
            subst[v].clone()
        },
    }
}

// TODO maybe move into EGraph API?
pub fn lookup_rec_expr<L: Language>(re: &RecExpr<L>, eg: &EGraph<L>) -> Option<AppliedId> {
    let mut n = re.node.clone();
    let mut refs: Vec<&mut AppliedId> = n.applied_id_occurences_mut();
    assert_eq!(re.children.len(), refs.len());
    for i in 0..refs.len() {
        *(refs[i]) = lookup_rec_expr(&re.children[i], eg)?;
    }
    eg.lookup(&n)
}

pub fn pattern_to_re<L: Language>(pat: &Pattern<L>) -> RecExpr<L> {
    let ENodeOrPVar::ENode(n) = &pat.node else { panic!() };
    let children: Vec<RecExpr<L>> = pat.children.iter().map(|x| pattern_to_re(x)).collect();
    RecExpr {
        node: n.clone(),
        children,
    }
}
