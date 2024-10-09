use crate::*;

mod find;
pub use find::*;

mod add;
pub use add::*;

mod union;
pub use union::*;

mod expl;
pub use expl::*;

mod check;
pub use check::*;

use std::sync::Mutex;

/// Each E-Class can be understood "semantically" or "syntactically":
/// - semantically means that it respects the equations already in the e-graph, and hence doesn't differentiate between equal things.
/// - syntactically means that it only talks about the single representative term associated to each E-Class, recursively obtainable using syn_enode.
#[derive(Clone, Debug)]
pub struct EClass<L: Language> {
    // The set of equivalent ENodes that make up this eclass.
    // for (sh, bij) in nodes; sh.apply_slotmap(bij) represents the actual ENode.
    nodes: HashMap<L, ProvenSourceNode>,

    // All other slots are considered "redundant" (or they have to be qualified by a ENode::Lam).
    // Should not contain Slot(0).
    slots: HashSet<Slot>,

    // Shows which Shapes refer to this EClass.
    usages: HashSet<L>,

    // Expresses the self-symmetries of this e-class.
    group: Group<ProvenPerm>,

    syn_enode: L,

    // is of the form `c[...] = c[...]` where everything is stabilized, except for the redundant slots which are just used on one side.
    // only relevant for the leader of an e-class.
    redundancy_proof: ProvenEq,
}

// invariants:
// 1. If two ENodes (that are in the EGraph) have equal .shape(), they have to be in the same eclass.
// 2. enode.slots() is always a superset of c.slots, if enode is within the eclass c.
//    if ENode::Lam(si) = enode, then we require i to not be in c.slots.
//    In practice, si will always be Slot(0).
// 3. AppliedId::m is always a bijection. (eg. c1(s0, s1, s0) is illegal!)
//    AppliedId::m also always has the same keys as the class expects slots.
// 4. Slot(0) should not be in EClass::slots of any class.
#[derive(Debug)]
pub struct EGraph<L: Language> {
    // an entry (l, r(sa, sb)) in unionfind corresponds to the equality l(s0, s1, s2) = r(sa, sb), where sa, sb in {s0, s1, s2}.
    // normalizes the eclass.
    // Each Id i that is an output of the unionfind itself has unionfind[i] = (i, identity()).

    // We use mutex to allow for inter mutability, so that find(&self) can do path compression.
    unionfind: Mutex<Vec<ProvenAppliedId>>,

    // if a class does't have unionfind[x].id = x, then it doesn't contain nodes / usages.
    // It's "shallow" if you will.
    classes: HashMap<Id, EClass<L>>,

    // For each shape contained in the EGraph, maps to the EClass that contains it.
    hashcons: HashMap<L, Id>,

    // For each (syn_slotset applied) non-normalized (i.e. "syntactic") weak shape, find the e-class who has this as syn_enode.
    syn_hashcons: HashMap<L, AppliedId>,

    // E-Nodes that need to be re-processed, stored as shapes.
    pending: HashSet<L>,

    proof_registry: ProofRegistry,
}

impl<L: Language> EGraph<L> {
    pub fn new() -> Self {
        EGraph {
            unionfind: Default::default(),
            classes: Default::default(),
            hashcons: Default::default(),
            syn_hashcons: Default::default(),
            pending: Default::default(),
            proof_registry: ProofRegistry::default(),
        }
    }

    pub fn slots(&self, id: Id) -> HashSet<Slot> {
        self.classes[&id].slots.clone()
    }

    pub fn syn_slots(&self, id: Id) -> HashSet<Slot> {
        self.classes[&id].syn_enode.slots()
    }

    pub fn ids(&self) -> Vec<Id> {
        self.unionfind_iter()
                       .filter(|(x, y)| x == &y.id)
                       .map(|(x, _)| x)
                       .collect()
    }

    // TODO For non-normalized inputs i, the slots in the output will definitely be wrong.
    // if x in enodes(i), then I'd expect x.slots() superset slots(i).
    pub fn enodes(&self, i: Id) -> HashSet<L> {
        let i = self.find_id(i);
        self.classes[&i].nodes.iter().map(|(x, psn)| x.apply_slotmap(&psn.elem)).collect()
    }

