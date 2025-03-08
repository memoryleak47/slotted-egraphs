use crate::*;

/// E-Graph Analysis allows you to propagate information upwards through the E-Graph.
pub trait Analysis<L: Language>: Eq + Clone {
    fn make(eg: &EGraph<L, Self>, enode: &L) -> Self;
    fn merge(l: Self, r: Self) -> Self;
}

impl<L: Language> Analysis<L> for () {
    fn make(_eg: &EGraph<L, Self>, _: &L) {}
    fn merge(_l: (), _r: ()) -> () {}
}
