#![allow(unused_imports)]

mod normalize;
pub use normalize::*;

mod step;
pub use step::*;

mod parse;
pub use parse::*;

mod display;
pub use display::*;

mod realization;
pub use realization::*;

pub mod build;

mod tst;
pub use tst::*;

mod gen;
pub use gen::*;

type HashSet<T> = std::collections::BTreeSet<T>;
type HashMap<K, V> = std::collections::BTreeMap<K, V>;

#[derive(Clone)]
pub enum Ast {
    Lam(String, Box<Ast>),
    App(Box<Ast>, Box<Ast>),
    Var(String),
}

impl Ast {
    pub fn size(&self) -> usize {
        match self {
            Ast::Lam(_, b) => b.size() + 1,
            Ast::App(l, r) => l.size() + r.size() + 1,
            Ast::Var(_) => 1,
        }
    }
}
