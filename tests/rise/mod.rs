use crate::*;

mod tst;
pub use tst::*;

mod rewrite;
pub use rewrite::*;

mod my_cost;
pub use my_cost::*;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rise {
    // lambda calculus:
    Lam(Slot, AppliedId),
    App(AppliedId, AppliedId),
    Var(Slot),
    Let(Slot, AppliedId, AppliedId),

    // rest:
    Number(u32),
    Symbol(Symbol),
}

impl Language for Rise {
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            Rise::Lam(x, b) => {
                out.push(x);
                out.extend(b.slots_mut());
            }
            Rise::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            Rise::Var(x) => {
                out.push(x);
            }
            Rise::Let(x, t, b) => {
                out.push(x);
                out.extend(t.slots_mut());
                out.extend(b.slots_mut());
            }
            Rise::Number(_) => {}
            Rise::Symbol(_) => {}
        }
        out
    }

    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let mut out = Vec::new();
        match self {
            Rise::Lam(x, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
            }
            Rise::App(l, r) => {
                out.extend(l.slots_mut());
                out.extend(r.slots_mut());
            }
            Rise::Var(x) => {
                out.push(x);
            }
            Rise::Let(x, t, b) => {
                out.extend(b.slots_mut().into_iter().filter(|y| *y != x));
                out.extend(t.slots_mut());
            }
            Rise::Number(_) => {}
            Rise::Symbol(_) => {}
        }
        out
    }

    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId> {
        match self {
            Rise::Lam(_, b) => vec![b],
            Rise::App(l, r) => vec![l, r],
            Rise::Var(_) => vec![],
            Rise::Let(_, t, b) => vec![t, b],
            Rise::Number(_) => vec![],
            Rise::Symbol(_) => vec![],
        }
    }

    fn to_op(&self) -> (String, Vec<Child>) {
        match self.clone() {
            Rise::Lam(s, a) => (String::from("lam"), vec![Child::Slot(s), Child::AppliedId(a)]),
            Rise::App(l, r) => (String::from("app"), vec![Child::AppliedId(l), Child::AppliedId(r)]),
            Rise::Var(s) => (String::from("var"), vec![Child::Slot(s)]),
            Rise::Let(s, t, b) => (String::from("let"), vec![Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]),
            Rise::Number(n) => (format!("{}", n), vec![]),
            Rise::Symbol(s) => (format!("{}", s), vec![]),
        }
    }

    fn from_op(op: &str, children: Vec<Child>) -> Option<Self> {
        match (op, &*children) {
            ("lam", [Child::Slot(s), Child::AppliedId(a)]) => Some(Rise::Lam(*s, a.clone())),
            ("app", [Child::AppliedId(l), Child::AppliedId(r)]) => Some(Rise::App(l.clone(), r.clone())),
            ("var", [Child::Slot(s)]) => Some(Rise::Var(*s)),
            ("let", [Child::Slot(s), Child::AppliedId(t), Child::AppliedId(b)]) => Some(Rise::Let(*s, t.clone(), b.clone())),
            (op, []) => {
                if let Ok(u) = op.parse::<u32>() {
                    Some(Rise::Number(u))
                } else {
                    let s: Symbol = op.parse().ok()?;
                    Some(Rise::Symbol(s))
                }
            },
            _ => None,
        }
    }

}


use std::fmt::*;

impl Debug for Rise {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Rise::Lam(s, b) => write!(f, "(lam {s:?} {b:?})"),
            Rise::App(l, r) => write!(f, "(app {l:?} {r:?})"),
            Rise::Var(s) => write!(f, "{s:?}"),
            Rise::Let(x, t, b) => write!(f, "(let {x:?} {t:?} {b:?})"),
            Rise::Number(i) => write!(f, "{i}"),
            Rise::Symbol(i) => write!(f, "symb{i:?}"),
        }
    }
}
