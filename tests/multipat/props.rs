//! Properties multipattern matching should have whatever the implementation
//! does: the result must not depend on the order the atoms are written in, on
//! the names of the slots in the program, or on an atom being repeated.  And a
//! nested pattern must not prove more than its depth-1 flattening.

use crate::multipat::*;
use crate::*;

fn perms<T: Clone>(v: &[T]) -> Vec<Vec<T>> {
    if v.len() <= 1 {
        return vec![v.to_vec()];
    }
    let mut out = Vec::new();
    for i in 0..v.len() {
        let mut rest = v.to_vec();
        let x = rest.remove(i);
        for mut p in perms(&rest) {
            p.insert(0, x.clone());
            out.push(p);
        }
    }
    out
}

fn order_independent(
    build: impl Fn(&mut MPGraph),
    atoms: &[&str],
    root: &str,
    rhs: &str,
    probes: &[&str],
) {
    let run = |atoms: &[&str]| {
        let mut eg = MPGraph::default();
        build(&mut eg);
        mp_saturate(&mut eg, atoms, root, rhs);
        mp_partition(&eg, probes)
    };
    let base = run(atoms);
    for p in perms(atoms) {
        assert_eq!(base, run(&p), "atom order {p:?} changed the result");
    }
}

#[test]
fn order_symmetry_join() {
    order_independent(
        |eg| {
            mp_union(eg, "(k (var $0) (var $1))", "(k (var $1) (var $0))");
            mp_add(eg, "(f (k (var $0) (var $1)) (var $2))");
            mp_add(eg, "(g (k (var $1) (var $0)) (var $2))");
        },
        &["?p == (f ?a ?b)", "?q == (g ?a ?b)"],
        "p",
        "zero",
        &[
            "(f (k (var $0) (var $1)) (var $2))",
            "(g (k (var $0) (var $1)) (var $2))",
            "zero",
        ],
    );
}

#[test]
fn order_three_level_flattening() {
    order_independent(
        |eg| {
            mp_union(eg, "(sub (var $9) (var $9))", "zero");
            mp_add(eg, "(g (f zero (var $0)) (var $1))");
        },
        &["?p == (g ?q ?z)", "?q == (f ?r ?y)", "?r == (sub ?x ?x)"],
        "p",
        "(h ?y ?z)",
        &["(g (f zero (var $0)) (var $1))", "(h (var $0) (var $1))"],
    );
}

#[test]
fn order_redundancy_join() {
    order_independent(
        |eg| {
            mp_union(eg, "(k (var $0) (var $1))", "zero");
            mp_add(eg, "(f zero (var $2))");
            mp_add(eg, "(g zero (var $2))");
        },
        &["?p == (f ?a ?b)", "?q == (g ?a ?b)", "?a == (k ?u ?v)"],
        "p",
        "(h ?u ?v)",
        &["(f zero (var $2))", "(h (var $0) (var $1))", "zero"],
    );
}

/// A three-cycle group: the join's two atoms differ by the square of the cycle.
#[test]
fn order_three_cycle_group() {
    order_independent(
        |eg| {
            mp_union(
                eg,
                "(p (p (var $0) (var $1)) (var $2))",
                "(p (p (var $1) (var $2)) (var $0))",
            );
            mp_add(eg, "(f (p (p (var $0) (var $1)) (var $2)) (var $3))");
            mp_add(eg, "(g (p (p (var $2) (var $0)) (var $1)) (var $3))");
        },
        &["?m == (f ?a ?b)", "?n == (g ?a ?b)"],
        "m",
        "zero",
        &["(f (p (p (var $0) (var $1)) (var $2)) (var $3))", "zero"],
    );
}

