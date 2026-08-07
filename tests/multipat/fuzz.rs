//! Randomised testing: random e-graphs and random depth-1 multipatterns, with
//! three things checked about every result.
//!
//!   * Every match is real.  Take a returned substitution, rebuild each of the
//!     pattern's nodes out of it, and look that node up: it has to be in the
//!     class the substitution claimed it was in.
//!   * Nothing is lost by rewriting a nested pattern into depth-1 equations.  A
//!     random nested pattern and its rewritten form are each run to saturation;
//!     every equality the nested one proves must also be proved by the other.
//!     (The reverse is allowed to fail: the depth-1 matcher sees through
//!     redundant slots that the nested one does not.)
//!   * The e-graph stays well formed, via `eg.check()` after each round.
//!
//! The seed counts here run in a few seconds; `fuzz_long` is the same at a scale
//! worth running by hand, and is `#[ignore]`d.

use crate::multipat::*;
use crate::*;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 11
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
}

fn term(r: &mut Rng, depth: usize, nslots: usize) -> String {
    if depth == 0 || r.below(4) == 0 {
        return if r.below(3) == 0 {
            "zero".into()
        } else {
            format!("(var ${})", r.below(nslots))
        };
    }
    let a = term(r, depth - 1, nslots);
    let b = term(r, depth - 1, nslots);
    match r.below(5) {
        0 => format!("(f {a} {b})"),
        1 => format!("(g {a} {b})"),
        2 => format!("(sub {a} {b})"),
        3 => format!("(h {a} {b})"),
        _ => format!("(lam ${} {a})", r.below(nslots)),
    }
}

fn build(seed: u64, nslots: usize, nterms: usize, nunions: usize) -> MPGraph {
    let mut eg = MPGraph::default();
    let mut r = Rng(seed);
    for _ in 0..nterms {
        let t = term(&mut r, 3, nslots);
        eg.add_expr(RecExpr::parse(&t).unwrap());
    }
    for _ in 0..nunions {
        let a = term(&mut r, 2, nslots);
        let b = term(&mut r, 2, nslots);
        let x = eg.add_expr(RecExpr::parse(&a).unwrap());
        let y = eg.add_expr(RecExpr::parse(&b).unwrap());
        eg.union(&x, &y);
    }
    eg
}

/// generates a nested pattern together with its depth-1 flattening
struct Gen {
    atoms: Vec<String>,
    n: usize,
}
impl Gen {
    fn node(&mut self, r: &mut Rng, depth: usize, nslots: usize) -> (String, String) {
        self.n += 1;
        let me = format!("t{}", self.n);
        let (nested, atom) = if depth == 0 {
            if r.below(2) == 0 {
                ("zero".to_string(), format!("?{me} == zero"))
            } else {
                let s = r.below(nslots);
                (format!("(var ${s})"), format!("?{me} == (var ${s})"))
            }
        } else if r.below(6) == 0 {
            let s = r.below(nslots);
            let (cn, cv) = self.child(r, depth - 1, nslots);
            (
                format!("(lam ${s} {cn})"),
                format!("?{me} == (lam ${s} ?{cv})"),
            )
        } else {
            let (an, av) = self.child(r, depth - 1, nslots);
            let (bn, bv) = self.child(r, depth - 1, nslots);
            let f = ["f", "g", "h", "sub"][r.below(4)];
            (
                format!("({f} {an} {bn})"),
                format!("?{me} == ({f} ?{av} ?{bv})"),
            )
        };
        self.atoms.push(atom);
        (nested, me)
    }
    fn child(&mut self, r: &mut Rng, depth: usize, nslots: usize) -> (String, String) {
        let leaves = ["a", "b", "c"];
        if depth == 0 || r.below(2) == 0 {
            let v = leaves[r.below(leaves.len())];
            (format!("?{v}"), v.to_string())
        } else {
            self.node(r, depth, nslots)
        }
    }
}

fn nested_and_flat(seed: u64, nslots: usize) -> (String, Vec<String>, String) {
    let mut r = Rng(seed ^ 0xF00D);
    let mut g = Gen {
        atoms: Vec::new(),
        n: 0,
    };
    let (nested, root) = g.node(&mut r, 3, nslots);
    (nested, g.atoms, root)
}

