mod slot;
pub use slot::*;

mod slotmap;
pub use slotmap::*;

mod registry;
pub use registry::*;

mod group;
pub use group::*;

mod suf;
pub use suf::*;

#[cfg(test)]
mod tst;

pub type HashMap<K, V> = std::collections::HashMap<K, V>;
pub type HashSet<T> = std::collections::HashSet<T>;

pub const CHECKS: bool = false;
