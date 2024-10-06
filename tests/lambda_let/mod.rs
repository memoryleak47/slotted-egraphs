use crate::*;

mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

mod tst;
pub use tst::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum LetENode {
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
    Let(Slot, AppliedId, AppliedId),
}

impl Language for LetENode {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            LetENode::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            },
            LetENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            LetENode::Var(x) => {
                out.push(x);
            }
            LetENode::Let(x, t, b) => {
                out.push(x);
                out.extend(t.slots_mut());
                out.extend(b.slots_mut());
            }
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            LetENode::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            },
            LetENode::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            LetENode::Var(x) => {
                out.push(x);
            }
            LetENode::Let(x, t, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
                out.extend(t.slots_mut());
            }
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            LetENode::Lam(_, b) => vec![b],
            LetENode::App(l, r) => vec![l, r],
            LetENode::Var(_) => vec![],
            LetENode::Let(_, t, b) => vec![t, b],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            LetENode::Lam(s, a) => (String::from("lam"), vec![Child::Slot(s), Child::AppliedId(a)]),
            LetENode::App(l, r) => (String::from("app"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
            LetENode::Var(s) => (String::from("var"), vec![Child::Slot(s)]),
            LetENode::Let(s, t, b) => (String::from("let"), vec![Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("lam", [Child::Slot(s), Child::AppliedId(a)]) => Some(LetENode::Lam(*s, a.clone())),
            ("app", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(LetENode::App(l.clone(), r.clone())),
            ("var", [Child::Slot(s)]) => Some(LetENode::Var(*s)),
            ("let", [Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]) => Some(LetENode::Let(*s, t.clone(), b.clone())),
            _ => None,
        }
    }
}


use std::fmt::*;

impl Debug for LetENode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            LetENode::Lam(s, b) => write!(f, "(lam {s:?} {b:?})"),
            LetENode::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            LetENode::Var(s) => write!(f, "{s:?}"),
            LetENode::Let(x, t, b) => write!(f, "(let {x:?} {t:?} {b:?})"),
        }
    }
}
