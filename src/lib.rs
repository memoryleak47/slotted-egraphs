#![allow(unused_imports)]

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

pub mod pattern;
pub(crate) use pattern::*;

mod group;
use group::*;
