use crate::*;

trait CoreComponent<L> {
    fn construct(l: &L) -> Self;
}

struct CoreEClass<L: Language, C: CoreComponent<L>> {
    nodes: HashMap<Node<L>, SlotMap>,
    c: C,
}

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

    pub fn add(&mut self, l: Node<L>) -> AppliedId where Node<L>: Hash + Eq {
        self.hashcons[&l].clone()
    }
}
