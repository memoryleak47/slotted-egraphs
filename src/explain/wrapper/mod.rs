// This module is supposed to "wrap" all the types that contain Explanations, to provide an explanation-agnostic API.
// We can later opt-out of explanations by either a feature flag, or type-system arguments.
// We want that all prove_X calls are used somewhere within this wrapper module.

// We will for now use @ghost to annotate code that should be excluded if explanations are off.

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

mod macros {
    #[cfg(feature = "explanations")]
    macro_rules! ghost {
        ($a: expr) => {
            $a
        };
    }

    #[cfg(not(feature = "explanations"))]
    macro_rules! ghost {
        ($a: expr) => {
            ()
        };
    }

    pub(crate) use ghost;
}
pub(crate) use macros::ghost;
