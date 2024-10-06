use crate::*;

mod big_step;
pub use big_step::*;

mod subst;
pub use subst::*;

mod small_step;
pub use small_step::*;

mod parse;
pub use parse::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ENode {
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
}

impl Language for ENode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            ENode::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            },
            ENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ENode::Var(x) => {
                out.push(x);
            }
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            ENode::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            },
            ENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            ENode::Var(x) => {
                out.push(x);
            }
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            ENode::Lam(_, b) => vec![b],
            ENode::App(l, r) => vec![l, r],
            ENode::Var(_) => vec![],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            ENode::Lam(s, a) => (String::from("lam"), vec![Child::Slot(s), Child::AppliedId(a)]),
            ENode::App(l, r) => (String::from("app"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
            ENode::Var(s) => (String::from("var"), vec![Child::Slot(s)]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("lam", [Child::Slot(s), Child::AppliedId(a)]) => Some(ENode::Lam(*s, a.clone())),
            ("app", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(ENode::App(l.clone(), r.clone())),
            ("var", [Child::Slot(s)]) => Some(ENode::Var(*s)),
            _ => None,
        }
    }

}

use std::fmt::*;

impl Debug for ENode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ENode::Lam(s, b) => write!(f, "(lam {s:?} {b:?})"),
            ENode::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            ENode::Var(s) => write!(f, "{s:?}"),
        }
    }
}
