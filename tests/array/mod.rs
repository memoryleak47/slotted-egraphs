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
pub enum Array {
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
    Let(Slot, AppliedId, AppliedId),

    Symbol(Symbol),
}

impl Language for Array {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            Array::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            }
            Array::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            Array::Var(x) => {
                out.push(x);
            }
            Array::Let(x, t, b) => {
                out.push(x);
                out.extend(t.slots_mut());
                out.extend(b.slots_mut());
            }
            Array::Symbol(_) => {}
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            Array::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            }
            Array::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            Array::Var(x) => {
                out.push(x);
            }
            Array::Let(x, t, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
                out.extend(t.slots_mut());
            }
            Array::Symbol(_) => {}
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            Array::Lam(_, b) => vec![b],
            Array::App(l, r) => vec![l, r],
            Array::Var(_) => vec![],
            Array::Let(_, t, b) => vec![t, b],
            Array::Symbol(_) => vec![],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            Array::Lam(s, a) => (String::from("lam"), vec![Child::Slot(s), Child::AppliedId(a)]),
            Array::App(l, r) => (String::from("app"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
            Array::Var(s) => (String::from("var"), vec![Child::Slot(s)]),
            Array::Let(s, t, b) => (String::from("let"), vec![Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]),
            Array::Symbol(s) => (s.to_string(), vec![]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("lam", [Child::Slot(s), Child::AppliedId(a)]) => Some(Array::Lam(*s, a.clone())),
            ("app", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(Array::App(l.clone(), r.clone())),
            ("var", [Child::Slot(s)]) => Some(Array::Var(*s)),
            ("let", [Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]) => Some(Array::Let(*s, t.clone(), b.clone())),
            (op, []) => {
                let s: Symbol = op.parse().ok()?;
                Some(Array::Symbol(s))
            },
            _ => None,
        }
    }

}
