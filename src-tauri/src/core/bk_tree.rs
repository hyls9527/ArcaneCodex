#![allow(missing_docs)]
use std::collections::HashMap;

pub struct BkTree<T> {
    root: Option<BkNode<T>>,
}

impl<T> Default for BkTree<T> {
    fn default() -> Self {
        Self { root: None }
    }
}

struct BkNode<T> {
    item: T,
    indices: Vec<usize>,
    children: HashMap<u32, BkNode<T>>,
}

impl<T> BkTree<T> {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn insert<F>(&mut self, item: T, index: usize, distance_fn: F)
    where
        F: Fn(&T, &T) -> u32,
    {
        let new_node = BkNode {
            item,
            indices: vec![index],
            children: HashMap::new(),
        };

        match self.root.take() {
            None => self.root = Some(new_node),
            Some(mut root) => {
                insert_recursive(&mut root, new_node, &distance_fn);
                self.root = Some(root);
            }
        }
    }

    pub fn search<F>(&self, query: &T, threshold: u32, distance_fn: F) -> Vec<(u32, usize)>
    where
        F: Fn(&T, &T) -> u32,
    {
        let mut results = Vec::new();

        if let Some(ref root) = self.root {
            search_recursive(root, query, threshold, &distance_fn, &mut results);
        }

        results
    }
}

fn insert_recursive<T, F>(node: &mut BkNode<T>, new_node: BkNode<T>, distance_fn: &F)
where
    F: Fn(&T, &T) -> u32,
{
    let dist = distance_fn(&new_node.item, &node.item);

    if dist == 0 {
        node.indices.extend(new_node.indices);
        return;
    }

    match node.children.get_mut(&dist) {
        Some(child) => insert_recursive(child, new_node, distance_fn),
        None => {
            node.children.insert(dist, new_node);
        }
    }
}

fn search_recursive<T, F>(
    node: &BkNode<T>,
    query: &T,
    threshold: u32,
    distance_fn: &F,
    results: &mut Vec<(u32, usize)>,
) where
    F: Fn(&T, &T) -> u32,
{
    let dist = distance_fn(query, &node.item);

    if dist <= threshold {
        for &idx in &node.indices {
            results.push((dist, idx));
        }
    }

    let low = dist.saturating_sub(threshold);
    let high = dist + threshold;

    for (child_dist, child) in &node.children {
        if *child_dist >= low && *child_dist <= high {
            search_recursive(child, query, threshold, distance_fn, results);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hamming(a: &u64, b: &u64) -> u32 {
        (a ^ b).count_ones()
    }

    #[test]
    fn test_insert_and_search_empty() {
        let tree: BkTree<u64> = BkTree::new();
        let results = tree.search(&0, 10, hamming);
        assert!(results.is_empty());
    }

    #[test]
    fn test_insert_single_and_find() {
        let mut tree = BkTree::new();
        tree.insert(0u64, 0, hamming);
        let results = tree.search(&0, 0, hamming);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, 0);
        assert_eq!(results[0].1, 0);
    }

    #[test]
    fn test_search_within_threshold() {
        let mut tree = BkTree::new();
        tree.insert(0u64, 0, hamming);
        tree.insert(1u64, 1, hamming);
        tree.insert(0xFFFF_FFFF_FFFF_FFFF, 2, hamming);

        let results = tree.search(&0, 2, hamming);
        let found_indices: Vec<usize> = results.iter().map(|r| r.1).collect();
        assert!(found_indices.contains(&0));
        assert!(found_indices.contains(&1));
        assert!(!found_indices.contains(&2));
    }

    #[test]
    fn test_search_no_results() {
        let mut tree = BkTree::new();
        tree.insert(0u64, 0, hamming);
        tree.insert(1u64, 1, hamming);

        let results = tree.search(&0xFFFF_FFFF_FFFF_FFFF, 2, hamming);
        assert!(results.is_empty());
    }

    #[test]
    fn test_duplicate_hash_stores_multiple_indices() {
        let mut tree = BkTree::new();
        tree.insert(42u64, 0, hamming);
        tree.insert(42u64, 1, hamming);
        tree.insert(42u64, 2, hamming);

        let results = tree.search(&42, 0, hamming);
        assert_eq!(results.len(), 3);
        let indices: Vec<usize> = results.iter().map(|r| r.1).collect();
        assert!(indices.contains(&0));
        assert!(indices.contains(&1));
        assert!(indices.contains(&2));
    }

    #[test]
    fn test_large_threshold() {
        let mut tree = BkTree::new();
        tree.insert(0u64, 0, hamming);
        tree.insert(0xFF, 1, hamming);
        tree.insert(0xFF00, 2, hamming);
        tree.insert(0xFFFF_FFFF_FFFF_FFFF, 3, hamming);

        let results = tree.search(&0, 64, hamming);
        assert_eq!(results.len(), 4);
    }
}
