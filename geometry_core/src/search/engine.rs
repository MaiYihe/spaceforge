use std::collections::HashSet;
use std::marker::PhantomData;

use crate::search::frontier::{Frontier};
use crate::search::node::SearchNode;
// 搜索引擎
pub struct SearchEngine<T, F>
where 
    T: SearchNode,
    F: Frontier<T>,
{
    frontier: F,
    visited: HashSet<u64>,
    step_count: usize,
    _marker: PhantomData<T>,
}

impl<T, F> SearchEngine<T, F>
where
    T: SearchNode,
    F: Frontier<T>,
{
    pub fn with_root(mut frontier: F, root: T) -> Self {
        frontier.push(root);

        Self {
            frontier,
            visited: HashSet::new(),
            step_count: 0,
            _marker: PhantomData,
        }
    }

    pub fn step(&mut self) {
        let Some(node) = self.frontier.pop() else {
            return;
        };

        let key = node.key();
        if self.visited.contains(&key) {
            return;
        }

        self.visited.insert(key);

        let children = node.expand();

        for child in children {
            let k = child.key();
            if !self.visited.contains(&k) {
                self.frontier.push(child);
            }
        }

        self.step_count += 1;
    }

    pub fn frontier_len(&self) -> usize {
        self.frontier.len()
    }

    pub fn steps(&self) -> usize {
        self.step_count
    }
}
