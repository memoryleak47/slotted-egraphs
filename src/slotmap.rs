use crate::*;
use std::ops::Index;

// Permutations are a special kind of Bijections.
// Their key & value sets agree!
pub(crate) type Perm = Bijection;

// Bijections are bijective SlotMaps.
pub(crate) type Bijection = SlotMap;

#[derive(Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Default)]
/// A mapping between the parameter-slots of an e-class, and some "invocation" slots that you want to put into them.
///
/// This type is relevant for [AppliedId]s.
pub struct SlotMap {
    // if (l, r) in map, then there is no (l, r') in map. Each key is uniquely contained.
    // Also: map is sorted by their keys.
    map: Vec<(Slot, Slot)>,
}

impl SlotMap {
    pub fn new() -> Self {
        SlotMap {
            map: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn search(&self, l: Slot) -> Result<usize, usize> {
        self.map.binary_search_by_key(&l, |(x, _)| *x)
    }

    pub fn contains_key(&self, k: Slot) -> bool {
        self.get(k).is_some()
    }

    pub fn insert(&mut self, l: Slot, r: Slot) {
        match self.search(l) {
            Ok(i) => {
                self.map[i] = (l, r);
            }
            Err(i) => {
                self.map.insert(i, (l, r));
            }
        }
    }

    pub fn get(&self, l: Slot) -> Option<Slot> {
        self.search(l).ok().map(|i| self.map[i].1)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Slot, Slot)> + '_ {
        self.map.iter().copied()
    }

    pub fn into_iter(self) -> impl Iterator<Item = (Slot, Slot)> {
        self.map.into_iter()
    }

    // ordered by their keys!
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut Slot> + '_ {
        self.map.iter_mut().map(|(_, y)| y)
    }

    pub fn values_immut(&self) -> impl Iterator<Item = &Slot> + '_ {
        self.map.iter().map(|(_, y)| y)
    }

    pub fn keys(&self) -> SmallHashSet<Slot> {
        self.iter().map(|(x, _)| x).collect()
    }
    pub fn values(&self) -> SmallHashSet<Slot> {
        self.iter().map(|(_, y)| y).collect()
    }
    pub fn keys_vec(&self) -> Vec<Slot> {
        self.iter().map(|(x, _)| x).collect()
    }
    pub fn values_vec(&self) -> Vec<Slot> {
        self.iter().map(|(_, y)| y).collect()
    }

    pub fn inverse(&self) -> SlotMap {
        if CHECKS {
            assert!(self.is_bijection());
        }

        let mut out = Self::new();
        for (x, y) in self.iter() {
            out.insert(y, x);
        }

        out
    }

    pub fn is_bijection(&self) -> bool {
        let mut found = HashSet::default();

        for (_, x) in self.iter() {
            if found.contains(&x) {
                return false;
            }

            found.insert(x);
        }

        true
    }

    pub fn is_perm(&self) -> bool {
        self.is_bijection() && self.keys() == self.values()
    }

    #[track_caller]
    pub fn compose(&self, other: &SlotMap) -> SlotMap {
        if CHECKS {
            assert_eq!(self.values(), other.keys(), "SlotMap::compose() failed!");
        }

        self.compose_partial(other)
    }

    // self :: X -> Y
    // other :: Y -> Z
    // -> out :: X -> Z
    //
    // In other words, compose_partial first runs "self" and then "other", for any given input Slot.
    pub fn compose_partial(&self, other: &SlotMap) -> SlotMap {
        let mut out = SlotMap::new();
        for (x, y) in self.iter() {
            if let Some(z) = other.get(y) {
                out.insert(x, z);
            }
        }
        out
    }

    // if some slot is missing in `other`, we just choose a fresh slot as output.
    // self.keys() == self.compose_fresh(other).keys() is guaranteed.
    pub fn compose_fresh(&self, other: &SlotMap) -> SlotMap {
        let mut out = SlotMap::new();
        for (x, y) in self.iter() {
            if let Some(z) = other.get(y) {
                out.insert(x, z);
            } else {
                out.insert(x, Slot::fresh());
            }
        }
        out
    }

    pub fn identity(set: &SmallHashSet<Slot>) -> SlotMap {
        let mut out = SlotMap::new();
        for &x in set {
            out.insert(x, x);
        }
        out
    }

    pub fn bijection_from_fresh_to(set: &SmallHashSet<Slot>) -> SlotMap {
        let mut out = SlotMap::new();
        for &x in set {
            out.insert(Slot::fresh(), x);
        }
        out
    }

    pub fn remove(&mut self, x: Slot) {
        if let Ok(i) = self.search(x) {
            self.map.remove(i);
        }
    }

    pub fn from_pairs(pairs: &[(Slot, Slot)]) -> SlotMap {
        pairs.iter().copied().collect()
    }

    // will panic, if the maps are incompatible.
    #[track_caller]
    pub fn union(&self, other: &SlotMap) -> Self {
        let mut out = self.clone();

        for (x, y) in other.iter() {
            if CHECKS {
                if let Some(z) = out.get(x) {
                    assert_eq!(
                        y, z,
                        "SlotMap::union: The SlotMaps disagree! {:?} -> {:?} / {:?}",
                        x, z, y
                    );
                }
            }
            out.insert(x, y);
        }

        out
    }

    pub fn try_union(&self, other: &SlotMap) -> Option<Self> {
        let mut out = self.clone();

        for (x, y) in other.iter() {
            if let Some(z) = out.get(x) {
                if y != z {
                    return None;
                }
            }
            out.insert(x, y);
        }

        Some(out)
    }

    // checks invariants.
    #[allow(unused)]
    fn check(&self) {
        // sortedness.
        let mut sorted = self.map.clone();
        sorted.sort_by_key(|(l, _)| *l);
        assert_eq!(&self.map, &sorted);

        // left-uniqueness.
        let mut found = HashSet::default();
        for &(x, _) in self.map.iter() {
            assert!(!found.contains(&x));
            found.insert(x);
        }
    }
}

impl Index<Slot> for SlotMap {
    type Output = Slot;

    #[track_caller]
    #[inline]
    fn index(&self, l: Slot) -> &Slot {
        let Ok(i) = self.search(l) else {
            panic!("SlotMap::index({:?}): index missing!", l);
        };

        &self.map[i].1
    }
}

impl FromIterator<(Slot, Slot)> for SlotMap {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Slot, Slot)>,
    {
        let mut m = SlotMap::new();
        for (x, y) in iter.into_iter() {
            if CHECKS {
                assert!(!m.contains_key(x));
            }
            m.insert(x, y);
        }
        m
    }
}

impl<const N: usize> From<[(Slot, Slot); N]> for SlotMap {
    fn from(pairs: [(Slot, Slot); N]) -> Self {
        let mut m = SlotMap::new();
        for (x, y) in pairs {
            if CHECKS {
                assert!(!m.contains_key(x));
            }

            m.insert(x, y);
        }
        m
    }
}

#[test]
fn test_slotmap() {
    let mut m: SlotMap = SlotMap::new();
    m.insert(Slot::numeric(3), Slot::numeric(7));
    m.insert(Slot::numeric(2), Slot::numeric(7));
    m.insert(Slot::numeric(3), Slot::numeric(8));
    m.insert(Slot::numeric(4), Slot::numeric(7));
    assert_eq!(m[Slot::numeric(3)], Slot::numeric(8));

    m.check();
}
