use crate::*;

use std::cmp::Ordering;

// Takes the `Ord` from U, but reverses it.
#[derive(PartialEq, Eq, Debug)]
pub struct WithOrdRev<T: Eq, U: Ord>(pub T, pub U);

impl<T: Eq, U: Ord> PartialOrd for WithOrdRev<T, U> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // It's the other way around, because we want a min-heap!
        other.1.partial_cmp(&self.1)
    }
}
impl<T: Eq, U: Ord> Ord for WithOrdRev<T, U> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}