fn random_multipattern(seed: u64, nslots: usize) -> Vec<String> {
    let mut r = Rng(seed);
    let pvars = ["a", "b", "c", "d"];
    let natoms = 1 + r.below(3);
    (0..natoms)
        .map(|i| {
            let root = if i == 0 {
                "p".to_string()
            } else {
                pvars[r.below(4)].to_string()
            };
            let x = pvars[r.below(4)];
            let y = pvars[r.below(4)];
            match r.below(6) {
                0 => format!("?{root} == (f ?{x} ?{y})"),
                1 => format!("?{root} == (g ?{x} ?{y})"),
                2 => format!("?{root} == (sub ?{x} ?{y})"),
                3 => format!("?{root} == (h ?{x} ?{y})"),
                4 => format!("?{root} == (lam ${} ?{x})", r.below(nslots)),
                _ => format!("?{root} == (var ${})", r.below(nslots)),
            }
        })
        .collect()
}

fn soundness(seeds: std::ops::Range<u64>) {
    for seed in seeds {
        let nslots = 2 + (seed as usize % 3);
        let atoms = random_multipattern(
            seed.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(12345),
            nslots,
        );
        let refs: Vec<&str> = atoms.iter().map(|x| x.as_str()).collect();
        let eg = build(seed, nslots, 4, 2);
        let mp: MultiPattern<MP> = MultiPattern::parse(&atoms.join(", ")).unwrap();
        let substs = multi_ematch(&mp, &eg);
        if let Err(e) = mp_check_sound(&eg, &refs, &substs) {
            panic!("seed {seed}\n  atoms {atoms:?}\n  {e}");
        }
    }
}

fn inclusion(seeds: std::ops::Range<u64>) {
    for seed in seeds {
        let nslots = 2 + (seed as usize % 2);
        let (nested, atoms, root) = nested_and_flat(seed, nslots);
        if atoms.len() > 6 {
            continue;
        }
        let refs: Vec<&str> = atoms.iter().map(|x| x.as_str()).collect();
        let probes: Vec<String> = {
            let mut r = Rng(seed ^ 0xBEEF);
            (0..6).map(|_| term(&mut r, 2, nslots)).collect()
        };
        let probe_refs: Vec<&str> = probes.iter().map(|x| x.as_str()).collect();

        let nested_eg = {
            let mut eg = build(seed, nslots, 4, 2);
            let rw = Rewrite::new("r", &nested, "zero");
            for _ in 0..5 {
                if !apply_rewrites(&mut eg, std::slice::from_ref(&rw)) {
                    break;
                }
            }
            eg
        };
        let multi_eg = {
            let mut eg = build(seed, nslots, 4, 2);
            mp_saturate(&mut eg, &refs, &root, "zero");
            eg
        };

        for i in 0..probe_refs.len() {
            for j in (i + 1)..probe_refs.len() {
                if mp_eq(&nested_eg, probe_refs[i], probe_refs[j]) {
                    assert!(
                        mp_eq(&multi_eg, probe_refs[i], probe_refs[j]),
                        "seed {seed}: nested proves {} = {} but the flattening does not\n  nested {nested}\n  atoms {atoms:?}",
                        probe_refs[i], probe_refs[j]
                    );
                }
            }
        }
    }
}

fn invariants(seeds: std::ops::Range<u64>) {
    for seed in seeds {
        let nslots = 2 + (seed as usize % 3);
        let (_, atoms, root) = nested_and_flat(seed, nslots);
        if atoms.len() > 6 {
            continue;
        }
        let refs: Vec<&str> = atoms.iter().map(|x| x.as_str()).collect();
        let mut eg = build(seed, nslots, 5, 3);
        mp_saturate(&mut eg, &refs, &root, "zero");
        eg.check();
    }
}

#[test]
fn fuzz_soundness() {
    soundness(0..400);
}

#[test]
fn fuzz_inclusion() {
    inclusion(0..300);
}

#[test]
fn fuzz_invariants() {
    invariants(0..200);
}

#[test]
#[ignore = "slow; run explicitly, ideally with --features checks"]
fn fuzz_long() {
    soundness(0..4000);
    inclusion(0..3000);
    invariants(0..1500);
}
