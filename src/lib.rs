#![allow(unused_imports)]

use std::hash::Hash;
use std::fmt::Debug;
use std::error::Error;
use std::sync::Arc;
use std::ops::Deref;

pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;

// Whether to enable invariant-checks.
#[cfg(feature = "checks")]
const CHECKS: bool = true;
#[cfg(not(feature = "checks"))]
const CHECKS: bool = false;

mod types;
pub use types::*;

mod parse;
pub use parse::*;

mod lang;
pub use lang::*;

mod tst;
pub use tst::*;

mod tst2;
pub use tst2::*;

mod slotmap;
pub use slotmap::*;

mod debug;

mod egraph;
pub use egraph::*;

mod extract;
pub use extract::*;

mod pattern;
pub use pattern::*;

mod group;
use group::*;
