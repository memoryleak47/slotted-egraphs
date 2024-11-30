mod access;
pub use access::*;

mod slot;
pub use slot::*;

mod slotmap;
pub use slotmap::*;

mod slotmap_like;
pub use slotmap_like::*;

mod applied;
pub use applied::*;

mod apply_map;
pub use apply_map::*;

mod rename;
pub use rename::*;

mod suf;
pub use suf::*;

mod lang;
pub use lang::*;

mod group;
pub use group::*;

mod segraph;
pub use segraph::*;

pub type HashMap<K, V> = fnv::FnvHashMap<K, V>;
pub type HashSet<T> = fnv::FnvHashSet<T>;
pub use std::collections::hash_map::Entry;
pub use std::ops::*;
pub use std::hash::Hash;

const CHECKS: bool = true;
