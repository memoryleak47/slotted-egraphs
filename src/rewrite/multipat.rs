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

#[derive(Clone)]
struct MultiState {
    diseq_constraints: HashMap<Slot, Vec<Slot>>,
    subst: Subst,
}

pub fn multi_ematch<L: Language>(pat: &MultiPattern<L>, eg: &EGraph<L>) -> Vec<Subst> {
    let mut states: Vec<MultiState> = vec![MultiState {
        diseq_constraints: HashMap::default(),
        subst: Subst::default(),
    }];

    for (v, n, ch) in &pat.pats {
        for st in std::mem::take(&mut states) {
            states.extend(multi_ematch_step(v, n, ch, st, eg));
        }
    }

    // TODO final_subst?
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
        let slots = &eg.slots(x);
        let m = SlotMap::bijection_from_fresh_to(&slots).inverse();
        let mut state = state.clone();
        state.subst.insert(pv.clone(), AppliedId::new(x, m));
        out.push(state);
    }
    out
}

fn multi_ematch_step_node<L: Language>(pv: &PVar, node: &L, children: &[PVar], mut state: MultiState, eg: &EGraph<L>) -> Vec<MultiState> {
    let gid = &state.subst[pv];
    let mut out = Vec::new();

    for n in eg.enodes_applied(gid) {
        if !matches_raw(node, &n) { continue }

        let mut accum = vec![state.clone()];
        for (child_pvar, child_gid) in children.iter().zip(n.applied_id_occurrences()) {
            for st in std::mem::take(&mut accum) {
                accum.extend(extend_subst(child_pvar, child_gid.clone(), st));
            }
        }
        out.extend(accum);
    }

    out
}

fn matches_raw<L: Language>(n1: &L, n2: &L) -> bool {
    nullify_app_ids(n1) == nullify_app_ids(n2)
}

fn extend_subst(pv: &PVar, x: AppliedId, mut st: MultiState) -> Vec<MultiState> {
    if let Some(y) = st.subst.get(pv) {
        st.subst.insert(pv.clone(), x);
        vec![st]
    } else {
        todo!()
    }
}
