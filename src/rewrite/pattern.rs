use crate::*;

#[derive(Clone, Hash, PartialEq, Eq)]
// PVar = pattern variable.
pub enum Pattern<L: Language> {
    ENode(L, Vec<Pattern<L>>),
    PVar(String), // ?x
    Let(Box<Pattern<L>>, Box<Pattern<L>>, Box<Pattern<L>>), // Let(x, t, b) means let x=t in b
}

// We write this as pattern[subst] for short.
pub fn pattern_subst<L: Language, N: Analysis<L>>(eg: &mut EGraph<L, N>, pattern: &Pattern<L>, subst: &Subst) -> AppliedId {
    match &pattern {
        Pattern::ENode(n, children) => {
            let mut n = n.clone();
            let mut refs: Vec<&mut _> = n.applied_id_occurences_mut();
            assert_eq!(children.len(), refs.len());
            for i in 0..refs.len() {
                *(refs[i]) = pattern_subst(eg, &children[i], subst);
            }
            eg.add_syn(n)
        },
        Pattern::PVar(v) => {
            subst[v].clone()
        },
        Pattern::Let(..) => panic!(),
    }
}

// TODO maybe move into EGraph API?
pub fn lookup_rec_expr<L: Language, N: Analysis<L>>(re: &RecExpr<L>, eg: &EGraph<L, N>) -> Option<AppliedId> {
    let mut n = re.node.clone();
    let mut refs: Vec<&mut AppliedId> = n.applied_id_occurences_mut();
    assert_eq!(re.children.len(), refs.len());
    for i in 0..refs.len() {
        *(refs[i]) = lookup_rec_expr(&re.children[i], eg)?;
    }
    eg.lookup(&n)
}

pub fn pattern_to_re<L: Language>(pat: &Pattern<L>) -> RecExpr<L> {
    let Pattern::ENode(n, children) = &pat else { panic!() };
    let children: Vec<RecExpr<L>> = children.iter().map(|x| pattern_to_re(x)).collect();
    RecExpr {
        node: n.clone(),
        children,
    }
}

pub fn re_to_pattern<L: Language>(re: &RecExpr<L>) -> Pattern<L> {
    let children: Vec<Pattern<L>> = re.children.iter().map(|x| re_to_pattern(x)).collect();
    Pattern::ENode(
        re.node.clone(),
        children,
    )
}
