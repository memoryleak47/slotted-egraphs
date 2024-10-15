#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

mod tst;
pub use tst::*;

mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

mod const_prop;
pub use const_prop::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Arith {
    // lambda calculus:
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
    Let(Slot, AppliedId, AppliedId),

    Add(AppliedId, AppliedId),
    Mul(AppliedId, AppliedId),

    // rest:
    Number(u32),
    Symbol(Symbol),
}

impl Language for Arith {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            Arith::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            }
            Arith::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            Arith::Var(x) => {
                out.push(x);
            }
            Arith::Let(x, t, b) => {
                out.push(x);
                out.extend(t.slots_mut());
                out.extend(b.slots_mut());
            }

            Arith::Add(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            Arith::Mul(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            Arith::Number(_) => {}
            Arith::Symbol(_) => {}
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            Arith::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            }
            Arith::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            Arith::Var(x) => {
                out.push(x);
            }
            Arith::Let(x, t, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
                out.extend(t.slots_mut());
            }
            Arith::Add(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            Arith::Mul(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            Arith::Number(_) => {}
            Arith::Symbol(_) => {}
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            Arith::Lam(_, b) => vec![b],
            Arith::App(l, r) => vec![l, r],
            Arith::Var(_) => vec![],
            Arith::Let(_, t, b) => vec![t, b],
            Arith::Add(l, r) => vec![l, r],
            Arith::Mul(l, r) => vec![l, r],
            Arith::Number(_) => vec![],
            Arith::Symbol(_) => vec![],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            Arith::Lam(s, a) => (String::from("lam"), vec![Child::Slot(s), Child::AppliedId(a)]),
            Arith::App(l, r) => (String::from("app"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
            Arith::Var(s) => (String::from("var"), vec![Child::Slot(s)]),
            Arith::Let(s, t, b) => (String::from("let"), vec![Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]),
            Arith::Number(n) => (format!("{}", n), vec![]),
            Arith::Symbol(s) => (format!("{}", s), vec![]),
            Arith::Add(l, r) => (String::from("add"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
            Arith::Mul(l, r) => (String::from("mul"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("lam", [Child::Slot(s), Child::AppliedId(a)]) => Some(Arith::Lam(*s, a.clone())),
            ("app", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(Arith::App(l.clone(), r.clone())),
            ("var", [Child::Slot(s)]) => Some(Arith::Var(*s)),
            ("let", [Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]) => Some(Arith::Let(*s, t.clone(), b.clone())),
            ("add", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(Arith::Add(l.clone(), r.clone())),
            ("mul", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(Arith::Mul(l.clone(), r.clone())),
            (op, []) => {
                if let Ok(u) = op.parse::<u32>() {
                    Some(Arith::Number(u))
                } else {
                    let s: Symbol = op.parse().ok()?;
                    Some(Arith::Symbol(s))
                }
            },
            _ => None,
        }
    }

}


use std::fmt::*;

impl Debug for Arith {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Arith::Lam(s, b) => write!(f, "(lam {s:?} {b:?})"),
            Arith::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            Arith::Var(s) => write!(f, "{s:?}"),
            Arith::Let(x, t, b) => write!(f, "(let {x:?} {t:?} {b:?})"),
            Arith::Add(l, r) => write!(f, "(+ {l:?} {r:?})"),
            Arith::Mul(l, r) => write!(f, "(* {l:?} {r:?})"),
            Arith::Number(i) => write!(f, "{i}"),
            Arith::Symbol(i) => write!(f, "symb{i:?}"),
        }
    }
}
