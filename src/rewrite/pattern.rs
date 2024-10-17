use crate::*;

pub type Pattern<L> = RecExpr<ENodeOrPVar<L>>;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
// PVar = pattern variable.
pub enum ENodeOrPVar<L: Language> {
    ENode(L),
    PVar(String),
}

impl<L: Language> Language for ENodeOrPVar<L> {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        match self {
            ENodeOrPVar::ENode(x) => x.all_slot_occurences_mut(),
            ENodeOrPVar::PVar(_) => vec![],
        }
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        match self {
            ENodeOrPVar::ENode(x) => x.public_slot_occurences_mut(),
            ENodeOrPVar::PVar(_) => vec![],
        }
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            ENodeOrPVar::ENode(x) => x.applied_id_occurences_mut(),
            ENodeOrPVar::PVar(_) => vec![],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self {
            ENodeOrPVar::ENode(l) => l.to_op(),
            ENodeOrPVar::PVar(s) => (format!("?{}", s), vec![]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        if children.len() == 0 && op.starts_with("?") {
            let var = &op[1..];
            Some(ENodeOrPVar::PVar(var.to_string()))
        } else {
            L::from_op(op, children).map(ENodeOrPVar::ENode)
        }
    }
}

// We write this as pattern[subst] for short.
pub fn pattern_subst<L: Language, N: Analysis<L>>(eg: &mut EGraph<L, N>, pattern: &Pattern<L>, subst: &Subst) -> AppliedId {
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
    let ENodeOrPVar::ENode(n) = &pat.node else { panic!() };
    let children: Vec<RecExpr<L>> = pat.children.iter().map(|x| pattern_to_re(x)).collect();
    RecExpr {
        node: n.clone(),
        children,
    }
}

