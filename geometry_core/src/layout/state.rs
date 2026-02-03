use crate::layout::{GeometryCache, Placement};
use crate::search::SearchNode;
use std::collections::{BinaryHeap, HashMap};
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;

pub struct LayoutState {
    placed: Vec<Placement>, // 已放家具
    occupancy: GeometryCache,
    inventory: HashMap<u32, u32>, // 用 HashMap 记录家具类型和数量
    score: f32,
}

impl SearchNode for LayoutState {
    fn expand(&self) -> BinaryHeap<Self> {
        let mut children = BinaryHeap::new();

        // 假设最大扩展128个子节点
        for _ in 0..128 {
            let new_node = LayoutState {
                placed: self.placed.clone(),       // 复制已放置的家具
                occupancy: self.occupancy.clone(), // 复制几何缓存
                inventory: self.inventory.clone(), // 复制剩余家具
                score: self.score + 1.0,           // 假设新节点的分数加 1
            };
            children.push(new_node);
        }

        children
    }

    fn score(&self) -> f32 {
        self.score
    }

    fn key(&self) -> u64 {
        self.hash_layout()
    }
}

// 计算布局状态的唯一key
impl LayoutState {
    // 计算布局状态的唯一key
    fn hash_layout(&self) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();

        // 对 `placed`, `inventory` 进行哈希运算
        self.placed.hash(&mut hasher);
        for (key, value) in &self.inventory {
            key.hash(&mut hasher); // 对键进行哈希
            value.hash(&mut hasher); // 对值进行哈希
        }

        // 返回哈希值
        hasher.finish()
    }
}

// 确保 LayoutState 在最大堆中可以存放
impl PartialEq for LayoutState {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for LayoutState {}

impl PartialOrd for LayoutState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LayoutState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}
