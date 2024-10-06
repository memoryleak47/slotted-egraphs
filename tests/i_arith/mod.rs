#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;

pub mod build;

mod tst;
pub use tst::*;

mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArithENode {
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

impl Language for ArithENode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            ArithENode::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            }
            ArithENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ArithENode::Var(x) => {
                out.push(x);
            }
            ArithENode::Let(x, t, b) => {
                out.push(x);
                out.extend(t.slots_mut());
                out.extend(b.slots_mut());
            }

            ArithENode::Add(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ArithENode::Mul(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ArithENode::Number(_) => {}
            ArithENode::Symbol(_) => {}
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            ArithENode::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            }
            ArithENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ArithENode::Var(x) => {
                out.push(x);
            }
            ArithENode::Let(x, t, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
                out.extend(t.slots_mut());
            }
            ArithENode::Add(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ArithENode::Mul(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ArithENode::Number(_) => {}
            ArithENode::Symbol(_) => {}
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            ArithENode::Lam(_, b) => vec![b],
            ArithENode::App(l, r) => vec![l, r],
            ArithENode::Var(_) => vec![],
            ArithENode::Let(_, t, b) => vec![t, b],
            ArithENode::Add(l, r) => vec![l, r],
            ArithENode::Mul(l, r) => vec![l, r],
            ArithENode::Number(_) => vec![],
            ArithENode::Symbol(_) => vec![],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            ArithENode::Lam(s, a) => (String::from("lam"), vec![Child::Slot(s), Child::AppliedId(a)]),
            ArithENode::App(l, r) => (String::from("app"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
            ArithENode::Var(s) => (String::from("var"), vec![Child::Slot(s)]),
            ArithENode::Let(s, t, b) => (String::from("let"), vec![Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]),
            ArithENode::Number(n) => (format!("num_{}", n), vec![]),
            ArithENode::Symbol(s) => (format!("sym_{}", s), vec![]),
            ArithENode::Add(l, r) => (String::from("add"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
            ArithENode::Mul(l, r) => (String::from("mul"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("lam", [Child::Slot(s), Child::AppliedId(a)]) => Some(ArithENode::Lam(*s, a.clone())),
            ("app", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(ArithENode::App(l.clone(), r.clone())),
            ("var", [Child::Slot(s)]) => Some(ArithENode::Var(*s)),
            ("let", [Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]) => Some(ArithENode::Let(*s, t.clone(), b.clone())),
            ("add", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(ArithENode::Add(l.clone(), r.clone())),
            ("mul", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(ArithENode::Mul(l.clone(), r.clone())),
            (op, []) if op.starts_with("num_") => {
                let u: u32 = op[4..].parse().ok()?;
                Some(ArithENode::Number(u))
            },
            (op, []) if op.starts_with("sym_") => {
                let s: Symbol = op[4..].parse().ok()?;
                Some(ArithENode::Symbol(s))
            },
            _ => None,
        }
    }

}


use std::fmt::*;

impl Debug for ArithENode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ArithENode::Lam(s, b) => write!(f, "(lam {s:?} {b:?})"),
            ArithENode::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            ArithENode::Var(s) => write!(f, "{s:?}"),
            ArithENode::Let(x, t, b) => write!(f, "(let {x:?} {t:?} {b:?})"),
            ArithENode::Add(l, r) => write!(f, "(+ {l:?} {r:?})"),
            ArithENode::Mul(l, r) => write!(f, "(* {l:?} {r:?})"),
            ArithENode::Number(i) => write!(f, "{i}"),
            ArithENode::Symbol(i) => write!(f, "symb{i:?}"),
        }
    }
}
