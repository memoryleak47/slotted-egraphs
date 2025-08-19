use crate::*;

/// E-Graph Analysis allows you to propagate information upwards through the E-Graph.
pub trait Analysis<L: Language>: Sized {
    type Data: Eq + Clone;
    fn make(eg: &EGraph<L, Self>, enode: &L) -> Self::Data;
    fn merge(l: Self::Data, r: Self::Data) -> Self::Data;
}

impl<L: Language> Analysis<L> for () {
    type Data = ();
    fn make(_eg: &EGraph<L, Self>, _: &L) {}
    fn merge(_l: (), _r: ()) -> () {}
}
