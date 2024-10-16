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
