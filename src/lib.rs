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

pub mod types;
pub(crate) use types::*;

mod parse;
pub(crate) use parse::*;

pub mod lang;
pub(crate) use lang::*;

pub mod slotmap;
pub(crate) use slotmap::*;

pub mod explain;
pub(crate) use explain::*;

mod debug;

mod egraph;
pub use egraph::*;

pub mod extract;
pub(crate) use extract::*;

pub mod rewrite;
pub(crate) use rewrite::*;

mod group;
use group::*;
