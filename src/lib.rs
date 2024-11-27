mod suf;
pub use suf::*;

mod types;
pub use types::*;

mod lang;
pub use lang::*;

pub type Map<K, V> = fnv::FnvHashMap<K, V>;
pub type Set<T> = fnv::FnvHashSet<T>;
