use crate::*;

trait Component<L: Language> {
    fn construct(l: &Shape<L>) -> Self;
}

struct SEClass<L: Language, C: Component<L>> {
    nodes: ApplyMap<Shape<L>, SlotMap>,
    c: C,
}

// The API of this module enforces that `hashcons`, `usages` and `nodes` agree with each other.
// Thus it also enforces that every shape exists uniquely, due to the hashcons.
//
// invariants:
// - hashcons[sh] represents sh
// - i represents classes[i].nodes[sh] * sh
// - usages[i].contains(sh) <-> sh in hashcons && sh references i
//
// Note: the bijection in the hashcons goes from Id    to Shape,
// while the bijection in the nodes    goes from Shape to Id.
pub struct SEGraph<L: Language, C: Component<L>> {
    suf: SUF<SEClass<L, C>>,
    hashcons: ApplyMap<Shape<L>, Id>,
    usages: HashMap<Id, HashSet<Shape<L>>>,
}

impl<L: Language, C: Component<L>> SEGraph<L, C> {
    pub fn new() -> Self {
        SEGraph {
            suf: SUF::new(),
            hashcons: ApplyMap::new(),
            usages: Default::default(),
        }
    }

    // These functions do not do shape computation! It needs to be done prior.

    // if the shape already exists, reject it, and return the collision.
    pub(in crate::segraph) fn add_shape(&mut self, sh: Shape<L>, x: AppliedId) -> Option<AppliedId> where Shape<L>: Hash + Eq + Clone {
        match self.hashcons.insert_raw(sh, x) {
            Some(e) => Some(e),

            None => {
                // TODO update usages and nodes.

                None
            }
        }
    }

    // returns the AppliedId that contained `sh`, if it existed.
    pub(in crate::segraph) fn remove_shape(&mut self, sh: &Shape<L>) -> Option<AppliedId> where Shape<L>: Hash + Eq {
        // self.hashcons.remove(sh);
        todo!()
    }
}
