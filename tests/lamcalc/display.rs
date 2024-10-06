use crate::lamcalc::*;

use std::fmt::*;

impl Display for Ast {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Ast::Lam(x, b) => write!(f, "(lam {x} {b})"),
            Ast::App(l, r) => write!(f, "(app {l} {r})"),
            Ast::Var(x) => write!(f, "{x}"),
        }
    }
}
