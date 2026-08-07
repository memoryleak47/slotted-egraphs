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

#[derive(Clone, Debug)]
struct MultiState {
    // There are two kinds of slots here:
    // 1. flexible slots, they come from some nodes with redundant variables, or from fresh slots allocated for some pvars.
    // 2. pattern slots, slots that come up in the actual pattern.

    // flexible slots are allowed to be replaced by other slots, pattern slots don't allow that.
    // this merging should always merge by choosing the pattern slots as leaders, as you want the final subst to contain the pattern slots.

    // flexible slots also spawn with disequality constraints among other flexible slots.
    // pattern slots can inherit disequality constraints from flexible slots though (without becoming flexible themselves).

    pattern_slots: HashSet<Slot>, // the set of pattern slots.
    diseq_constraints: HashMap<Slot, HashSet<Slot>>,
    subst: Subst,
    slot_uf: HashMap<Slot, Slot>,
}

pub fn multi_ematch<L: Language>(pat: &MultiPattern<L>, eg: &EGraph<L>) -> Vec<Subst> {
    let mut states: Vec<MultiState> = vec![MultiState {
        pattern_slots: HashSet::default(),
        diseq_constraints: HashMap::default(),
        subst: Subst::default(),
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
        add_disjointness_constraint(m.values().into_iter().collect(), &mut state);

        state.subst.insert(pv.clone(), AppliedId::new(x, m));
        out.push(state);
    }
    out
}

fn multi_ematch_step_node<L: Language>(pv: &PVar, node: &L, children: &[PVar], mut state: MultiState, eg: &EGraph<L>) -> Vec<MultiState> {
    let gid = &state.subst[pv];
    let mut out = Vec::new();

    for n in eg.enodes_applied(gid) {
        let mut state = state.clone();
        let set = n.all_slot_occurrences().into_iter().collect::<HashSet<Slot>>();
        add_disjointness_constraint(set, &mut state);

        let Some(mut state) = matches_raw(node, &n, state.clone()) else { continue };

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
    let n1 = nullify_app_ids(n1);
    let n2 = nullify_app_ids(n2);

    let (sh1, _) = n1.weak_shape();
    let (sh2, _) = n2.weak_shape();
    if sh1 != sh2 { return None }

    // as we've done nullify_app_ids, the only remaining slots are the slots not stored in AppliedIds.
    for (x1, y1) in n1.all_slot_occurrences().into_iter().zip(n2.all_slot_occurrences()) {
        st.pattern_slots.insert(x1);
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
fn allows_directed_union(x: Slot, st: &MultiState) -> bool {
    !st.pattern_slots.contains(&x)
}

// We replace x -> y, if allowed.
fn union_slot(x: Slot, y: Slot, mut st: MultiState) -> Option<MultiState> {
    let mut x = state_find(x, &st);
    let mut y = state_find(y, &st);

    if x == y { return Some(st) }

    if let Some(xx) = st.diseq_constraints.get(&x) { if xx.contains(&y) { return None } }
    if let Some(yy) = st.diseq_constraints.get(&y) { if yy.contains(&x) { return None } }

    if !allows_directed_union(x, &st) { (x, y) = (y, x); }
    if !allows_directed_union(x, &st) { return None }

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

    let mut diseq_constraints: HashMap<Slot, HashSet<Slot>> = HashMap::default();
    for (xx, vs) in st.diseq_constraints.iter() {
        let xx = state_find(*xx, st);
        let vs: Vec<Slot> = vs.iter().map(|a| state_find(*a, st)).collect();
        diseq_constraints.entry(xx).or_default().extend(vs);
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

fn add_disjointness_constraint(set: HashSet<Slot>, st: &mut MultiState) {
    for x in &set {
        let mut rest = set.clone();
        rest.remove(&x);
        st.diseq_constraints.entry(*x).or_default().extend(rest);
    }
}
