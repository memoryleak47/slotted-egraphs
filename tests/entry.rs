pub use std::hash::Hash;
pub use slotted_egraphs::*;

pub use symbol_table::GlobalSymbol as Symbol;

mod lamcalc;
pub use lamcalc::*;

mod langs;
pub use langs::*;

mod small;
pub use small::*;

pub fn singleton_set<T: Eq + Hash>(t: T) -> HashSet<T> {
    [t].into_iter().collect()
}
