use crate::*;

use std::collections::HashMap;

// In order to be compatible with the literature:
// https://en.wikipedia.org/wiki/Schreier%27s_lemma
// I define "x y" = x.compose(y)

type OT = [Option<SlotMap>];

#[derive(Clone, Debug)]
pub struct Group {
    // all perms are bijections : omega -> omega.
    // omega = keys(identity) = values(identity).
    // omega = 0..arity
    arity: usize,
    next: Option<Box<Next>>,
}

#[derive(Clone, Debug)]
struct Next {
    // the Slot we are stabilizing
    stab: Slot,

    // the orbit tree.
    // ot[x] is a perm that maps stab to x.
    ot: Box<OT>,

    g: Group,
}

impl Group {
    pub fn new(arity: usize, generators: HashSet<SlotMap>) -> Self {
        let next = find_lowest_nonstab(&generators).map(|s| Box::new(Next::new(s, arity, generators)));
        Group { arity, next }
    }

    pub fn is_trivial(&self) -> bool {
        self.next.is_none()
    }

    pub fn trivial(arity: usize) -> Self {
        Self::new(arity, HashSet::default())
    }

    pub fn orbit(&self, s: Slot) -> HashSet<Slot> {
        let gens = self.generators();
        let mut orbit: HashSet<Slot> = HashSet::new();
        orbit.insert(s);

        loop {
            let n = orbit.len();
            for slot in orbit.clone() {
                for g in &gens {
                    orbit.insert(g[slot]);
                }
            }
            if orbit.len() == n { return orbit; }
        }
    }

    fn generators_impl(&self) -> HashSet<SlotMap> {
        match &self.next {
            None => HashSet::default(),
            Some(n) => &n.ot.iter().cloned().filter_map(|x| x).collect::<HashSet<_>>() | &n.g.generators_impl(),
        }
    }

    pub fn generators(&self) -> HashSet<SlotMap> {
        let mut out = self.generators_impl();
        out.remove(&SlotMap::identity(self.arity));
        out
    }

    pub fn contains(&self, p: &SlotMap) -> bool {
        match &self.next {
            None => p.is_identity(),
            Some(n) => {
                let Some(part) = &n.ot[p[n.stab].n as usize] else { return false };
                n.g.contains(&p.compose(&part.inverse()))
            }
        }
    }

    pub fn count(&self) -> usize {
        match &self.next {
            None => 1,
            Some(n) => n.ot.len() * n.g.count(),
        }
    }
}

impl Next {
    fn new(stab: Slot, arity: usize, generators: HashSet<SlotMap>) -> Self {
        let ot = build_ot(stab, arity, &generators);
        let generators = schreiers_lemma(stab, &*ot, generators);
        let g = Group::new(arity, generators);
        Next { stab, ot, g }
    }
}

fn build_ot(stab: Slot, arity: usize, generators: &HashSet<SlotMap>) -> Box<OT> {
    let mut ot: Box<OT> = vec![None; arity].into_boxed_slice();
    ot[stab.n as usize] = Some(SlotMap::identity(arity));

    loop {
        let len = ot.len();

        for g in generators {
            for v in ot.clone().iter().filter_map(|x| x.as_ref()) {
                let new = v.compose(g);
                let target = new[stab];
                if ot[target.n as usize].is_none() {
                    ot[target.n as usize] = Some(new);
                }
            }
        }

        let len2 = ot.len();
        if len == len2 { break }
    }

    ot
}

// extends the set of generators using Schreiers Lemma.
fn schreiers_lemma(stab: Slot, ot: &OT, generators: HashSet<SlotMap>) -> HashSet<SlotMap> {
    let mut out = HashSet::default();
    for r in ot {
        let Some(r) = r else { continue };
        for s in &generators {
            let rs = r.compose(s);
            let Some(rs2) = &ot[rs[stab].n as usize] else { continue };
            let rs2_inv = rs2.inverse();
            out.insert(rs.compose(&rs2_inv));
        }
    }
    out
}

// finds the lowest Slot that's not stabilized in at least one of the generators.
fn find_lowest_nonstab(generators: &HashSet<SlotMap>) -> Option<Slot> {
    let mut min = None;
    for gen_ in generators {
        for (x, y) in gen_.slice.iter().enumerate() {
            let x = Slot { n: x as u32 };
            if x != *y {
                min = min.iter().copied().chain(std::iter::once(x)).min();
            }
        }
    }
    min
}

