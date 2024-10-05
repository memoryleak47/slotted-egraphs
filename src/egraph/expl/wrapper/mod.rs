// This module is supposed to "wrap" all the types that contain Explanations, to provide an explanation-agnostic API.
// We can later opt-out of explanations by either a feature flag, or type-system arguments.
// We want that all prove_X calls are used somewhere within this wrapper module.

mod perm;
pub use perm::*;

mod applied_id;
pub use applied_id::*;

mod node;
pub use node::*;
