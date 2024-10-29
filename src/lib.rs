#![allow(unused_imports)]

/*!
Slotted e-graphs are a datastructure for representing congruence relations over terms with variables and binders.

For a higher level introduction to slotted e-graphs, consider
* the [talk](https://www.youtube.com/watch?v=4Cg365LVbYg)
* the [pre-print](https://michel.steuwer.info/files/publications/2024/EGRAPHS-2024.pdf)

For an example implementation of a Language with binders in slotted e-graphs,
consider the RISE implementation in [here](https://github.com/memoryleak47/slotted-egraphs/tree/main/tests/rise/mod.rs).
*/

use std::hash::Hash;
use std::fmt::Debug;
use std::error::Error;
use std::sync::Arc;
use std::ops::Deref;

pub(crate) type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub(crate) type HashSet<T> = fnv::FnvHashSet<T>;

// Whether to enable invariant-checks.
#[cfg(feature = "checks")]
const CHECKS: bool = true;
#[cfg(not(feature = "checks"))]
const CHECKS: bool = false;

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

pub(crate) use tracing::instrument;