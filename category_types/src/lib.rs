use bitflags::bitflags;

pub type CategoryId = u32;
pub type SurfaceId = u32;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CategoryMask: u32 {
        const NONE = 0;
        const ALL = u32::MAX;
    }
}

impl CategoryMask {
    pub fn from_id(id: CategoryId) -> Option<Self> {
        if id >= 32 {
            return None;
        }
        Some(CategoryMask::from_bits_truncate(1u32 << id))
    }

    pub fn contains_id(self, id: CategoryId) -> bool {
        if let Some(mask) = Self::from_id(id) {
            self.contains(mask)
        } else {
            false
        }
    }

    pub fn insert_id(&mut self, id: CategoryId) -> bool {
        if let Some(mask) = Self::from_id(id) {
            self.insert(mask);
            true
        } else {
            false
        }
    }
}
