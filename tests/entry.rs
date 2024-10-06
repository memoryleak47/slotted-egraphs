pub use std::hash::Hash;
pub use slotted_egraphs::*;

pub use symbol_table::GlobalSymbol as Symbol;

mod lamcalc;
pub use lamcalc::*;

mod i_arith;
pub use i_arith::*;

mod i_array;
pub use i_array::*;

mod i_lambda;
pub use i_lambda::*;

mod i_let;
pub use i_let::*;

mod i_main;
pub use i_main::*;

mod i_rise;
pub use i_rise::*;

mod i_symbol;
pub use i_symbol::*;

pub fn singleton_set<T: Eq + Hash>(t: T) -> HashSet<T> {
    [t].into_iter().collect()
}
