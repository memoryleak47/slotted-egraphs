use crate::*;

trait Component<L: Language> {
    fn construct(l: &Node<L>) -> Self;
}

struct SEClass<L: Language, C: Component<L>> {
    nodes: HashMap<Node<L>, SlotMap>,
    c: C,
}

// The API of this module enforces that `hashcons`, `usages` and `nodes` agree with each other.
// Thus it also enforces that every shape exists uniquely, due to the hashcons.
// TODO: decide whether shape computation belongs here.
pub struct SEGraph<L: Language, C: Component<L>> {
    suf: SUF<SEClass<L, C>>,
    hashcons: HashMap<Node<L>, AppliedId>,
    usages: HashMap<Id, HashSet<Node<L>>>,
}

impl<L: Language, C: Component<L>> SEGraph<L, C> {
    pub fn new() -> Self {
        SEGraph {
            suf: SUF::new(),
            hashcons: Default::default(),
            usages: Default::default(),
        }
    }

    // These functions do not do shape computation! It needs to be done prior.

    // if the shape already exists, reject it, and return the collision.
    pub(in crate::segraph) fn add_shape(&mut self, l: Node<L>, a: AppliedId) -> Option<(Node<L>, AppliedId, AppliedId)> where Node<L>: Hash + Eq + Clone {
        match self.hashcons.entry(l) {
            Entry::Occupied(e) => {
                Some((e.key().clone(), e.get().clone(), a))
            },
            Entry::Vacant(e) => {
                e.insert(a);
                None
            },
        }
    }

    // returns the AppliedId that contained `l`, if it existed.
    pub(in crate::segraph) fn remove_shape(&mut self, l: &Node<L>) -> Option<AppliedId> where Node<L>: Hash + Eq {
        self.hashcons.remove(l)
    }
}
