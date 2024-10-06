pub use std::hash::Hash;
pub use slotted_egraphs::*;

pub use symbol_table::GlobalSymbol as Symbol;

mod lamcalc;
pub use lamcalc::*;

mod small;
pub use small::*;

mod arith;
pub use arith::*;

mod array;
pub use array::*;

mod lambda;
pub use lambda::*;

mod lambda_let;
pub use lambda_let::*;

mod rise;
pub use rise::*;

mod symbol;
pub use symbol::*;

mod var;
pub use var::*;

mod fgh;
pub use fgh::*;


pub fn singleton_set<T: Eq + Hash>(t: T) -> HashSet<T> {
    [t].into_iter().collect()
}
