// This module is supposed to "wrap" all the types that contain Explanations, to provide an explanation-agnostic API.
// We can later opt-out of explanations by either a feature flag, or type-system arguments.
// We want that all prove_X calls are used somewhere within this wrapper module.

// We will for now use @ghost to annotate code that should be excluded if explanations are off.

use std::marker::PhantomData;

mod perm;
pub use perm::*;

mod applied_id;
pub use applied_id::*;

mod node;
pub use node::*;

mod source_node;
pub use source_node::*;

mod contains;
pub use contains::*;

#[cfg(feature = "explanations_tmp")]
pub type Ghost<T> = T;

#[cfg(not(feature = "explanations_tmp"))]
pub type Ghost<T> = PhantomData<T>;
