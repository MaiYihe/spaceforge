use std::hash::{Hash, Hasher};

#[derive(Clone, Debug)]
pub struct Placement {
    pub item_id: u32,
    pub pose: Pose2D,
}

#[derive(Clone, Debug)]
pub struct Pose2D {
    pub x: f32,
    pub y: f32,
    pub theta: f32,
}

impl Hash for Pose2D {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // 对 Pose2D 中的字段进行哈希
        self.x.to_bits().hash(state); // f32 转换为 u32 后哈希
        self.y.to_bits().hash(state); // f32 转换为 u32 后哈希
        self.theta.to_bits().hash(state); // f32 转换为 u32 后哈希
    }
}

impl Hash for Placement {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // 对 item_id 和 pose 进行哈希
        self.item_id.hash(state);
        self.pose.hash(state); // Pose2D 会通过它自己的 `hash` 方法来处理
    }
}
