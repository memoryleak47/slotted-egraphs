use crate::*;

pub trait Analysis<L: Language>: Eq + Clone {
    fn make(eg: &EGraph<L, Self>, enode: &L) -> Self;
    fn merge(l: Self, r: Self) -> Self;
}

impl<L: Language> Analysis<L> for () {
    fn make(eg: &EGraph<L, Self>, _: &L) {}
    fn merge(l: (), r: ()) -> () {}
}
