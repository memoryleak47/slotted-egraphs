mod slot;
pub use slot::*;

mod slotmap;
pub use slotmap::*;

mod registry;
pub use registry::*;

pub type HashMap<K, V> = std::collections::HashMap<K, V>;
pub type HashSet<T> = std::collections::HashSet<T>;

pub const CHECKS: bool = false;

pub struct Id(usize);
pub struct AppliedId(SlotMap, Id);
pub struct Equation(Id, Id, SlotMap);

struct Suf {
    
}

fn main() {
    println!("Hello, world!");
}
