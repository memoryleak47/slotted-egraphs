use crate::*;

pub type PVar = String; // TODO this should be interned, or better: index-based.

pub struct MultiPattern<L: Language> {
    // covers equations like `= ?a (f ?b ?c)`.
    // we require them to have nesting depth exactly one.
    // This is not a restriction:
    // - nesting depth 0 `(= ?a ?b)` can be solved via pre-processing, and
    // - nesting depth >1 `(= ?a (f (f ?x)))` can be solved via flattening `(= ?a (f ?b)), (= ?b (f ?x))`.
    // variables are allowed to come up multiple times on the left and right.
    pub(crate) pats: Vec<(PVar, L, Vec<PVar>)>,
}

#[derive(Clone, Copy, Debug)]
enum SlotKind {
    Redundant,
    Global,
    Pattern,
}

#[derive(Clone, Debug)]
struct MultiState {
    // There are three kinds of slots here:
    // 1. redundant slots, they come from some nodes with redundant variables.
    // 2. global slots, which are the fresh slots that we renamed some pvar to point to.
    // 3. pattern slots, slots that come up in the actual pattern.

    // They form a hierarchy. Any lower slot can be merged into a higher slot, but not the other way around.
    // redundant slots are allowed to alias other redundant slots, global slots and pattern slots.
    // global slots are allowed to alias other global slots and pattern slots.
    // pattern slots can't alias anything.
    // the slot_uf needs to respect this directionality.

    slot_kind: HashMap<Slot, SlotKind>,
    diseq_constraints: HashMap<Slot, Vec<Slot>>,
    subst: Subst,
    slot_uf: HashMap<Slot, Slot>,
}

pub fn multi_ematch<L: Language>(pat: &MultiPattern<L>, eg: &EGraph<L>) -> Vec<Subst> {
    let mut states: Vec<MultiState> = vec![MultiState {
        diseq_constraints: HashMap::default(),
        subst: Subst::default(),
        slot_kind: HashMap::default(),
        slot_uf: HashMap::default(),
    }];

    for (v, n, ch) in &pat.pats {
        for st in std::mem::take(&mut states) {
            states.extend(multi_ematch_step(v, n, ch, st, eg));
        }
    }

    states.into_iter().map(|x| x.subst).collect()
}

fn multi_ematch_step<L: Language>(pv: &PVar, node: &L, children: &[PVar], mut state: MultiState, eg: &EGraph<L>) -> Vec<MultiState> {
    let mut out = Vec::new();
    for state in multi_ematch_step_class(pv, node, children, state, eg) {
        out.extend(multi_ematch_step_node(pv, node, children, state, eg));
    }
    out
}


fn multi_ematch_step_class<L: Language>(pv: &PVar, node: &L, children: &[PVar], mut state: MultiState, eg: &EGraph<L>) -> Vec<MultiState> {
    if state.subst.contains_key(pv) { return vec![state] }

    let mut out = Vec::new();
    for x in eg.ids() {
        let mut state = state.clone();

        let slots = &eg.slots(x);
        let m = SlotMap::bijection_from_fresh_to(&slots).inverse();
        for xx in m.values() { state.slot_kind.insert(xx, SlotKind::Global); }
        state.subst.insert(pv.clone(), AppliedId::new(x, m));
        out.push(state);
    }
    out
}

fn multi_ematch_step_node<L: Language>(pv: &PVar, node: &L, children: &[PVar], mut state: MultiState, eg: &EGraph<L>) -> Vec<MultiState> {
    let gid = &state.subst[pv];
    let mut out = Vec::new();

    for n in eg.enodes_applied(gid) {
        let Some(mut state) = matches_raw(node, &n, state.clone()) else { continue };

        for slot in n.all_slot_occurrences().into_iter().collect::<HashSet<Slot>>() {
            if !gid.m.values().contains(&slot) {
                // At this point, we know that `slot` is a fresh slot coming from some redundant variable.
                state.diseq_constraints.insert(slot, gid.m.values().into_iter().collect());
                state.slot_kind.insert(slot, SlotKind::Redundant);
            }
        }

        let mut accum = vec![state];
        for (child_pvar, child_gid) in children.iter().zip(n.applied_id_occurrences()) {
            for st in std::mem::take(&mut accum) {
                accum.extend(extend_subst(child_pvar, child_gid.clone(), st, eg));
            }
        }
        out.extend(accum);
    }

    out
}