    // Generates fresh slots for redundant slots.
    pub fn enodes_applied(&self, i: &AppliedId) -> HashSet<L> {
        let i = self.find_applied_id(i);

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

    fn is_alive(&self, i: Id) -> bool {
        self.find_id(i) == i
    }

    // refreshes all internal slots of l.
    pub fn refresh_internals(&self, l: &L) -> L {
        let i = self.lookup(l).unwrap();
        l.refresh_internals(i.slots())
    }

    // converts l to its class normal form, so that calling lookup on it yields the identity AppliedId.
    pub fn class_nf(&self, l: &L) -> L {
        let l = self.refresh_internals(l);
        let i = self.lookup(&l).unwrap();
        let l = l.apply_slotmap(&i.m);

        if CHECKS {
            assert!(self.lookup(&l).unwrap().m.iter().all(|(x, y)| x == y));
        }

        l
    }

    pub fn dump(&self) {
        println!("");
        let mut v: Vec<(&Id, &EClass<L>)> = self.classes.iter().collect();
        v.sort_by_key(|(x, _)| *x);

        for (i, c) in v {
            let slot_str = c.slots.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(", ");
            println!("\n{:?}({}):", i, &slot_str);

            println!(">> {:?}", &c.syn_enode);
            println!(">>> {:?}", &c.redundancy_proof.equ());

            for (sh, ProvenSourceNode { elem: bij, src_id: app_id}) in &c.nodes {
                let n = sh.apply_slotmap(bij);
                println!(" - {n:?}    [originally {app_id:?}]");
            }
            for pp in &c.group.generators() {
                println!(" -- {:?}", pp.elem);
            }
        }
        println!("");
    }

    // The resulting e-nodes are written as they exist in the e-class.
    pub fn usages(&self, i: Id) -> Vec<L> {
        let mut out = Vec::new();
        for x in &self.classes[&i].usages {
            let j = self.lookup(x).unwrap().id;
            let bij = &self.classes[&j].nodes[&x].elem;
            let x = x.apply_slotmap(bij);
            out.push(x);
        }
        out
    }

    pub fn shape(&self, e: &L) -> (L, Bijection) {
        let (pnode, bij) = self.proven_shape(e);
        (pnode.elem, bij)
    }

    pub fn proven_shape(&self, e: &L) -> (ProvenNode<L>, Bijection) {
        let ProvenNode { elem: e, proofs: v1 } = self.proven_find_enode(e);
        let (ProvenNode { elem: sh, proofs: v2 }, bij) = self.proven_get_group_compatible_variants(&e)
            .into_iter()
            .map(|ProvenNode { elem: x, proofs: prfs}| {
                let (sh, bij) = x.weak_shape();
                (ProvenNode { elem: sh, proofs: prfs}, bij)
            }).min_by_key(|(ProvenNode { elem: x, ..}, _)| x.all_slot_occurences()).unwrap();

        let mut out: Vec<ProvenEq> = Vec::new();

        assert_eq!(v1.len(), e.applied_id_occurences().len());
        assert_eq!(v2.len(), e.applied_id_occurences().len());

        let v1 = v1.into_iter();
        let v2 = v2.into_iter();

        for (l, r) in v1.zip(v2) {
            out.push(self.prove_transitivity(l, r));
        }

        (ProvenNode { elem: sh, proofs: out }, bij)
    }

    fn refl_proof(&self, i: Id) -> ProvenEq {
        let syn_slots = self.syn_slots(i);
        let identity = SlotMap::identity(&syn_slots);
        let app_id = AppliedId::new(i, identity);
        self.prove_reflexivity(&app_id)
    }

    // for all AppliedIds that are contained in `enode`, permute their arguments as their groups allow.
    // TODO every usage of this function hurts performance drastically. Which of them can I eliminate?
    pub fn proven_get_group_compatible_variants(&self, enode: &L) -> HashSet<ProvenNode<L>> {
        // should only be called with an up-to-date e-node.
        if CHECKS {
            for x in enode.applied_id_occurences() {
                assert!(self.is_alive(x.id));
            }
        }

        let n = enode.applied_id_occurences().len();
        let mut s: HashSet<ProvenNode<L>> = HashSet::default();

        // the proofs in `s` should express how its node changed relative to `enode`.
        let s_inv = |s: &HashSet<ProvenNode<L>>| {
            if CHECKS {
                for ProvenNode { elem: new_enode, proofs: prfs } in s {
                    for i in 0..n {
                        let l = enode.applied_id_occurences()[i].clone();
                        let r = new_enode.applied_id_occurences()[i].clone();
                        let eq = Equation { l, r };
                        assert_proves_equation(&prfs[i], &eq);
                    }
                }
            }
        };


        let mut init = Vec::new();
        for x in enode.applied_id_occurences() {
            init.push(self.refl_proof(x.id));
        }

        s.insert(ProvenNode { elem: enode.clone(), proofs: init });

        s_inv(&s);

        for (i, app_id) in enode.applied_id_occurences().iter().enumerate() {
            let grp_perms = self.classes[&app_id.id].group.all_perms();
            let mut next = HashSet::default();
            s_inv(&s);
            for ProvenNode { elem: x, proofs: x_prfs} in s {
                for proven_perm in &grp_perms {
                    proven_perm.check();
                    let x_i = x.applied_id_occurences()[i].clone();
                    let x_prfs_i = x_prfs[i].clone();
                    let tmp_pai = ProvenAppliedId {
                        elem: x_i,
                        proof: x_prfs_i,
                    };
                    let ProvenAppliedId { elem: app_id, proof: prf } = self.chain_pai_pp(tmp_pai, proven_perm);

                    let mut x2 = x.clone();
                    *x2.applied_id_occurences_mut()[i] = app_id;

                    let mut x_prfs2 = x_prfs.clone();
                    x_prfs2[i] = prf;

                    next.insert(ProvenNode { elem: x2, proofs: x_prfs2});
                }
            }
            s = next;
            s_inv(&s);
        }

        s
    }

    pub fn get_group_compatible_variants(&self, enode: &L) -> HashSet<L> {
        self.proven_get_group_compatible_variants(enode).into_iter().map(|pnode| pnode.elem).collect()
    }

    pub fn get_group_compatible_weak_variants(&self, enode: &L) -> HashSet<L> {
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

    pub fn synify_app_id(&self, app: AppliedId) -> AppliedId {
        let mut app = app;
        for s in self.syn_slots(app.id) {
            if !app.m.contains_key(s) {
                app.m.insert(s, Slot::fresh());
            }
        }
        app
    }

    pub fn synify_enode(&self, enode: L) -> L {
        enode.map_applied_ids(|app| self.synify_app_id(app))
    }

    pub fn semify_app_id(&self, app: AppliedId) -> AppliedId {
        let slots = self.slots(app.id);

        let mut app = app;
        for k in app.m.keys() {
            if !slots.contains(&k) {
                app.m.remove(k);
            }
        }
        app
    }

    pub fn semify_enode(&self, enode: L) -> L {
        enode.map_applied_ids(|app| self.semify_app_id(app))
    }

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
}
