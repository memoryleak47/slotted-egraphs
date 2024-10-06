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
pub enum RiseENode {
    // lambda calculus:
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
    Let(Slot, AppliedId, AppliedId),

    // rest:
    Number(u32),
    Symbol(Symbol),
}

impl Language for RiseENode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            RiseENode::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            }
            RiseENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            RiseENode::Var(x) => {
                out.push(x);
            }
            RiseENode::Let(x, t, b) => {
                out.push(x);
                out.extend(t.slots_mut());
                out.extend(b.slots_mut());
            }
            RiseENode::Number(_) => {}
            RiseENode::Symbol(_) => {}
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            RiseENode::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            }
            RiseENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            RiseENode::Var(x) => {
                out.push(x);
            }
            RiseENode::Let(x, t, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
                out.extend(t.slots_mut());
            }
            RiseENode::Number(_) => {}
            RiseENode::Symbol(_) => {}
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            RiseENode::Lam(_, b) => vec![b],
            RiseENode::App(l, r) => vec![l, r],
            RiseENode::Var(_) => vec![],
            RiseENode::Let(_, t, b) => vec![t, b],
            RiseENode::Number(_) => vec![],
            RiseENode::Symbol(_) => vec![],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            RiseENode::Lam(s, a) => (String::from("lam"), vec![Child::Slot(s), Child::AppliedId(a)]),
            RiseENode::App(l, r) => (String::from("app"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
            RiseENode::Var(s) => (String::from("var"), vec![Child::Slot(s)]),
            RiseENode::Let(s, t, b) => (String::from("let"), vec![Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]),
            RiseENode::Number(n) => (format!("num_{}", n), vec![]),
            RiseENode::Symbol(s) => (format!("sym_{}", s), vec![]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("lam", [Child::Slot(s), Child::AppliedId(a)]) => Some(RiseENode::Lam(*s, a.clone())),
            ("app", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(RiseENode::App(l.clone(), r.clone())),
            ("var", [Child::Slot(s)]) => Some(RiseENode::Var(*s)),
            ("let", [Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]) => Some(RiseENode::Let(*s, t.clone(), b.clone())),
            (op, []) if op.starts_with("num_") => {
                let u: u32 = op[4..].parse().ok()?;
                Some(RiseENode::Number(u))
            },
            (op, []) if op.starts_with("sym_") => {
                let s: Symbol = op[4..].parse().ok()?;
                Some(RiseENode::Symbol(s))
            },
            _ => None,
        }
    }

}


use std::fmt::*;

impl Debug for RiseENode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            RiseENode::Lam(s, b) => write!(f, "(lam {s:?} {b:?})"),
            RiseENode::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            RiseENode::Var(s) => write!(f, "{s:?}"),
            RiseENode::Let(x, t, b) => write!(f, "(let {x:?} {t:?} {b:?})"),
            RiseENode::Number(i) => write!(f, "{i}"),
            RiseENode::Symbol(i) => write!(f, "symb{i:?}"),
        }
    }
}
