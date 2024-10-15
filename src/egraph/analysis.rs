use crate::*;

pub trait Analysis<L: Language>: Eq + Clone {
    fn make(enode: &L) -> Self;
    fn merge(l: Self, r: Self) -> Self;
}

impl<L: Language> Analysis<L> for () {
    fn make(_: &L) {}
    fn merge(l: (), r: ()) -> () {}
}
