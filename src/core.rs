use crate::*;

trait CoreComponent<L> {
    fn construct(l: &L) -> Self;
}

struct CoreEClass<L: Language, C: CoreComponent<L>> {
    nodes: HashMap<Node<L>, SlotMap>,
    c: C,
}

// The CoreEGraph enforces that `hashcons`, `usages` and `nodes` agree with each other.
// Thus it also enforces that every shape exists uniquely, due to the hashcons.
// TODO so the CoreEGraph understands shapes and equivalence up to renaming?
pub struct CoreEGraph<L: Language, C: CoreComponent<L>> {
    suf: SlottedUF<CoreEClass<L, C>>,
    hashcons: HashMap<Node<L>, AppliedId>,
    usages: HashMap<Id, HashSet<Node<L>>>,
}

impl<L: Language, C: CoreComponent<L>> CoreEGraph<L, C> {
    pub fn new() -> Self {
        CoreEGraph {
            suf: SlottedUF::new(),
            hashcons: Default::default(),
            usages: Default::default(),
        }
    }

    // if the shape already exists, reject it, and return the collision.
    pub fn add(&mut self, l: Node<L>, a: AppliedId) -> Option<(Node<L>, AppliedId, AppliedId)> {
        todo!()
    }

    // returns the AppliedId that contained `l`, if it existed.
    pub fn remove(&mut self, l: Node<L>) -> Option<AppliedId> {
        todo!()
    }
}
