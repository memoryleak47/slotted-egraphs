#![allow(unused)]
#![allow(non_snake_case)]

use slotted_egraphs::*;
use crate::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum FghENode {
    F(Slot, Slot),
    G(Slot, Slot),
    H(Slot, Slot),
}

impl Language for FghENode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        match self {
            FghENode::F(x, y) => vec![x, y],
            FghENode::G(x, y) => vec![x, y],
            FghENode::H(x, y) => vec![x, y],
        }
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        match self {
            FghENode::F(x, y) => vec![x, y],
            FghENode::G(x, y) => vec![x, y],
            FghENode::H(x, y) => vec![x, y],
        }
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        vec![]
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            FghENode::F(x, y) => (String::from("f"), vec![Child::Slot(x), Child::Slot(y)]),
            FghENode::G(x, y) => (String::from("g"), vec![Child::Slot(x), Child::Slot(y)]),
            FghENode::H(x, y) => (String::from("h"), vec![Child::Slot(x), Child::Slot(y)]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("f", [Child::Slot(x), Child::Slot(y)]) => Some(FghENode::F(*x, *y)),
            ("g", [Child::Slot(x), Child::Slot(y)]) => Some(FghENode::G(*x, *y)),
            ("h", [Child::Slot(x), Child::Slot(y)]) => Some(FghENode::H(*x, *y)),
            _ => None,
        }
    }
}

#[test]
fn fgh_test() {
    let eg: &mut EGraph<FghENode> = &mut EGraph::new();
    equate("(f s1 s2)", "(g s2 s1)", eg);
    equate("(g s1 s2)", "(h s1 s2)", eg);
    eg.dump();
    explain("(f s1 s2)", "(h s2 s1)", eg);
}
