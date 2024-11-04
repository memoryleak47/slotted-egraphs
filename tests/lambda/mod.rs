use crate::*;

mod build;
pub use build::*;

mod my_cost;
pub use my_cost::*;

mod tst;
pub use tst::*;

mod normalize;
pub use normalize::*;

mod realization;
pub use realization::*;

mod subst;
pub use subst::*;

mod step;
pub use step::*;

mod big_step;
pub use big_step::*;

mod lambda_small_step;
pub use lambda_small_step::*;

mod let_small_step;
pub use let_small_step::*;

mod native;
pub use native::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Lambda {
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
    Let(Slot, AppliedId, AppliedId),
}

impl Language for Lambda {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            Lambda::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            },
            Lambda::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            Lambda::Var(x) => {
                out.push(x);
            }
            Lambda::Let(x, t, b) => {
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
            Lambda::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            },
            Lambda::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            Lambda::Var(x) => {
                out.push(x);
            }
            Lambda::Let(x, t, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
                out.extend(t.slots_mut());
            }
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            Lambda::Lam(_, b) => vec![b],
            Lambda::App(l, r) => vec![l, r],
            Lambda::Var(_) => vec![],
            Lambda::Let(_, t, b) => vec![t, b],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            Lambda::Lam(s, a) => (String::from("lam"), vec![Child::Slot(s), Child::AppliedId(a)]),
            Lambda::App(l, r) => (String::from("app"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
            Lambda::Var(s) => (String::from("var"), vec![Child::Slot(s)]),
            Lambda::Let(s, t, b) => (String::from("let"), vec![Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("lam", [Child::Slot(s), Child::AppliedId(a)]) => Some(Lambda::Lam(*s, a.clone())),
            ("app", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(Lambda::App(l.clone(), r.clone())),
            ("var", [Child::Slot(s)]) => Some(Lambda::Var(*s)),
            ("let", [Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]) => Some(Lambda::Let(*s, t.clone(), b.clone())),
            _ => None,
        }
    }
}


use std::fmt::*;

impl Debug for Lambda {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Lambda::Lam(s, b) => write!(f, "(lam {s:?} {b:?})"),
            Lambda::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            Lambda::Var(s) => write!(f, "{s:?}"),
            Lambda::Let(x, t, b) => write!(f, "(let {x:?} {t:?} {b:?})"),
        }
    }
}
