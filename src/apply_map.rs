use crate::*;

pub struct ApplyMap<K, V>(HashMap<K, Applied<V>>);

impl<K, V> ApplyMap<K, V> {
    pub fn new() -> Self {
        ApplyMap(HashMap::default())
    }

    pub fn insert(&mut self, Applied(mk, k): Applied<K>, Applied(mv, v): Applied<V>) where K: Hash + Eq {
        self.0.insert(k, Applied(&mk.inverse() * mv, v));
    }

    pub fn get(&self, Applied(m, k): Applied<K>) -> Option<Applied<V>> where K: Hash + Eq, V: Clone {
        self.0.get(&k).map(|v| &m * v.clone())
    }
}
