use crate::*;

mod find;
pub use find::*;

mod add;
pub use add::*;

mod union;
pub use union::*;

mod rebuild;
pub use rebuild::*;

mod check;
pub use check::*;

mod analysis;
pub use analysis::*;

use std::cell::RefCell;

// invariants:
// 1. If two ENodes (that are in the EGraph) have equal .shape(), they have to be in the same eclass.
// 2. enode.slots() is always a superset of c.slots, if enode is within the eclass c.
//    if ENode::Lam(si) = enode, then we require i to not be in c.slots.
//    In practice, si will always be Slot(0).
// 3. AppliedId::m is always a bijection. (eg. c1(s0, s1, s0) is illegal!)
//    AppliedId::m also always has the same keys as the class expects slots.
// 4. Slot(0) should not be in EClass::slots of any class.
/// A datastructure to efficiently represent congruence relations on terms with binders.
pub struct EGraph<L: Language, N: Analysis<L> = ()> {
    // an entry (l, r(sa, sb)) in unionfind corresponds to the equality l(s0, s1, s2) = r(sa, sb), where sa, sb in {s0, s1, s2}.
    // normalizes the eclass.
    // Each Id i that is an output of the unionfind itself has unionfind[i] = (i, identity()).

    // We use RefCell to allow for inter mutability, so that find(&self) can do path compression.
    unionfind: RefCell<Vec<ProvenAppliedId>>,

    // if a class does't have unionfind[x].id = x, then it doesn't contain nodes / usages.
    // It's "shallow" if you will.
    pub(crate) classes: HashMap<Id, EClass<L, N>>,

    // For each shape contained in the EGraph, maps to the EClass that contains it.
    hashcons: HashMap<L, Id>,

    // For each (syn_slotset applied) non-normalized (i.e. "syntactic") weak shape, find the e-class who has this as syn_enode.
    // TODO remove this if explanations are disabled.
    syn_hashcons: HashMap<L, AppliedId>,

    // E-Nodes that need to be re-processed, stored as shapes.
    pending: HashMap<L, PendingType>,

    // TODO remove this if explanations are disabled.
    pub(crate) proof_registry: ProofRegistry,

    pub(crate) subst_method: Option<Box<dyn SubstMethod<L, N>>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum PendingType {
    OnlyAnalysis, // only analysis needs to be updated.
    Full, // the e-node, it's strong shape & the analysis need to be updated.
}

/// Each E-Class can be understood "semantically" or "syntactically":
/// - semantically means that it respects the equations already in the e-graph, and hence doesn't differentiate between equal things.
/// - syntactically means that it only talks about the single representative term associated to each E-Class, recursively obtainable using syn_enode.
#[derive(Clone)]
pub(crate) struct EClass<L: Language, N: Analysis<L>> {
    // The set of equivalent ENodes that make up this eclass.
    // for (sh, bij) in nodes; sh.apply_slotmap(bij) represents the actual ENode.
    nodes: HashMap<L, ProvenSourceNode>,

    // All other slots are considered "redundant" (or they have to be qualified by a ENode::Lam).
    // Should not contain Slot(0).
    slots: HashSet<Slot>,

    // Shows which Shapes refer to this EClass.
    usages: HashSet<L>,

    // Expresses the self-symmetries of this e-class.
    pub(crate) group: Group<ProvenPerm>,

    // TODO remove this if explanations are disabled.
    syn_enode: L,

    analysis_data: N,
}


impl<L: Language, N: Analysis<L>> EGraph<L, N> {
    /// Creates an empty e-graph.
    pub fn new() -> Self {
        Self::with_subst_method::<SynExprSubst>()
    }

    /// Creates an empty e-graph, while specifying the substitution method to use.
    pub fn with_subst_method<S: SubstMethod<L, N>>() -> Self {
        EGraph {
            unionfind: Default::default(),
            classes: Default::default(),
            hashcons: Default::default(),
            syn_hashcons: Default::default(),
            pending: Default::default(),
            proof_registry: ProofRegistry::default(),
            subst_method: Some(S::new_boxed()),
        }
    }

    pub fn slots(&self, id: Id) -> HashSet<Slot> {
        self.classes[&id].slots.clone()
    }

    pub(crate) fn syn_slots(&self, id: Id) -> HashSet<Slot> {
        self.classes[&id].syn_enode.slots()
    }

    pub fn analysis_data(&self, i: Id) -> &N {
        &self.classes[&self.find_id(i)].analysis_data
    }

    pub fn analysis_data_mut(&mut self, i: Id) -> &mut N {
        &mut self.classes.get_mut(&self.find_id(i)).unwrap().analysis_data
    }

    pub fn enodes(&self, i: Id) -> HashSet<L> {
        // We prevent this, as otherwise the output will have wrong slots.
        assert!(self.is_alive(i), "Can't access e-nodes of dead class");

        self.classes[&i].nodes.iter().map(|(x, psn)| x.apply_slotmap(&psn.elem)).collect()
    }

