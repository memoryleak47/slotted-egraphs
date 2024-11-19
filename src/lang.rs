use crate::*;

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct Bind<T> {
    pub slot: Slot,
    pub elem: T,
}

define_language! {
    enum ExampleLanguage {
        Lambda(Bind<AppliedId>) = "lambda",
        App(AppliedId, AppliedId) = "app",
        Var(Slot) = "var",
        // #[substitution_op] Let(AppliedId, Bind<AppliedId>) = "let",
    }
}

#[derive(Debug, Clone)]
/// A child node of some term ("child" in the sense of an AST).
///
/// This type is used for parsing and displaying of your e-node.
/// You only need to implement [Language::to_op] and [Language::from_op] to express, whether your E-Node expects [Slot]s or [AppliedId]s at particular positions.
pub enum Child {
    AppliedId(AppliedId),
    Slot(Slot),
}

/// A trait to define your Language (i.e. your E-Node type).
pub trait Language: Debug + Clone + Hash + Eq {
    /// List the mutable references of all child [Slot]s in your E-Node, in order of occurence.
    fn all_slot_occurences_mut(&mut self) -> Vec<&mut Slot>;

    /// List the mutable references to all *public* child [Slot]s in your E-Node, in order of occurence.
    ///
    /// Public Slots are those, which are visible from the outside of that e-node.
    /// * A typical example would be a `(var $x)` e-node, which has a *public* slot `$x`.
    /// * A typical counter-example would be the `(lam $x body)` e-node, which has a *private* slot `$x`.
    fn public_slot_occurences_mut(&mut self) -> Vec<&mut Slot>;

    /// List the mutable references to all child [AppliedId]s in your E-Node, in the order of occurence.
    fn applied_id_occurences_mut(&mut self) -> Vec<&mut AppliedId>;

    // for parsing and pretty-printing.
    /// Decomposes an E-Node into it's "operator" string and a list of children.
    ///
    /// This function will be used to display your E-Node.
    fn to_op(&self) -> (String, Vec<Child>);

    /// Computes your E-Node from an "operator" string and a list of children.
    ///
    /// This function will be used to parse your E-Node.
    fn from_op(op: &str, children: Vec<Child>) -> Option<Self>;

    #[track_caller]
    #[doc(hidden)]
    fn check(&self) {
        let mut c = self.clone();
        let all: HashSet<*mut Slot> = c.all_slot_occurences_mut().into_iter().map(|x| x as *mut Slot).collect();
        let public: HashSet<*mut Slot> = c.public_slot_occurences_mut().into_iter().map(|x| x as *mut Slot).collect();
        let private: HashSet<*mut Slot> = c.private_slot_occurences_mut().into_iter().map(|x| x as *mut Slot).collect();

        assert!(public.is_disjoint(&private));

        // This also catches errors, where different Slot-addresses have the same slot names. This also counts as a collision!
        let f = |x: Vec<Slot>| x.into_iter().collect::<HashSet<_>>();
        assert!(f(c.public_slot_occurences()).is_disjoint(&f(c.private_slot_occurences())));

        let all2: HashSet<*mut Slot> = public.union(&private).copied().collect();
        assert_eq!(all2, all);
    }


    // generated methods:

    #[doc(hidden)]
    fn private_slot_occurences_mut(&mut self) -> Vec<&mut Slot> {
        let public = self.public_slot_occurences();
        let mut out = self.all_slot_occurences_mut();
        out.retain(|x| !public.contains(x));
        out
    }

    #[doc(hidden)]
    fn all_slot_occurences(&self) -> Vec<Slot> {
        self.clone().all_slot_occurences_mut().into_iter().map(|x| x.clone()).collect()
    }

    #[doc(hidden)]
    fn public_slot_occurences(&self) -> Vec<Slot> {
        self.clone().public_slot_occurences_mut().into_iter().map(|x| x.clone()).collect()
    }

    #[doc(hidden)]
    fn applied_id_occurences(&self) -> Vec<AppliedId> {
        self.clone().applied_id_occurences_mut().into_iter().map(|x| x.clone()).collect()
    }

    #[doc(hidden)]
    fn private_slot_occurences(&self) -> Vec<Slot> {
        self.clone().private_slot_occurences_mut().into_iter().map(|x| x.clone()).collect()
    }

    #[doc(hidden)]
    fn private_slots(&self) -> HashSet<Slot> {
        self.private_slot_occurences().into_iter().collect()
    }

    #[doc(hidden)]
    fn map_applied_ids(&self, mut f: impl FnMut(AppliedId) -> AppliedId) -> Self {
        let mut c = self.clone();
        for x in c.applied_id_occurences_mut() {
            *x = f(x.clone());
        }
        c
    }

