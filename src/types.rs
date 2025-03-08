use crate::*;

/// Ids identify e-classes.
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id(pub usize);

/// AppliedIds are invocations of e-classes.
///
/// Recall that in slotted egraphs, e-classes have arguments - and in order to talk about the set of terms in an e-class, you always need to invocate your e-class using a bunch of arguments.
/// This "invocation" is what an AppliedId represents. The [Id] part identifies an e-class, and the [SlotMap] is used to map the argument-slots of the e-class to the values that you put into them.
#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct AppliedId {
    pub id: Id,

    // m is always a bijection!
    // m maps the slots from `id` (be it ENode::slots() in a RecExpr, or EGraph::slots(Id) for eclasses) to the slots that we insert into it.
    // m.keys() == id.slots
    pub m: SlotMap,
}

/// A "term" or "expression" from some given [Language] L.
// The AppliedIds in `node` are ignored (any typically set to AppliedId::null()). They are replaced by the children RecExpr.
// A non-fancy version of RecExpr that uses the slots as "names".
#[derive(Clone, PartialEq, Eq)]
pub struct RecExpr<L: Language> {
    pub node: L,
    pub children: Vec<RecExpr<L>>,
}

impl AppliedId {
    pub fn new(id: Id, m: SlotMap) -> Self {
        let s = AppliedId { id, m };
        if CHECKS {
            s.check();
        }
        s
    }

    pub(crate) fn check(&self) {
        assert!(self.m.is_bijection());
    }

    #[track_caller]
    pub fn apply_slotmap(&self, m: &SlotMap) -> AppliedId {
        if CHECKS {
            assert!(
                m.keys().is_superset(&self.slots()),
                "AppliedId::apply_slotmap: The SlotMap doesn't map all free slots!"
            );
        }
        self.apply_slotmap_partial(m)
    }

    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    pub fn apply_slotmap_partial(&self, m: &SlotMap) -> AppliedId {
        AppliedId::new(self.id, self.m.compose_partial(m))
    }

    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    pub fn apply_slotmap_fresh(&self, m: &SlotMap) -> AppliedId {
        AppliedId::new(self.id, self.m.compose_fresh(m))
    }

    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    pub fn slots(&self) -> SmallHashSet<Slot> {
        self.m.values()
    }

    // ordered!
    pub fn slots_mut(&mut self) -> Vec<&mut Slot> {
        self.m.values_mut().collect()
    }

    pub fn null() -> Self {
        AppliedId {
            id: Id(0),
            m: SlotMap::new(),
        }
    }
}
