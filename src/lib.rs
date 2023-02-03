#![feature(map_many_mut)]

use std::{borrow::Borrow, collections::HashMap, hash::Hash};

type NodeId = usize;

#[derive(Debug)]
struct Node<T> {
    next_id: Option<NodeId>,
    prev_id: Option<NodeId>,
    element: T,
}

#[derive(Debug)]
struct Graph<T> {
    nodes: HashMap<NodeId, Node<T>>,
    head_id: Option<NodeId>,
    tail_id: Option<NodeId>,
}

impl<T> Graph<T> {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: HashMap::with_capacity(capacity),
            head_id: None,
            tail_id: None,
        }
    }

    fn remove(&mut self, node_id: NodeId) -> Option<T> {
        let node = self.nodes.remove(&node_id)?;

        match (node.prev_id, node.next_id) {
            (Some(prev_id), Some(next_id)) => {
                let [prev, next] = self.nodes.get_many_mut([&prev_id, &next_id]).unwrap();

                next.prev_id = Some(prev_id);
                prev.next_id = Some(next_id);
            }
            (Some(prev_id), None) => {
                let prev = self.nodes.get_mut(&prev_id).unwrap();

                prev.next_id = None;
                self.tail_id = Some(prev_id);
            }
            (None, Some(next_id)) => {
                let next = self.nodes.get_mut(&next_id).unwrap();

                self.head_id = Some(next_id);
                next.prev_id = None;
            }
            (None, None) => {
                self.head_id = None;
                self.tail_id = None;
            }
        }

        Some(node.element)
    }

    fn pop_front(&mut self) -> Option<T> {
        let head_id = self.head_id?;
        self.remove(head_id)
    }

    fn push_back(&mut self, node_id: NodeId, element: T) {
        let tail_id = match self.tail_id.replace(node_id) {
            Some(tail_id) => {
                let tail = self.nodes.get_mut(&tail_id).unwrap();
                tail.next_id = Some(node_id);

                Some(tail_id)
            }
            None => {
                self.head_id = Some(node_id);
                None
            }
        };

        self.nodes.insert(
            node_id,
            Node {
                next_id: None,
                prev_id: tail_id,
                element,
            },
        );
    }

    fn get(&mut self, node_id: NodeId) -> Option<&T> {
        let elem = self.remove(node_id)?;
        self.push_back(node_id, elem);
        let node = self.nodes.get(&node_id).unwrap();
        Some(&node.element)
    }
}

#[derive(Debug)]
pub struct LruCache<K, V> {
    node_ids: HashMap<K, NodeId>,
    graph: Graph<(K, V)>,
    id: NodeId,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + Hash,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            node_ids: HashMap::with_capacity(capacity),
            graph: Graph::with_capacity(capacity),
            id: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.node_ids.len()
    }

    pub fn capacity(&self) -> usize {
        self.node_ids.capacity()
    }

    pub fn get<Q: ?Sized>(&mut self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let node_id = *self.node_ids.get(k)?;
        let (_k, v) = self.graph.get(node_id).unwrap();
        Some(&v)
    }

    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        let node_id = self.node_ids.remove(k)?;
        let (_k, v) = self.graph.remove(node_id).unwrap();
        Some(v)
    }

    pub fn insert(&mut self, k: K, v: V)
    where
        K: Clone,
    {
        if let Some(&node_id) = self.node_ids.get(&k) {
            self.graph.get(node_id).unwrap();
            return;
        }

        let node_id = self.id;
        self.id += 1;

        if self.len() == self.capacity() {
            let (k, _v) = self.graph.pop_front().unwrap();
            self.node_ids.remove(&k);
        }

        self.node_ids.insert(k.clone(), node_id);
        self.graph.push_back(node_id, (k, v));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache() {
        let mut cache = LruCache::new(3);

        cache.insert(&1, "aws");
        assert_eq!(cache.get(&1), Some(&"aws"));

        cache.insert(&2, "gcp");
        cache.insert(&3, "azure");
        assert_eq!(cache.get(&3), Some(&"azure"));

        cache.insert(&4, "vmware");
        assert_eq!(cache.get(&2), Some(&"gcp"));
        assert_eq!(cache.get(&1), None);

        cache.insert(&5, "val");
        assert_eq!(cache.get(&5), Some(&"val"));
        assert_eq!(cache.get(&4), Some(&"vmware"));
        assert_eq!(cache.get(&3), None);
        assert_eq!(cache.get(&2), Some(&"gcp"));
        assert_eq!(cache.get(&1), None);
    }
}
