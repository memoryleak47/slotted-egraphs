#![allow(unused_imports)]

/*!
Slotted e-graphs are a datastructure for representing congruence relations over terms with variables and binders.

For a higher level introduction to slotted e-graphs, consider
* the [talk](https://www.youtube.com/watch?v=4Cg365LVbYg)
* the [pre-print](https://michel.steuwer.info/files/publications/2024/EGRAPHS-2024.pdf)

For an example implementation of a Language with binders in slotted e-graphs,
consider the RISE implementation in [here](https://github.com/memoryleak47/slotted-egraphs/tree/main/tests/rise/mod.rs).
*/

use std::error::Error;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Deref;
use std::sync::Arc;

#[doc(hidden)]
pub type HashMap<K, V> = rustc_hash::FxHashMap<K, V>;
#[doc(hidden)]
pub type HashSet<T> = rustc_hash::FxHashSet<T>;

pub type SmallHashSet<T> = vec_collections::VecSet<[T; 8]>;
pub type SmallHashMap<T> = vec_collections::VecMap<[(T, T); 8]>;
pub use vec_collections::AbstractVecMap;
pub use vec_collections::AbstractVecSet;

pub use symbol_table::GlobalSymbol as Symbol;

// Whether to enable invariant-checks.
#[cfg(feature = "checks")]
const CHECKS: bool = true;
#[cfg(not(feature = "checks"))]
const CHECKS: bool = false;

pub use slotted_egraphs_derive::define_language;

mod slot;
pub use slot::*;

mod types;
pub use types::*;

mod parse;
pub(crate) use parse::*;

mod lang;
pub use lang::*;

mod slotmap;
pub use slotmap::*;

mod explain;
pub use explain::*;

mod debug;

mod egraph;
pub use egraph::*;

mod extract;
pub use extract::*;

mod rewrite;
pub use rewrite::*;

mod group;
use group::*;

mod run;
pub use run::*;
