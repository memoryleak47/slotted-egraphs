#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

mod tst;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SymbolENode {
    pub op: Symbol,
    pub children: Vec<AppliedId>,
}

impl Language for SymbolENode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        Vec::new()
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        Vec::new()
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        self.children.iter_mut().collect()
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        (self.op.to_string(), self.children.iter().cloned().map(Child::AppliedId).collect())
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        Some(SymbolENode {
            op: Symbol::from(op),
            children: children.iter().map(|x| {
                let Child::AppliedId(i) = x else { panic!() };
                i.clone()
            }).collect(),
        })
    }
}


#[test]
fn symbol_lang_parse_roundtrip() {
    let s = "(foo 32 (❤ 32 (fooz ok)) (nice Σ))";

    let x: RecExpr<SymbolENode> = RecExpr::parse(s).unwrap();
    assert_eq!(s, &*x.to_string());
}