    // Generates fresh slots for redundant slots.
    pub fn enodes_applied(&self, i: &AppliedId) -> HashSet<L> {
        let mut out = HashSet::default();
        for x in self.enodes(i.id) {
            // This is necessary, as i.slots() might collide with the private/redundant slots of our e-nodes.
            let set: HashSet<_> = x.all_slot_occurences()
                                   .into_iter()
                                   .collect::<HashSet<_>>()
                                   .difference(&self.classes[&i.id].slots)
                                   .copied()
                                   .collect();
            let x = x.refresh_slots(set);

            let red = &x.slots() - &i.m.keys();
            let fbij = SlotMap::bijection_from_fresh_to(&red);
            let m = fbij.inverse().union(&i.m);
            out.insert(x.apply_slotmap(&m));
        }

        if CHECKS {
            for x in &out {
                assert!(self.eq(&self.lookup(x).unwrap(), &i));
            }
        }

        out
    }

    // number of enodes in the egraph.
    pub fn total_number_of_nodes(&self) -> usize {
        self.hashcons.len()
    }

    /// Checks that two AppliedIds are semantically equal.
    pub fn eq(&self, a: &AppliedId, b: &AppliedId) -> bool {
        let a = self.find_applied_id(a);
        let b = self.find_applied_id(b);

        if CHECKS {
            self.check_sem_applied_id(&a);
            self.check_sem_applied_id(&b);
        }

        if a.id != b.id { return false; }
        if a.m.values() != b.m.values() { return false; }
        let id = a.id;

        let perm = a.m.compose(&b.m.inverse());
        if CHECKS {
            assert!(perm.is_perm());
            assert_eq!(&perm.values(), &self.classes[&id].slots);
        }

        self.classes[&id].group.contains(&perm)
    }

    // refreshes all internal slots of l.
    pub(crate) fn refresh_internals(&self, l: &L) -> L {
        let i = self.lookup(l).unwrap();
        l.refresh_internals(i.slots())
    }

    // converts l to its class normal form, so that calling lookup on it yields the identity AppliedId.
    pub(crate) fn class_nf(&self, l: &L) -> L {
        let l = self.refresh_internals(l);
        let i = self.lookup(&l).unwrap();

        // needs to be `apply_slotmap_fresh` in case `l` has redundancies.
        let l = l.apply_slotmap_fresh(&i.m);

        if CHECKS {
            let identity = self.mk_sem_identity_applied_id(i.id);
            assert!(self.eq(&i, &identity));
        }

        l
    }

    /// Prints the contents of the E-Graph. Helpful for debugging.
    pub fn dump(&self) {
        println!("");
        let mut v: Vec<(&Id, &EClass<L, N>)> = self.classes.iter().collect();
        v.sort_by_key(|(x, _)| *x);

        for (i, c) in v {
            if c.nodes.len() == 0 { continue; }

            let mut slot_order: Vec<Slot> = c.slots.iter().cloned().collect();
            slot_order.sort();
            let slot_str = slot_order.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
            println!("\n{:?}({}):", i, &slot_str);

            println!(">> {:?}", &c.syn_enode);

            for (sh, psn) in &c.nodes {
                let n = sh.apply_slotmap(&psn.elem);

                #[cfg(feature = "explanations")]
                println!(" - {n:?}    [originally {:?}]", psn.src_id);

                #[cfg(not(feature = "explanations"))]
                println!(" - {n:?}");
            }
            for pp in &c.group.generators() {
                println!(" -- {:?}", pp.elem);
            }
        }
        println!("");
    }

    // The resulting e-nodes are written as they exist in the e-class.
    pub(crate) fn usages(&self, i: Id) -> Vec<L> {
        let mut out = Vec::new();
        for x in &self.classes[&i].usages {
            let j = self.lookup(x).unwrap().id;
            let bij = &self.classes[&j].nodes[&x].elem;
            let x = x.apply_slotmap(bij);
            out.push(x);
        }
        out
    }

    pub(crate) fn shape(&self, e: &L) -> (L, Bijection) {
        let (pnode, bij) = self.proven_shape(e);
        (pnode.elem, bij)
    }

    pub(crate) fn proven_shape(&self, e: &L) -> (ProvenNode<L>, Bijection) {
        self.proven_proven_shape(&self.refl_pn(e))
    }

    pub(crate) fn proven_proven_shape(&self, e: &ProvenNode<L>) -> (ProvenNode<L>, Bijection) {
        self.proven_proven_pre_shape(&e).weak_shape()
    }

    pub(crate) fn proven_proven_pre_shape(&self, e: &ProvenNode<L>) -> ProvenNode<L> {
        let e = self.proven_proven_find_enode(e);
        self.proven_proven_get_group_compatible_variants(&e)
            .into_iter()
            .min_by_key(|pn| pn.weak_shape().0.elem.all_slot_occurences())
            .unwrap()
    }

