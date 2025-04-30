use crate::*;

#[derive(Debug, Clone)]
pub enum SyntaxElem {
    String(String), // used for identitifers and payloads
    AppliedId(AppliedId),
    Slot(Slot),
}

pub trait LanguageChildren: Debug + Clone + Hash + Eq {
    // TODO: add private_slot_occurrences aswell!
    fn all_slot_occurrences_iter_mut(&mut self) -> impl Iterator<Item = &mut Slot>;
    fn public_slot_occurrences_iter_mut(&mut self) -> impl Iterator<Item = &mut Slot>;
    fn applied_id_occurrences_iter_mut(&mut self) -> impl Iterator<Item = &mut AppliedId>;

    fn all_slot_occurrences_iter(&self) -> impl Iterator<Item = &Slot>;
    fn public_slot_occurrences_iter(&self) -> impl Iterator<Item = &Slot>;
    fn applied_id_occurrences_iter(&self) -> impl Iterator<Item = &AppliedId>;

    fn to_syntax(&self) -> Vec<SyntaxElem>;
    fn from_syntax(_: &[SyntaxElem]) -> Option<Self>;

    fn weak_shape_impl(&mut self, _m: &mut (SlotMap, u32)) {
        todo!()
    }
}

fn on_see_slot(s: &mut Slot, m: &mut (SlotMap, u32)) {
    if let Some(s2) = m.0.get(*s) {
        *s = s2;
    } else {
        add_slot(s, m);
    }
}

fn add_slot(s: &mut Slot, m: &mut (SlotMap, u32)) {
    let s2 = Slot::numeric(m.1);
    m.1 += 1;
    m.0.insert(*s, s2);
    *s = s2;
}

impl LanguageChildren for AppliedId {
    fn all_slot_occurrences_iter_mut(&mut self) -> impl Iterator<Item = &mut Slot> {
        self.m.values_mut()
    }
    fn public_slot_occurrences_iter_mut(&mut self) -> impl Iterator<Item = &mut Slot> {
        self.m.values_mut()
    }
    fn applied_id_occurrences_iter_mut(&mut self) -> impl Iterator<Item = &mut AppliedId> {
        std::iter::once(self)
    }

    fn all_slot_occurrences_iter(&self) -> impl Iterator<Item = &Slot> {
        self.m.values_immut()
    }
    fn public_slot_occurrences_iter(&self) -> impl Iterator<Item = &Slot> {
        self.m.values_immut()
    }
    fn applied_id_occurrences_iter(&self) -> impl Iterator<Item = &AppliedId> {
        std::iter::once(self)
    }

    fn to_syntax(&self) -> Vec<SyntaxElem> {
        vec![SyntaxElem::AppliedId(self.clone())]
    }
    fn from_syntax(elems: &[SyntaxElem]) -> Option<Self> {
        match elems {
            [SyntaxElem::AppliedId(x)] => Some(x.clone()),
            _ => None,
        }
    }

    fn weak_shape_impl(&mut self, m: &mut (SlotMap, u32)) {
        for x in self.m.values_mut() {
            on_see_slot(x, m);
        }
    }
}

impl LanguageChildren for Slot {
    fn all_slot_occurrences_iter_mut(&mut self) -> impl Iterator<Item = &mut Slot> {
        std::iter::once(self)
    }
    fn public_slot_occurrences_iter_mut(&mut self) -> impl Iterator<Item = &mut Slot> {
        std::iter::once(self)
    }
    fn applied_id_occurrences_iter_mut(&mut self) -> impl Iterator<Item = &mut AppliedId> {
        std::iter::empty()
    }

    fn all_slot_occurrences_iter(&self) -> impl Iterator<Item = &Slot> {
        std::iter::once(self)
    }
    fn public_slot_occurrences_iter(&self) -> impl Iterator<Item = &Slot> {
        std::iter::once(self)
    }
    fn applied_id_occurrences_iter(&self) -> impl Iterator<Item = &AppliedId> {
        std::iter::empty()
    }

    fn to_syntax(&self) -> Vec<SyntaxElem> {
        vec![SyntaxElem::Slot(*self)]
    }
    fn from_syntax(elems: &[SyntaxElem]) -> Option<Self> {
        match elems {
            [SyntaxElem::Slot(x)] => Some(x.clone()),
            _ => None,
        }
    }

