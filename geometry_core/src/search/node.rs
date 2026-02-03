use std::cmp::Ordering;
use std::collections::BinaryHeap;

/// 任何参与搜索的状态都必须实现这个 trait
pub trait SearchNode: Sized {
    /// 扩展当前节点 → 生成子节点
    fn expand(&self) -> BinaryHeap<Self>;

    /// 节点评分（越大越优先）
    fn score(&self) -> f32;

    /// 可选：唯一标识（用于 visited 去重）
    fn key(&self) -> u64;
}

/// 包装节点，让 Frontier 的 BinaryHeap 可以排序
#[derive(Debug)]
pub struct ScoredNode<T: SearchNode>(pub T);

impl<T: SearchNode> PartialEq for ScoredNode<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0.score() == other.0.score()
    }
}

impl<T: SearchNode> Eq for ScoredNode<T> {}

impl<T: SearchNode> PartialOrd for ScoredNode<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: SearchNode> Ord for ScoredNode<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
