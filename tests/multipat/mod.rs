#![allow(unused)]
#![allow(non_snake_case)]

use crate::*;
use std::collections::BTreeSet;

mod fuzz;
mod known_bugs;
mod props;
mod refine;
mod regress;

define_language! {
    pub enum MP {
        Var(Slot) = "var",
        F(AppliedId, AppliedId) = "f",
        G(AppliedId, AppliedId) = "g",
        H(AppliedId, AppliedId) = "h",
        K(AppliedId, AppliedId) = "k",
        P(AppliedId, AppliedId) = "p",
        Q(AppliedId, AppliedId) = "q",
        Sub(AppliedId, AppliedId) = "sub",
        Lam(Bind<AppliedId>) = "lam",
        App(AppliedId, AppliedId) = "app",
        Zero() = "zero",
    }
}

pub type MPGraph = EGraph<MP>;

pub fn mp_add(eg: &mut MPGraph, s: &str) -> AppliedId {
    eg.add_expr(RecExpr::parse(s).unwrap())
}

pub fn mp_union(eg: &mut MPGraph, a: &str, b: &str) {
    let x = mp_add(eg, a);
    let y = mp_add(eg, b);
    eg.union(&x, &y);
}

pub fn mp_lookup(eg: &MPGraph, s: &str) -> Option<AppliedId> {
    lookup_rec_expr(&RecExpr::parse(s).unwrap(), eg)
}

pub fn mp_eq(eg: &MPGraph, a: &str, b: &str) -> bool {
    match (mp_lookup(eg, a), mp_lookup(eg, b)) {
        (Some(x), Some(y)) => eg.eq(&x, &y),
        _ => false,
    }
}

/// Run to saturation: repeatedly find every match of `atoms` and, for each one,
/// union the class matched by `root` with the right-hand side built from that
/// match.
pub fn mp_saturate(eg: &mut MPGraph, atoms: &[&str], root: &str, rhs: &str) {
    let pat: MultiPattern<MP> = MultiPattern::parse(&atoms.join(", ")).unwrap();
    let from = Pattern::PVar(root.to_string());
    let to: Pattern<MP> = Pattern::parse(rhs).unwrap();
    for _ in 0..10 {
        let before = eg.progress();
        for s in multi_ematch(&pat, eg) {
            eg.union_instantiations(&from, &to, &s, None);
        }
        if before == eg.progress() {
            break;
        }
    }
}

/// Groups `probes` by which ones the e-graph considers equal, and adds a few
/// summary numbers.  Two runs can be compared by comparing these strings, which
/// avoids depending on the internal slot names, which are not stable.
pub fn mp_partition(eg: &MPGraph, probes: &[&str]) -> String {
    let ids: Vec<Option<AppliedId>> = probes.iter().map(|p| mp_lookup(eg, p)).collect();
    let mut groups: Vec<BTreeSet<usize>> = Vec::new();
    for i in 0..probes.len() {
        let Some(a) = &ids[i] else { continue };
        let mut placed = false;
        for g in groups.iter_mut() {
            let j = *g.iter().next().unwrap();
            if eg.eq(a, ids[j].as_ref().unwrap()) {
                g.insert(i);
                placed = true;
                break;
            }
        }
        if !placed {
            groups.push([i].into_iter().collect());
        }
    }
    let mut gs: Vec<String> = groups
        .iter()
        .map(|g| format!("{:?}", g.iter().copied().collect::<Vec<_>>()))
        .collect();
    gs.sort();
    let missing: Vec<usize> = (0..probes.len()).filter(|i| ids[*i].is_none()).collect();
    let mut slots: Vec<usize> = eg.ids().iter().map(|i| eg.slots(*i).len()).collect();
    slots.sort();
    format!(
        "{} missing{:?} slots{:?} nodes{}",
        gs.join(""),
        missing,
        slots,
        eg.total_number_of_nodes()
    )
}

/// Checks that every match is real: for each returned substitution and each
/// equation `?p == (f ?a ?b)`, rebuild the node from the substitution's values
/// for `?a` and `?b` and look it up.  It has to be in the class the substitution
/// gave `?p`.
pub fn mp_check_sound(eg: &MPGraph, atoms: &[&str], substs: &[Subst]) -> Result<(), String> {
    for s in substs {
        for a in atoms {
            let (lhs, rhs) = a.split_once("==").unwrap();
            let root = lhs.trim().trim_start_matches('?').to_string();
            let pat: Pattern<MP> = Pattern::parse(rhs.trim()).unwrap();
            let Pattern::ENode(tmpl, kids) = &pat else {
                continue;
            };
            let Some(bound) = s.get(&root) else { continue };

            let mut n = tmpl.clone();
            let mut ok = true;
            {
                let mut refs: Vec<&mut AppliedId> = n.applied_id_occurrences_mut();
                for (i, k) in kids.iter().enumerate() {
                    let Pattern::PVar(v) = k else {
                        ok = false;
                        break;
                    };
                    let Some(val) = s.get(v) else {
                        ok = false;
                        break;
                    };
                    *refs[i] = val.clone();
                }
            }
            if !ok {
                continue;
            }

            match eg.lookup(&n) {
                None => {
                    return Err(format!(
                        "atom `{a}`: the instantiated node is not in the e-graph\n  subst {s:?}"
                    ))
                }
                Some(found) => {
                    if !eg.eq(&found, bound) {
                        return Err(format!("atom `{a}`: the instantiated node is in a different class than ?{root}\n  subst {s:?}"));
                    }
                }
            }
        }
    }
    Ok(())
}