    fn weak_shape_impl(&mut self, m: &mut (SlotMap, u32)) {
        on_see_slot(self, m);
    }
}

/// Implements [LanguageChildren] for payload types that are independent of Slots. For example u32, String etc.
#[macro_export]
macro_rules! bare_language_child {
    ($($id:ident),*) => {
        $(
        impl LanguageChildren for $id {
            fn all_slot_occurrences_iter_mut(&mut self) -> impl Iterator<Item=&mut Slot> { std::iter::empty() }
            fn public_slot_occurrences_iter_mut(&mut self) -> impl Iterator<Item=&mut Slot> { std::iter::empty() }
            fn applied_id_occurrences_iter_mut(&mut self) -> impl Iterator<Item=&mut AppliedId> { std::iter::empty() }

            fn all_slot_occurrences_iter(&self) -> impl Iterator<Item=&Slot> { std::iter::empty() }
            fn public_slot_occurrences_iter(&self) -> impl Iterator<Item=&Slot> { std::iter::empty() }
            fn applied_id_occurrences_iter(&self) -> impl Iterator<Item=&AppliedId> { std::iter::empty() }

            fn to_syntax(&self) -> Vec<SyntaxElem> { vec![SyntaxElem::String(self.to_string())] }
            fn from_syntax(elems: &[SyntaxElem]) -> Option<Self> {
                match elems {
                    [SyntaxElem::String(x)] => x.parse().ok(),
                    _ => None,
                }
            }

            fn weak_shape_impl(&mut self, _m: &mut (SlotMap, u32)) {}
        }
        )*
    }
}

bare_language_child!(
    u128, u64, u32, u16, u8, i128, i64, i32, i16, i8, usize, isize, bool, char, Symbol
);

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct Bind<T> {
    pub slot: Slot,
    pub elem: T,
}

impl<L: LanguageChildren> LanguageChildren for Bind<L> {
    // mut:
    fn all_slot_occurrences_iter_mut(&mut self) -> impl Iterator<Item = &mut Slot> {
        std::iter::once(&mut self.slot).chain(self.elem.all_slot_occurrences_iter_mut())
    }

    fn public_slot_occurrences_iter_mut(&mut self) -> impl Iterator<Item = &mut Slot> {
        self.elem
            .public_slot_occurrences_iter_mut()
            .filter(|x| **x != self.slot)
    }

    fn applied_id_occurrences_iter_mut(&mut self) -> impl Iterator<Item = &mut AppliedId> {
        self.elem.applied_id_occurrences_iter_mut()
    }

    // immut:
    fn all_slot_occurrences_iter(&self) -> impl Iterator<Item = &Slot> {
        std::iter::once(&self.slot).chain(self.elem.all_slot_occurrences_iter())
    }

    fn public_slot_occurrences_iter(&self) -> impl Iterator<Item = &Slot> {
        self.elem
            .public_slot_occurrences_iter()
            .filter(|x| **x != self.slot)
    }

    fn applied_id_occurrences_iter(&self) -> impl Iterator<Item = &AppliedId> {
        self.elem.applied_id_occurrences_iter()
    }

    // syntax:
    fn to_syntax(&self) -> Vec<SyntaxElem> {
        let mut v = vec![SyntaxElem::Slot(self.slot)];
        v.extend(self.elem.to_syntax());

        v
    }

    fn from_syntax(elems: &[SyntaxElem]) -> Option<Self> {
        let SyntaxElem::Slot(slot) = elems.get(0)? else {
            return None;
        };
        let elem = L::from_syntax(&elems[1..])?;

        Some(Bind { slot: *slot, elem })
    }

    fn weak_shape_impl(&mut self, m: &mut (SlotMap, u32)) {
        let s = self.slot;
        add_slot(&mut self.slot, m);
        self.elem.weak_shape_impl(m);
        m.0.remove(s);
    }
}

// TODO: add LanguageChildren definition for tuples.

/// A trait to define your Language (i.e. your E-Node type).
pub trait Language: Debug + Clone + Hash + Eq {
    /// List the mutable references of all child [Slot]s in your E-Node, in order of occurrence.
    fn all_slot_occurrences_mut(&mut self) -> Vec<&mut Slot>;

