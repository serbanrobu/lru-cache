#![feature(map_many_mut)]

use std::{borrow::Borrow, collections::HashMap, hash::Hash};

pub struct LruCache<K, V> {
    map: HashMap<K, (Option<K>, V, Option<K>)>,
    first_k: Option<K>,
    last_k: Option<K>,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + Hash,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::with_capacity(capacity),
            first_k: None,
            last_k: None,
        }
    }

    pub fn get<Q: ?Sized>(&mut self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Clone,
        Q: Hash + Eq,
    {
        let (k1, _k2, v) = self.remove(k)?;
        self.insert(k1, v);
        let (_, v, _) = self.map.get(k)?;
        Some(v)
    }

    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<(K, K, V)>
    where
        K: Borrow<Q> + Clone,
        Q: Hash + Eq,
    {
        let (prev_k, v, next_k) = self.map.remove(k)?;

        let (k1, k2) = match (prev_k, next_k) {
            (Some(prev_k), Some(next_k)) => {
                let [(_, _, prev_next_k), (next_prev_k, _, _)] = self
                    .map
                    .get_many_mut([prev_k.borrow(), next_k.borrow()])
                    .unwrap();

                (
                    next_prev_k.replace(prev_k).unwrap(),
                    prev_next_k.replace(next_k).unwrap(),
                )
            }
            (Some(prev_k), None) => {
                let (_, _, prev_next_k) = self.map.get_mut(prev_k.borrow()).unwrap();

                (
                    prev_next_k.take().unwrap(),
                    self.last_k.replace(prev_k).unwrap(),
                )
            }
            (None, Some(next_k)) => {
                let (next_prev_k, _, _) = self.map.get_mut(next_k.borrow()).unwrap();

                (
                    self.first_k.replace(next_k).unwrap(),
                    next_prev_k.take().unwrap(),
                )
            }
            (None, None) => (self.first_k.take().unwrap(), self.last_k.take().unwrap()),
        };

        Some((k1, k2, v))
    }

    pub fn insert(&mut self, k: K, v: V)
    where
        K: Clone,
    {
        let last_k = match self.last_k.replace(k.clone()) {
            Some(last_k) => {
                let (_, _, last_next_k) = self.map.get_mut(&last_k).unwrap();
                *last_next_k = Some(k.clone());
                Some(last_k)
            }
            None => {
                self.first_k = Some(k.clone());
                None
            }
        };

        self.map.insert(k, (last_k, v, None));
    }
}
