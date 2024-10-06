#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

mod parse;
pub use parse::*;

mod tst;

// This is a close-as possible to SymbolLang to be comparable with https://github.com/Bastacyclop/egg-sketches/blob/main/tests/maps.rs
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ArrayENode {
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
    Let(Slot, AppliedId, AppliedId),

    Symbol(Symbol),
}

impl Language for ArrayENode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            ArrayENode::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            }
            ArrayENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ArrayENode::Var(x) => {
                out.push(x);
            }
            ArrayENode::Let(x, t, b) => {
                out.push(x);
                out.extend(t.slots_mut());
                out.extend(b.slots_mut());
            }
            ArrayENode::Symbol(_) => {}
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            ArrayENode::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            }
            ArrayENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ArrayENode::Var(x) => {
                out.push(x);
            }
            ArrayENode::Let(x, t, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
                out.extend(t.slots_mut());
            }
            ArrayENode::Symbol(_) => {}
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            ArrayENode::Lam(_, b) => vec![b],
            ArrayENode::App(l, r) => vec![l, r],
            ArrayENode::Var(_) => vec![],
            ArrayENode::Let(_, t, b) => vec![t, b],
            ArrayENode::Symbol(_) => vec![],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            ArrayENode::Lam(s, a) => (String::from("lam"), vec![Child::Slot(s), Child::AppliedId(a)]),
            ArrayENode::App(l, r) => (String::from("app"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
            ArrayENode::Var(s) => (String::from("var"), vec![Child::Slot(s)]),
            ArrayENode::Let(s, t, b) => (String::from("let"), vec![Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]),
            ArrayENode::Symbol(s) => (s.to_string(), vec![]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("lam", [Child::Slot(s), Child::AppliedId(a)]) => Some(ArrayENode::Lam(*s, a.clone())),
            ("app", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(ArrayENode::App(l.clone(), r.clone())),
            ("var", [Child::Slot(s)]) => Some(ArrayENode::Var(*s)),
            ("let", [Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]) => Some(ArrayENode::Let(*s, t.clone(), b.clone())),
            (op, []) => {
                let s: Symbol = op.parse().ok()?;
                Some(ArrayENode::Symbol(s))
            },
            _ => None,
        }
    }

}
