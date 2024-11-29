mod slot;
pub use slot::*;

mod slotmap;
pub use slotmap::*;

mod applied;
pub use applied::*;

mod suf;
pub use suf::*;

mod id;
pub use id::*;

mod lang;
pub use lang::*;

mod group;
pub use group::*;

pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;
pub use std::ops::*;
pub use std::hash::Hash;

const CHECKS: bool = true;
