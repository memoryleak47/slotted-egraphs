use crate::*;

trait CoreComponent<L> {
    fn construct(l: &L) -> Self;
}

struct CoreEClass<L, C: CoreComponent<L>> {
    nodes: HashMap<L, SlotMap>,
    c: C,
}

pub struct CoreEGraph<L: Language, C: CoreComponent<L>> {
    suf: SlottedUF<CoreEClass<L, C>>,
    hashcons: HashMap<L, AppliedId>,
    usages: HashMap<Id, HashSet<L>>,
}

impl<L: Language, C: CoreComponent<L>> CoreEGraph<L, C> {
    pub fn new() -> Self {
        CoreEGraph {
            suf: SlottedUF::new(),
            hashcons: Default::default(),
            usages: Default::default(),
        }
    }

    pub fn add(&mut self, l: L) -> AppliedId {
        todo!()
    }
}
