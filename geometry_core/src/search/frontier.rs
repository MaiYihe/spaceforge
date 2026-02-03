use std::collections::BinaryHeap;
use crate::search::node::{ScoredNode, SearchNode};

pub trait Frontier<T: SearchNode> {
    fn push(&mut self, node: T);
    fn pop(&mut self) -> Option<T>;
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct HeapFrontier<T: SearchNode> {
    heap: BinaryHeap<ScoredNode<T>>,
}

impl<T: SearchNode> HeapFrontier<T> {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }
}

// Default ≈ 无参构造函数
impl<T: SearchNode> Default for HeapFrontier<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: SearchNode> Frontier<T> for HeapFrontier<T> {
    fn push(&mut self, node: T) {
        self.heap.push(ScoredNode(node));
    }

    fn pop(&mut self) -> Option<T> {
        self.heap.pop().map(|n| n.0)
    }

    fn len(&self) -> usize {
        self.heap.len()
    }
}
