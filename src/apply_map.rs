use crate::*;

pub struct ApplyMap<K, V>(HashMap<K, Applied<V>>);

impl<K, V> ApplyMap<K, V> {
    pub fn new() -> Self {
        ApplyMap(HashMap::default())
    }

    pub fn insert(&mut self, AppliedBy(mk, k): AppliedBy<SlotMapRef<impl SlotMapLike>, K>, AppliedBy(mv, v): AppliedBy<impl SlotMapLike, V>) -> Option<Applied<V>> where K: Hash + Eq, V: Clone {
        match self.0.entry(k) {
            Entry::Vacant(e) => {
                let v = AppliedBy(mk.inverse().rf() * mv.into(), v);
                e.insert(v);
                None
            }
            Entry::Occupied(e) => {
                Some(mk * e.get().clone())
            },
        }
    }

    pub fn insert_raw(&mut self, k: K, a_v: Applied<V>) -> Option<Applied<V>> where K: Hash + Eq, V: Clone {
        match self.0.entry(k) {
            Entry::Vacant(e) => {
                e.insert(a_v);
                None
            }
            Entry::Occupied(e) => {
                Some(e.get().clone())
            },
        }
    }

    pub fn get(&self, AppliedBy(m, k): Applied<K>) -> Option<Applied<V>> where K: Hash + Eq, V: Clone {
        self.0.get(&k).map(|v| m.rf() * v.clone())
    }

    pub fn remove(&mut self, AppliedBy(mk, k): Applied<K>) where K: Hash + Eq {
        self.0.remove(&k);
    }
}