    /// List the mutable references to all *public* child [Slot]s in your E-Node, in order of occurrence.
    ///
    /// Public Slots are those, which are visible from the outside of that e-node.
    /// * A typical example would be a `(var $x)` e-node, which has a *public* slot `$x`.
    /// * A typical counter-example would be the `(lam $x body)` e-node, which has a *private* slot `$x`.
    fn public_slot_occurrences_mut(&mut self) -> Vec<&mut Slot>;

    /// List the mutable references to all child [AppliedId]s in your E-Node, in the order of occurrence.
    fn applied_id_occurrences_mut(&mut self) -> Vec<&mut AppliedId>;

    fn all_slot_occurrences(&self) -> Vec<Slot>;
    fn public_slot_occurrences(&self) -> Vec<Slot>;
    fn applied_id_occurrences(&self) -> Vec<&AppliedId>;

    /// This function will be used to display your E-Node.
    fn to_syntax(&self) -> Vec<SyntaxElem>;

    /// This function will be used to parse your E-Node.
    fn from_syntax(_: &[SyntaxElem]) -> Option<Self>;

    fn slots(&self) -> SmallHashSet<Slot>;
    fn weak_shape_inplace(&mut self) -> Bijection;

    #[track_caller]
    #[doc(hidden)]
    fn check(&self) {
        let mut c = self.clone();
        let all: HashSet<*mut Slot> = c
            .all_slot_occurrences_mut()
            .into_iter()
            .map(|x| x as *mut Slot)
            .collect();
        let public: HashSet<*mut Slot> = c
            .public_slot_occurrences_mut()
            .into_iter()
            .map(|x| x as *mut Slot)
            .collect();
        let private: HashSet<*mut Slot> = c
            .private_slot_occurrences_mut()
            .into_iter()
            .map(|x| x as *mut Slot)
            .collect();

        assert!(public.is_disjoint(&private));

        // This also catches errors, where different Slot-addresses have the same slot names. This also counts as a collision!
        let f = |x: Vec<Slot>| x.into_iter().collect::<HashSet<_>>();
        assert!(f(c.public_slot_occurrences()).is_disjoint(&f(c.private_slot_occurrences())));

        let all2: HashSet<*mut Slot> = public.union(&private).copied().collect();
        assert_eq!(all2, all);
    }

    fn is_leaf(&self) -> bool {
        self.applied_id_occurrences().len() == 0
    }

    // generated methods:

    #[doc(hidden)]
    fn private_slot_occurrences_mut(&mut self) -> Vec<&mut Slot> {
        let public = self.public_slot_occurrences();
        let mut out = self.all_slot_occurrences_mut();
        out.retain(|x| !public.contains(x));
        out
    }

    #[doc(hidden)]
    fn private_slot_occurrences(&self) -> Vec<Slot> {
        let public = self.public_slot_occurrences();
        let mut out = self.all_slot_occurrences();
        out.retain(|x| !public.contains(x));
        out
    }

    #[doc(hidden)]
    fn private_slots(&self) -> SmallHashSet<Slot> {
        self.private_slot_occurrences().into_iter().collect()
    }

    #[doc(hidden)]
    fn map_applied_ids(&self, mut f: impl FnMut(AppliedId) -> AppliedId) -> Self {
        let mut c = self.clone();
        for x in c.applied_id_occurrences_mut() {
            *x = f(x.clone());
        }
        c
    }

    // TODO m.values() might collide with your private slot names.
    // Should we rename our private slots to be safe?
    #[doc(hidden)]
    fn apply_slotmap_partial(&self, m: &SlotMap) -> Self {
        let mut prv = vec![].into();
        if CHECKS {
            prv = self.private_slots();
        }

        let mut c = self.clone();
        for x in c.public_slot_occurrences_mut() {
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
            assert!(
                m.keys().is_superset(&self.slots()),
                "Language::apply_slotmap: The SlotMap doesn't map all free slots!"
            );
        }
        self.apply_slotmap_partial(m)
    }