fn shift(s: &str, off: i64) -> String {
    let b: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == '$' && i + 1 < b.len() && b[i + 1].is_ascii_digit() {
            let mut j = i + 1;
            let mut n = 0i64;
            while j < b.len() && b[j].is_ascii_digit() {
                n = n * 10 + b[j] as i64 - '0' as i64;
                j += 1;
            }
            out.push('$');
            out.push_str(&(n + off).to_string());
            i = j;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    out
}

fn renaming_invariant(
    build: impl Fn(&mut MPGraph, i64),
    atoms: &[&str],
    root: &str,
    rhs: &str,
    probes: &[&str],
) {
    let run = |off: i64| {
        let mut eg = MPGraph::default();
        build(&mut eg, off);
        let a: Vec<String> = atoms.iter().map(|x| shift(x, off)).collect();
        let a: Vec<&str> = a.iter().map(|x| x.as_str()).collect();
        mp_saturate(&mut eg, &a, root, rhs);
        let p: Vec<String> = probes.iter().map(|x| shift(x, off)).collect();
        let p: Vec<&str> = p.iter().map(|x| x.as_str()).collect();
        mp_partition(&eg, &p)
    };
    assert_eq!(run(0), run(40), "renaming every slot changed the result");
}

#[test]
fn renaming_symmetry() {
    renaming_invariant(
        |eg, o| {
            mp_union(
                eg,
                &shift("(k (var $0) (var $1))", o),
                &shift("(k (var $1) (var $0))", o),
            );
            mp_add(eg, &shift("(f (k (var $0) (var $1)) (var $2))", o));
        },
        &["?p == (f ?a ?b)", "?a == (k ?u ?v)"],
        "p",
        "(h ?u ?v)",
        &[
            "(f (k (var $0) (var $1)) (var $2))",
            "(h (var $0) (var $1))",
            "(h (var $1) (var $0))",
        ],
    );
}

#[test]
fn renaming_redundancy() {
    renaming_invariant(
        |eg, o| {
            mp_union(eg, &shift("(sub (var $9) (var $9))", o), "zero");
            mp_add(eg, &shift("(f (var $0) zero)", o));
        },
        &["?p == (f ?x ?q)", "?q == (sub ?x ?x)"],
        "p",
        "zero",
        &["(f (var $0) zero)", "zero"],
    );
}

#[test]
fn duplicate_atom_is_idempotent() {
    let build = |eg: &mut MPGraph| {
        mp_union(eg, "(k (var $0) (var $1))", "(k (var $1) (var $0))");
        mp_add(eg, "(f (k (var $0) (var $1)) (var $2))");
    };
    let probes = [
        "(f (k (var $0) (var $1)) (var $2))",
        "(h (var $0) (var $1))",
        "(h (var $1) (var $0))",
    ];
    let run = |atoms: &[&str]| {
        let mut eg = MPGraph::default();
        build(&mut eg);
        mp_saturate(&mut eg, atoms, "p", "(h ?u ?v)");
        mp_partition(&eg, &probes)
    };
    assert_eq!(
        run(&["?p == (f ?a ?b)", "?a == (k ?u ?v)"]),
        run(&["?p == (f ?a ?b)", "?a == (k ?u ?v)", "?a == (k ?u ?v)"]),
    );
}

/// A nested pattern and its depth-1 flattening: everything the nested matcher
/// proves, the flattened multipattern must prove too.  (Multipattern matching is
/// allowed to prove strictly more -- it sees through redundancies that
/// `ematch_all` does not.)
fn flattening_proves_at_least_as_much(
    build: impl Fn(&mut MPGraph),
    nested: &str,
    atoms: &[&str],
    root: &str,
    rhs: &str,
    probes: &[&str],
) {
    let nested_eg = {
        let mut eg = MPGraph::default();
        build(&mut eg);
        let rw = Rewrite::new("r", nested, rhs);
        for _ in 0..10 {
            if !apply_rewrites(&mut eg, std::slice::from_ref(&rw)) {
                break;
            }
        }
        eg
    };
    let multi_eg = {
        let mut eg = MPGraph::default();
        build(&mut eg);
        mp_saturate(&mut eg, atoms, root, rhs);
        eg
    };
    for i in 0..probes.len() {
        for j in (i + 1)..probes.len() {
            if mp_eq(&nested_eg, probes[i], probes[j]) {
                assert!(
                    mp_eq(&multi_eg, probes[i], probes[j]),
                    "nested proves {} = {} but the flattening does not",
                    probes[i],
                    probes[j]
                );
            }
        }
    }
}

#[test]
fn flattening_symmetry_with_repeated_pvar() {
    flattening_proves_at_least_as_much(
        |eg| {
            mp_union(eg, "(k (var $0) (var $1))", "(k (var $1) (var $0))");
            mp_add(eg, "(f (k (var $0) (var $1)) (k (var $1) (var $0)))");
            mp_add(eg, "zero");
        },
        "(f ?x ?x)",
        &["?p == (f ?x ?x)"],
        "p",
        "zero",
        &[
            "(f (k (var $0) (var $1)) (k (var $1) (var $0)))",
            "zero",
            "(k (var $0) (var $1))",
        ],
    );
}

#[test]
fn flattening_two_levels() {
    flattening_proves_at_least_as_much(
        |eg| {
            mp_union(eg, "(k (var $0) (var $1))", "(k (var $1) (var $0))");
            mp_add(eg, "(g (f (k (var $0) (var $1)) (var $2)) (var $3))");
        },
        "(g (f ?x ?y) ?z)",
        &["?p == (g ?q ?z)", "?q == (f ?x ?y)"],
        "p",
        "(h ?x ?z)",
        &[
            "(g (f (k (var $0) (var $1)) (var $2)) (var $3))",
            "(h (k (var $0) (var $1)) (var $3))",
            "(h (k (var $1) (var $0)) (var $3))",
        ],
    );
}

#[test]
fn flattening_through_a_binder() {
    flattening_proves_at_least_as_much(
        |eg| {
            mp_add(eg, "(lam $0 (app (var $0) (var $1)))");
            mp_add(eg, "(lam $2 (app (var $2) (var $3)))");
        },
        "(lam $v (app ?b (var $v)))",
        &["?p == (lam $v ?q)", "?q == (app ?b ?c)", "?c == (var $v)"],
        "p",
        "?b",
        &[
            "(lam $0 (app (var $0) (var $1)))",
            "(var $1)",
            "(lam $2 (app (var $2) (var $3)))",
        ],
    );
}

#[test]
fn flattening_redundancy_three_levels() {
    flattening_proves_at_least_as_much(
        |eg| {
            mp_union(eg, "(sub (var $9) (var $9))", "zero");
            mp_add(eg, "(g (f zero (var $0)) (var $1))");
        },
        "(g (f (sub ?x ?x) ?y) ?z)",
        &["?p == (g ?q ?z)", "?q == (f ?r ?y)", "?r == (sub ?x ?x)"],
        "p",
        "(h ?y ?z)",
        &["(g (f zero (var $0)) (var $1))", "(h (var $0) (var $1))"],
    );
}
