use crate::*;

pub trait Analysis<L: Language> {
    type Data: Eq;

    fn make(enode: &L) -> Self::Data;
    fn merge(l: &Self::Data, r: &Self::Data) -> Self::Data;
}
