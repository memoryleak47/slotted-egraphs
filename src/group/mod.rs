use crate::*;

mod perm;
pub use perm::*;

#[cfg(test)]
mod tst;

// In order to be compatible with the literature:
// https://en.wikipedia.org/wiki/Schreier%27s_lemma
// I define "x y" = x.compose(y)

// Trivial implementation of permutation Group.
#[derive(Clone, Debug)]
pub struct Group<P: Permutation> {
    // all perms are bijections : omega -> omega.
    // omega = keys(identity) = values(identity).
    identity: P,
    next: Option<Box<Next<P>>>,
}

#[derive(Clone, Debug)]
struct Next<P: Permutation> {
    // the Slot we are stabilizing
    stab: Slot,

    // the orbit tree.
    // ot[x] is a perm that maps stab to x.
    ot: HashMap<Slot, P>,

    g: Group<P>,
}

impl<P: Permutation> Group<P> {
    pub fn new(identity: &P, generators: HashSet<P>) -> Self {
        let identity = identity.clone();
        let next = find_lowest_nonstab(&generators)
                    .map(|s| Box::new(Next::new(s, &identity, generators)));
        Group { identity, next }
    }

    pub fn is_trivial(&self) -> bool {
        self.next.is_none()
    }

    pub fn identity(identity: &P) -> Self {
        Self::new(identity, HashSet::default())
    }

    pub fn orbit(&self, s: Slot) -> HashSet<Slot> {
        build_ot(s, &self.identity, &self.generators())
            .keys()
            .cloned()
            .collect()
    }

    fn generators_impl(&self) -> HashSet<P> {
        match &self.next {
            None => HashSet::default(),
            Some(n) => &n.ot.values().cloned().collect::<HashSet<_>>() | &n.g.generators_impl(),
        }
    }

    pub fn generators(&self) -> HashSet<P> {
        let mut out = self.generators_impl();
        out.remove(&self.identity);
        out
    }

    // Should be very rarely called.
    pub fn all_perms(&self) -> HashSet<P> {
        match &self.next {
            None => [self.identity.clone()].into_iter().collect(),
            Some(n) => {
                let mut out = HashSet::default();

                let left = n.ot.values().cloned().collect::<HashSet<_>>();
                let right = n.g.all_perms();

                for l in &left {
                    for r in &right {
                        out.insert(r.compose(l));
                    }
                }

                out
            }
        }
    }

    pub fn contains(&self, p: &SlotMap) -> bool {
        match &self.next {
            None => p.iter().all(|(x, y)| x == y),
            Some(n) => {
                let Some(part) = &n.ot.get(&p[n.stab]) else { return false };
                n.g.contains(&p.compose(&part.inverse().to_slotmap()))
            },
        }
    }

    pub fn proven_contains(&self, p: &SlotMap) -> Option<P> {
        match &self.next {
            None if p.iter().all(|(x, y)| x == y) => Some(self.identity.clone()),
            None => None,
            Some(n) => {
                let part = &n.ot.get(&p[n.stab])?;
                let step = n.g.proven_contains(&p.compose(&part.inverse().to_slotmap()))?;
                // step == p * part^-1
                // -> step * part == p
                let out = step.compose(part);
                Some(out)
            },
        }
    }

    pub fn add(&mut self, p: P) -> bool {
        self.add_set([p].into_iter().collect())
    }

    pub fn add_set(&mut self, mut perms: HashSet<P>) -> bool {
        // There might be ways to make this faster, by iterating through the stab chain and determining at which layer this perm actually has an effect.
        // But it's polytime, so fast enough I guess.

        perms.retain(|x| !self.contains(&x.to_slotmap()));

        if !perms.is_empty() {
            *self = Group::new(&self.identity, &self.generators() | &perms);

            true
        } else { false }
    }

    pub fn count(&self) -> usize {
        match &self.next {
            None => 1,
            Some(n) => n.ot.len() * n.g.count(),
        }
    }
}

impl<P: Permutation> Next<P> {
    fn new(stab: Slot, identity: &P, generators: HashSet<P>) -> Self {
        let ot = build_ot(stab, identity, &generators);
        let generators = schreiers_lemma(stab, &ot, generators);
        let g = Group::new(identity, generators);
        Next { stab, ot, g }
    }
}

fn build_ot<P: Permutation>(stab: Slot, identity: &P, generators: &HashSet<P>) -> HashMap<Slot, P> {
    let mut ot = HashMap::default();
    ot.insert(stab, identity.clone());

    loop {
        let len = ot.len();

        for g in generators {
            for (_, v) in ot.clone() {
                let new = v.compose(g);
                let target = new[stab];
                if !ot.contains_key(&target) {
                    ot.insert(target, new);
                }
            }
        }

        let len2 = ot.len();
        if len == len2 { break; }
    }

    ot
}

// extends the set of generators using Schreiers Lemma.
fn schreiers_lemma<P: Permutation>(stab: Slot, ot: &HashMap<Slot, P>, generators: HashSet<P>) -> HashSet<P> {
    let mut out = HashSet::default();
    for (_, r) in ot {
        for s in &generators {
            let rs = r.compose(s);
            let rs2_inv = ot[&rs[stab]].inverse();
            out.insert(rs.compose(&rs2_inv));
        }
    }
    out
}

// finds the lowest Slot that's not stabilized in at least one of the generators.
fn find_lowest_nonstab<P: Permutation>(generators: &HashSet<P>) -> Option<Slot> {
    let mut min = None;
    for gen in generators {
        for (x, y) in gen.iter() {
            if x != y {
                min = min.iter()
                         .copied()
                         .chain(std::iter::once(x))
                         .min();
            }
        }
    }
    min
}