// n1 comes from the pattern, whereas n2 from the e-graph.
fn matches_raw<L: Language>(n1: &L, n2: &L, mut st: MultiState) -> Option<MultiState> {
    let (n1, _) = nullify_app_ids(n1).weak_shape();
    let (n2, _) = nullify_app_ids(n2).weak_shape();
    if n1 != n2 { return None }

    for (x1, y1) in n1.all_slot_occurrences().into_iter().zip(n2.all_slot_occurrences()) {
        st.slot_kind.insert(x1, SlotKind::Pattern);
        st = union_slot(x1, y1, st)?;
    }
    Some(st)
}

fn extend_subst<L: Language>(pv: &PVar, x: AppliedId, mut st: MultiState, eg: &EGraph<L>) -> Vec<MultiState> {
    if let Some(y) = st.subst.get(pv).cloned() {
        unify(&x, &y, st, eg)
    } else {
        st.subst.insert(pv.clone(), x);
        vec![st]
    }
}

fn unify<L: Language>(x: &AppliedId, y: &AppliedId, mut st: MultiState, eg: &EGraph<L>) -> Vec<MultiState> {
    let x = &state_appid_find(x.clone(), &st);
    let y = &state_appid_find(y.clone(), &st);

    if x.id != y.id { return Vec::new() }

    let xslots: HashSet<Slot> = x.m.values().iter().copied().collect();
    let yslots: HashSet<Slot> = y.m.values().iter().copied().collect();
    let xonly = &xslots - &yslots;
    let yonly = &yslots - &xslots;

    assert_eq!(xonly.len(), yonly.len());
    if xonly.is_empty() {
        if eg.eq(x, y) {
            vec![st]
        } else {
            Vec::new()
        }
    } else {
        let &xx = xonly.iter().next().unwrap();
        let mut out = Vec::new();

        for &yy in yonly.iter() {
            let st = st.clone();
            if let Some(st) = union_slot(xx, yy, st) {
                out.extend(unify(x, y, st, eg));
            }
        }

        out
    }
}

// whether we allow x -> y replacement.
fn allows_directed_union(x: Slot, y: Slot, st: &MultiState) -> bool {
    match (st.slot_kind[&x], st.slot_kind[&y]) {
        (SlotKind::Redundant, _) => true,
        (SlotKind::Pattern, _) => false,
        (SlotKind::Global, SlotKind::Global|SlotKind::Pattern) => true,
        (SlotKind::Global, SlotKind::Redundant) => false,
    }
}

// We replace x -> y, if allowed.
fn union_slot(x: Slot, y: Slot, mut st: MultiState) -> Option<MultiState> {
    let mut x = state_find(x, &st);
    let mut y = state_find(y, &st);

    if x == y { return Some(st) }

    if let Some(xx) = st.diseq_constraints.get(&x) { if xx.contains(&y) { return None } }
    if let Some(yy) = st.diseq_constraints.get(&y) { if yy.contains(&x) { return None } }

    if !allows_directed_union(x, y, &st) { (x, y) = (y, x); }
    if !allows_directed_union(x, y, &st) { return None }

    st.slot_uf.insert(x, y);

    update_state(&mut st);
    Some(st)
}

fn update_state(st: &mut MultiState) {
    let mut subst = st.subst.clone();
    for v in subst.values_mut() {
        *v = state_appid_find(v.clone(), st);
    }
    st.subst = subst;

    let mut diseq_constraints = HashMap::default();
    for (xx, vs) in st.diseq_constraints.iter() {
        let xx = state_find(*xx, st);
        let vs: Vec<Slot> = vs.iter().map(|a| state_find(*a, st)).collect();
        let vv: &mut Vec<_> = diseq_constraints.entry(xx).or_default();
        vv.extend(vs);
        vv.sort();
        vv.dedup();
    }

    st.diseq_constraints = diseq_constraints;
}

fn state_find(mut x: Slot, st: &MultiState) -> Slot {
    while let Some(y) = st.slot_uf.get(&x) {
        x = *y;
    }
    x
}

fn state_appid_find(mut x: AppliedId, st: &MultiState) -> AppliedId {
    for xx in x.m.values_mut() {
        *xx = state_find(*xx, st);
    }
    x
}
