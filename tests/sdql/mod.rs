#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

mod rewrite;
pub use rewrite::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Sdql {
    Lam(Slot, AppliedId),
    Var(Slot),
    Sing(AppliedId, AppliedId),
    Sum(Slot, Slot, /*range: */AppliedId, /*body: */ AppliedId),
}

impl Language for Sdql {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            Sdql::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            }
            Sdql::Var(x) => {
                out.push(x);
            }
            Sdql::Sing(x, y) => {
                out.extend(x.slots_mut());
                out.extend(y.slots_mut());
            }
            Sdql::Sum(k, v, r, b) => {
                out.push(k);
                out.push(v);
                out.extend(r.slots_mut());
                out.extend(b.slots_mut());
            }
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            Sdql::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));

            }
            Sdql::Var(x) => {
                out.push(x);
            }
            Sdql::Sing(x, y) => {
                out.extend(x.slots_mut());
                out.extend(y.slots_mut());
            }
            Sdql::Sum(k, v, r, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != k && *y != v));
                out.extend(r.slots_mut());
            }
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            Sdql::Lam(_, y) => vec![y],
            Sdql::Var(_) => vec![],
            Sdql::Sing(x, y) => vec![x, y],
            Sdql::Sum(_, _, r, b) => vec![r, b],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            Sdql::Lam(s, a) => (String::from("lambda"), vec![Child::Slot(s), Child::AppliedId(a)]),
            Sdql::Var(s) => (String::from("var"), vec![Child::Slot(s)]),
            Sdql::Sing(x, y) => (String::from("sing"), vec![Child::AppliedId(x), Child::AppliedId(y)]),
            Sdql::Sum(k, v, r, b) => (String::from("sum"), vec![Child::Slot(k), Child::Slot(v), Child::AppliedId(r), Child::AppliedId(b)]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("lambda", [Child::Slot(s), Child::AppliedId(a)]) => Some(Sdql::Lam(*s, a.clone())),
            ("var", [Child::Slot(s)]) => Some(Sdql::Var(*s)),
            ("sing", [Child::AppliedId(x), Child::AppliedId(y)]) => Some(Sdql::Sing(x.clone(), y.clone())),
            ("sum", [Child::Slot(k), Child::Slot(v), Child::AppliedId(r), Child::AppliedId(b)]) => Some(Sdql::Sum(*k, *v, r.clone(), b.clone())),
            _ => None,
        }
    }
}