    #[doc(hidden)]
    fn apply_slotmap_fresh(&self, m: &SlotMap) -> Self {
        let mut prv = vec![].into();
        if CHECKS {
            prv = self.private_slots();
        }

        let mut c = self.clone();
        for x in c.public_slot_occurrences_mut() {
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
    fn ids(&self) -> Vec<Id> {
        self.applied_id_occurrences()
            .into_iter()
            .map(|x| x.id)
            .collect()
    }

    // let n.weak_shape() = (sh, bij); then
    // - sh.apply_slotmap(bij) is equivalent to n (excluding lambda variable renames)
    // - bij.slots() == n.slots(). Note that these would also include redundant slots.
    // - sh is the lexicographically lowest equivalent version of n, reachable by bijective renaming of slots (including redundant ones).
    #[doc(hidden)]
    fn weak_shape(&self) -> (Self, Bijection) {
        let mut c = self.clone();
        let bij = c.weak_shape_inplace();
        (c, bij)
    }

    #[doc(hidden)]
    fn refresh_private(&self) -> Self {
        let mut c = self.clone();
        let prv: SmallHashSet<Slot> = c.private_slot_occurrences().into_iter().collect();
        let fresh = SlotMap::bijection_from_fresh_to(&prv).inverse();
        for x in c.private_slot_occurrences_mut() {
            *x = fresh[*x];
        }
        c
    }

    #[doc(hidden)]
    fn refresh_slots(&self, set: SmallHashSet<Slot>) -> Self {
        let mut c = self.clone();
        let fresh = SlotMap::bijection_from_fresh_to(&set).inverse();
        for x in c.all_slot_occurrences_mut() {
            if set.contains(x) {
                *x = fresh[*x];
            }
        }
        c
    }

    // refreshes private and redundant slots.
    // The public slots are given by `public`.
    #[doc(hidden)]
    fn refresh_internals(&self, public: SmallHashSet<Slot>) -> Self {
        let mut c = self.clone();
        let internals = &c
            .all_slot_occurrences()
            .into_iter()
            .collect::<SmallHashSet<_>>()
            - &public;
        let fresh = SlotMap::bijection_from_fresh_to(&internals).inverse();
        for x in c.all_slot_occurrences_mut() {
            if internals.contains(x) {
                *x = fresh[*x];
            }
        }
        c
    }

    // fn map_children<F: FnMut(AppliedId) -> AppliedId>(mut self, f: F) -> Self {
    //     // self.update_children(f);
    //     self.applied_id_occurrences_mut()
    //         .into_iter()
    //         .for_each(|aid| *aid = f(*aid));
    //     self
    // }

    fn build_recexpr<F>(&self, mut get_node: F) -> RecExprFlat<Self>
    where
        F: FnMut(AppliedId) -> Self,
    {
        self.try_build_recexpr::<_, std::convert::Infallible>(|id| Ok(get_node(id)))
            .unwrap()
    }

    /// Same as [`Language::build_recexpr`], but fallible.
    fn try_build_recexpr<F, Err>(&self, mut get_node: F) -> Result<RecExprFlat<Self>, Err>
    where
        F: FnMut(AppliedId) -> Result<Self, Err>,
    {
        let mut set = indexmap::IndexSet::<Self>::default();
        let mut ids = HashMap::<Id, Id>::default();
        let mut todo: Vec<AppliedId> = self
            .applied_id_occurrences()
            .iter()
            .map(|&aid| aid.clone())
            .collect();

        while let Some(id) = todo.last().cloned() {
            if ids.contains_key(&id.id) {
                todo.pop();
                continue;
            }

            let node = get_node(id.clone())?;

            // check to see if we can do this node yet
            let mut ids_has_all_children = true;
            for child in node.applied_id_occurrences() {
                if !ids.contains_key(&child.id) {
                    ids_has_all_children = false;
                    todo.push(child.clone())
                }
            }

            // all children are processed, so we can lookup this node safely
            if ids_has_all_children {
                // let node = node.map_children(|id| ids[&id]);
                let mut n2 = node.clone();
                n2.applied_id_occurrences_mut()
                    .iter_mut()
                    .for_each(|id| id.id = ids[&id.id]);
                let new_id = set.insert_full(n2).0;
                ids.insert(id.id, Id(new_id));
                todo.pop();
            }
        }

        // finally, create the expression and add the root node
        let expr: Vec<_> = set.into_iter().collect();
        let mut expr = RecExprFlat { nodes: expr };
        let mut nexpr = self.clone();
        nexpr
            .applied_id_occurrences_mut()
            .iter_mut()
            .for_each(|aid| aid.id = ids[&aid.id]);
        // .map_children(|id| ids[&id]);
        expr.add(nexpr);
        Ok(expr)
    }
}
