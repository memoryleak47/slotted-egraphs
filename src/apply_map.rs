use crate::*;

pub struct ApplyMap<K, V>(HashMap<K, Applied<V>>);

impl<K, V> ApplyMap<K, V> {
    pub fn new() -> Self {
        ApplyMap(HashMap::default())
    }

    pub fn insert(&mut self, Applied(mk, k): Applied<K>, Applied(mv, v): Applied<V>) -> Option<Applied<V>> where K: Hash + Eq, V: Clone {
        let v = Applied(&mk.inverse() * mv, v);
        match self.0.entry(k) {
            Entry::Vacant(e) => {
                e.insert(v);
                None
            }
            Entry::Occupied(e) => {
                Some(&mk * e.get().clone())
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

    pub fn get(&self, Applied(m, k): Applied<K>) -> Option<Applied<V>> where K: Hash + Eq, V: Clone {
        self.0.get(&k).map(|v| &m * v.clone())
    }

    pub fn remove(&mut self, Applied(mk, k): Applied<K>) where K: Hash + Eq {
        self.0.remove(&k);
    }
}