    // TODO m.values() might collide with your private slot names.
    // Should we rename our private slots to be safe?
    #[doc(hidden)]
    fn apply_slotmap_partial(&self, m: &SlotMap) -> Self {
        let prv = self.private_slots();

        let mut c = self.clone();
        for x in c.public_slot_occurences_mut() {
            let y = m[*x];

            // If y collides with a private slot, we have a problem.
            if CHECKS {
                assert!(!prv.contains(&y));
            }

            *x = y;
        }
        c
    }


    #[track_caller]
    #[doc(hidden)]
    fn apply_slotmap(&self, m: &SlotMap) -> Self {
        if CHECKS {
            assert!(m.keys().is_superset(&self.slots()), "Language::apply_slotmap: The SlotMap doesn't map all free slots!");
        }
        self.apply_slotmap_partial(m)
    }

    #[doc(hidden)]
    fn apply_slotmap_fresh(&self, m: &SlotMap) -> Self {
        let prv = self.private_slots();

        let mut c = self.clone();
        for x in c.public_slot_occurences_mut() {
            let y = m.get(*x).unwrap_or_else(Slot::fresh);

            // If y collides with a private slot, we have a problem.
            if CHECKS {
                assert!(!prv.contains(&y));
            }

            *x = y;
        }
        c
    }


    #[doc(hidden)]
    fn slot_occurences(&self) -> Vec<Slot> {
        self.public_slot_occurences()
    }

    #[doc(hidden)]
    fn slot_order(&self) -> Vec<Slot> { firsts(self.slot_occurences()) }

    #[doc(hidden)]
    fn slots(&self) -> HashSet<Slot> { as_set(self.slot_occurences()) }

    #[doc(hidden)]
    fn ids(&self) -> Vec<Id> {
        self.applied_id_occurences().into_iter().map(|x| x.id).collect()
    }

    // let n.weak_shape() = (sh, bij); then
    // - sh.apply_slotmap(bij) is equivalent to n (excluding lambda variable renames)
    // - bij.slots() == n.slots(). Note that these would also include redundant slots.
    // - sh is the lexicographically lowest equivalent version of n, reachable by bijective renaming of slots (including redundant ones).
    #[doc(hidden)]
    fn weak_shape(&self) -> (Self, Bijection) {
        let mut c = self.clone();
        let mut m = SlotMap::new();
        let mut i = 0;

        for x in c.all_slot_occurences_mut() {
            let x_val = *x;
            if !m.contains_key(x_val) {
                let new_slot = Slot::numeric(i);
                i += 1;

                m.insert(x_val, new_slot);
            }

            *x = m[x_val];
        }

        let m = m.inverse();

        let public = c.slots();
        let m: SlotMap = m.iter().filter(|(x, _)| public.contains(x)).collect();

        (c, m)
    }

    #[doc(hidden)]
    fn refresh_private(&self) -> Self {
        let mut c = self.clone();
        let prv: HashSet<Slot> = c.private_slot_occurences().into_iter().collect();
        let fresh = SlotMap::bijection_from_fresh_to(&prv).inverse();
        for x in c.private_slot_occurences_mut() {
            *x = fresh[*x];
        }
        c
    }

    #[doc(hidden)]
    fn refresh_slots(&self, set: HashSet<Slot>) -> Self {
        let mut c = self.clone();
        let fresh = SlotMap::bijection_from_fresh_to(&set).inverse();
        for x in c.all_slot_occurences_mut() {
            if set.contains(x) {
                *x = fresh[*x];
            }
        }
        c
    }

    // refreshes private and redundant slots.
    // The public slots are given by `public`.
    #[doc(hidden)]
    fn refresh_internals(&self, public: HashSet<Slot>) -> Self {
        let mut c = self.clone();
        let internals = &c.all_slot_occurences().into_iter().collect::<HashSet<_>>() - &public;
        let fresh = SlotMap::bijection_from_fresh_to(&internals).inverse();
        for x in c.all_slot_occurences_mut() {
            if internals.contains(x) {
                *x = fresh[*x];
            }
        }
        c
    }
}

// sorts as_set(v) by their first usage in v.
pub(crate) fn firsts(v: Vec<Slot>) -> Vec<Slot> {
    let mut out = Vec::new();
    for x in v {
        if !out.contains(&x) {
            out.push(x);
        }
    }
    out
}

pub(crate) fn as_set(v: Vec<Slot>) -> HashSet<Slot> {
    v.into_iter().collect()
}
