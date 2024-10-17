#![allow(unused)]
#![allow(non_snake_case)]

use slotted_egraphs::*;
use crate::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Fgh {
    F(Slot, Slot),
    G(Slot, Slot),
    H(Slot, Slot),
}

impl Language for Fgh {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        match self {
            Fgh::F(x, y) => vec![x, y],
            Fgh::G(x, y) => vec![x, y],
            Fgh::H(x, y) => vec![x, y],
        }
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        match self {
            Fgh::F(x, y) => vec![x, y],
            Fgh::G(x, y) => vec![x, y],
            Fgh::H(x, y) => vec![x, y],
        }
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        vec![]
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            Fgh::F(x, y) => (String::from("f"), vec![Child::Slot(x), Child::Slot(y)]),
            Fgh::G(x, y) => (String::from("g"), vec![Child::Slot(x), Child::Slot(y)]),
            Fgh::H(x, y) => (String::from("h"), vec![Child::Slot(x), Child::Slot(y)]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("f", [Child::Slot(x), Child::Slot(y)]) => Some(Fgh::F(*x, *y)),
            ("g", [Child::Slot(x), Child::Slot(y)]) => Some(Fgh::G(*x, *y)),
            ("h", [Child::Slot(x), Child::Slot(y)]) => Some(Fgh::H(*x, *y)),
            _ => None,
        }
    }
}

#[test]
fn transitive_symmetry() {
    let eg: &mut EGraph<Fgh> = &mut EGraph::new();
    equate("(f $1 $2)", "(g $2 $1)", eg);
    equate("(g $1 $2)", "(h $1 $2)", eg);
    eg.dump();
    explain("(f $1 $2)", "(h $2 $1)", eg);
}