    pub(crate) fn proven_proven_get_group_compatible_variants(&self, enode: &ProvenNode<L>) -> HashSet<ProvenNode<L>> {
        // should only be called with an up-to-date e-node.
        if CHECKS {
            for x in enode.elem.applied_id_occurences() {
                assert!(self.is_alive(x.id));
            }
        }

        let n = enode.elem.applied_id_occurences().len();

        let mut out = HashSet::default();

        let groups: Vec<Vec<ProvenPerm>> = enode.elem.applied_id_occurences().iter().map(
                    |x| self.classes[&x.id].group.all_perms().into_iter().collect()
            ).collect();

        for l in cartesian(&groups) {
            let pn = enode.clone();
            let pn = self.chain_pn_map(&pn, |i, pai| self.chain_pai_pp(&pai, l[i]));
            // TODO fix check.
            // if CHECKS { pn.check_base(enode.base()); }
            out.insert(pn);
        }

        out
    }

    // for all AppliedIds that are contained in `enode`, permute their arguments as their groups allow.
    // TODO every usage of this function hurts performance drastically. Which of them can I eliminate?
    pub(crate) fn proven_get_group_compatible_variants(&self, enode: &L) -> HashSet<ProvenNode<L>> {
        self.proven_proven_get_group_compatible_variants(&self.refl_pn(enode))
    }

    pub(crate) fn get_group_compatible_variants(&self, enode: &L) -> HashSet<L> {
        self.proven_get_group_compatible_variants(enode).into_iter().map(|pnode| pnode.elem).collect()
    }

    #[cfg_attr(feature = "trace", instrument(level = "trace", skip_all))]
    pub(crate) fn get_group_compatible_weak_variants(&self, enode: &L) -> HashSet<L> {
        let set = self.get_group_compatible_variants(enode);
        let mut shapes = HashSet::default();
        let mut out = HashSet::default();

        for x in set {
            let (sh, _) = x.weak_shape();
            if shapes.contains(&sh) { continue; }
            shapes.insert(sh);
            out.insert(x);
        }

        out
    }

    pub(crate) fn synify_app_id(&self, app: AppliedId) -> AppliedId {
        let mut app = app;
        for s in self.syn_slots(app.id) {
            if !app.m.contains_key(s) {
                app.m.insert(s, Slot::fresh());
            }
        }
        app
    }

    pub(crate) fn synify_enode(&self, enode: L) -> L {
        enode.map_applied_ids(|app| self.synify_app_id(app))
    }

    pub(crate) fn semify_app_id(&self, app: AppliedId) -> AppliedId {
        let slots = self.slots(app.id);

        let mut app = app;
        for k in app.m.keys() {
            if !slots.contains(&k) {
                app.m.remove(k);
            }
        }
        app
    }

    pub(crate) fn semify_enode(&self, enode: L) -> L {
        enode.map_applied_ids(|app| self.semify_app_id(app))
    }

    /// Returns the canonical term corresponding to `i`.
    ///
    /// This function will use [EGraph::get_syn_node] repeatedly to build up this term.
    pub fn get_syn_expr(&self, i: &AppliedId) -> RecExpr<L> {
        let enode = self.get_syn_node(i);
        let cs = enode.applied_id_occurences()
                      .iter()
                      .map(|x| self.get_syn_expr(x))
                      .collect();
        RecExpr {
            node: nullify_app_ids(&enode),
            children: cs,
        }
    }

    /// Returns the canonical e-node corresponding to `i`.
    pub fn get_syn_node(&self, i: &AppliedId) -> L {
        let syn = &self.classes[&i.id].syn_enode;
        syn.apply_slotmap(&i.m)
    }
}

impl PendingType {
    pub(crate) fn merge(self, other: PendingType) -> PendingType {
        match (self, other) {
            (PendingType::Full, _) => PendingType::Full,
            (_, PendingType::Full) => PendingType::Full,
            (PendingType::OnlyAnalysis, PendingType::OnlyAnalysis) => PendingType::OnlyAnalysis,
        }
    }
}

// {1,2} x {3} x {4,5} -> (1,3,4), (1,3,5), (2,3,4), (2,3,5)
// TODO re-enable use<...> when it's stabilized.
fn cartesian<'a, T>(input: &'a [Vec<T>]) -> impl Iterator<Item=Vec<&'a T>> /*+ use<'a, T>*/ + '_ {
    let n = input.len();
    let mut indices = vec![0; n];
    let mut done = false;
    let f = move || {
        if done { return None; }
        let out: Vec<&T> = (0..n).map(|i| &input[i][indices[i]]).collect();
        for i in 0..n {
            indices[i] += 1;
            if indices[i] >= input[i].len() {
                indices[i] = 0;
            } else {
                return Some(out);
            }
        }
        done = true;
        Some(out)
    };
    std::iter::from_fn(f)
}

#[test]
fn cartesian1() {
    let v = [vec![1, 2], vec![3], vec![4, 5]];
    let vals = cartesian(&v);
    assert_eq!(vals.count(), 4);
}
