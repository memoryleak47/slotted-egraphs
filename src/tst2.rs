#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TstENode {
    F(Slot, Slot),
    G(Slot, Slot),
    H(Slot, Slot),
}

impl Language for TstENode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        match self {
            TstENode::F(x, y) => vec![x, y],
            TstENode::G(x, y) => vec![x, y],
            TstENode::H(x, y) => vec![x, y],
        }
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        match self {
            TstENode::F(x, y) => vec![x, y],
            TstENode::G(x, y) => vec![x, y],
            TstENode::H(x, y) => vec![x, y],
        }
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        vec![]
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            TstENode::F(x, y) => (String::from("f"), vec![Child::Slot(x), Child::Slot(y)]),
            TstENode::G(x, y) => (String::from("g"), vec![Child::Slot(x), Child::Slot(y)]),
            TstENode::H(x, y) => (String::from("h"), vec![Child::Slot(x), Child::Slot(y)]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("f", [Child::Slot(x), Child::Slot(y)]) => Some(TstENode::F(*x, *y)),
            ("g", [Child::Slot(x), Child::Slot(y)]) => Some(TstENode::G(*x, *y)),
            ("h", [Child::Slot(x), Child::Slot(y)]) => Some(TstENode::H(*x, *y)),
            _ => None,
        }
    }
}
