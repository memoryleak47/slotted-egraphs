use crate::*;

pub type Subst = HashMap<String, AppliedId>;

#[derive(Default, Clone)]
struct State {
    // uses egraph slots.
    partial_subst: Subst,

    // maps from the egraph slots to the pattern slots.
    partial_slotmap: SlotMap,
}

pub fn ematch_all<L: Language, N: Analysis<L>>(
    eg: &EGraph<L, N>,
    pattern: &PatternAst<L>,
) -> Vec<Subst> {
    let mut out = Vec::new();
    for i in eg.ids() {
        let i = eg.mk_sem_identity_applied_id(i);
        out.extend(
            ematch_impl(pattern, State::default(), i, eg)
                .into_iter()
                .map(final_subst),
        );
    }
    out
}

// `i` uses egraph slots instead of pattern slots.
fn ematch_impl<L: Language, N: Analysis<L>>(
    pattern: &PatternAst<L>,
    st: State,
    i: AppliedId,
    eg: &EGraph<L, N>,
) -> Vec<State> {
    match &pattern {
        PatternAst::PVar(v) => {
            let mut st = st;
            if let Some(j) = st.partial_subst.get(v) {
                if !eg.eq(&i, j) {
                    return Vec::new();
                }
            } else {
                st.partial_subst.insert(v.clone(), i);
            }
            vec![st]
        }
        PatternAst::ENode(n, children) => {
            let mut out = Vec::new();
            for nn in eg.enodes_applied(&i) {
                let d = std::mem::discriminant(n);
                let dd = std::mem::discriminant(&nn);
                if d != dd {
                    continue;
                };

                ematch_node(&st, eg, &n, children, &mut out, &nn);
            }
            out
        }
        PatternAst::Subst(..) => panic!(),
    }
}

fn ematch_node<L: Language, N: Analysis<L>>(
    st: &State,
    eg: &EGraph<L, N>,
    n: &L,
    children: &[PatternAst<L>],
    out: &mut Vec<State>,
    nn: &L,
) {
    'nodeloop: for n2 in eg.get_group_compatible_weak_variants(&nn) {
        if CHECKS {
            assert_eq!(&nullify_app_ids(n), n);
        }

        let clear_n2 = nullify_app_ids(&n2);
        // We can use weak_shape here, as the inputs are nullified
        // i.e. they only have id0() without slot args, so there are no permutations possible.
        let (n_sh, _) = n.weak_shape();
        let (clear_n2_sh, _) = clear_n2.weak_shape();
        if n_sh != clear_n2_sh {
            continue 'nodeloop;
        }

        let mut st = st.clone();

        for (x, y) in clear_n2
            .all_slot_occurrences()
            .into_iter()
            .zip(n.all_slot_occurrences().into_iter())
        {
            if !try_insert_compatible_slotmap_bij(x, y, &mut st.partial_slotmap) {
                continue 'nodeloop;
            }
        }

        let mut acc = vec![st];
        for (sub_id, sub_pat) in n2.applied_id_occurrences().into_iter().zip(children.iter()) {
            let mut next = Vec::new();
            for a in acc {
                next.extend(ematch_impl(sub_pat, a, sub_id.clone(), eg));
            }
            acc = next;
        }

        out.extend(acc);
    }
}

pub(crate) fn nullify_app_ids<L: Language>(l: &L) -> L {
    let mut l = l.clone();
    for x in l.applied_id_occurrences_mut() {
        *x = AppliedId::null();
    }
    l
}

fn try_insert_compatible_slotmap_bij(k: Slot, v: Slot, map: &mut SlotMap) -> bool {
    if let Some(v_old) = map.get(k) {
        if v_old != v {
            return false;
        }
    }
    map.insert(k, v);
    map.is_bijection()
}

fn final_subst(s: State) -> Subst {
    let State {
        partial_subst: mut subst,
        partial_slotmap: mut slotmap,
    } = s;

    // Previously, the subst uses `egraph`-based slot names.
    // Afterwards, the subst uses `pattern`-based slot names.
    for (_, v) in subst.iter_mut() {
        // All slots that are not covered by the pattern, need a fresh new name.
        for s in v.slots() {
            if !slotmap.contains_key(s) {
                slotmap.insert(s, Slot::fresh());
            }
        }

        *v = v.apply_slotmap(&slotmap);
    }

    subst
}
