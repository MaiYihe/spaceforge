use bitflags::bitflags;

pub type RegionsType = u32;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct RegionsTypeMask: u32 {
        const NONE = 0;
        const ALL = u32::MAX;
    }
}

impl RegionsTypeMask {
    pub fn from_id(id: RegionsType) -> Option<Self> {
        if id >= 32 {
            return None;
        }
        Some(RegionsTypeMask::from_bits_truncate(1u32 << id))
    }

    pub fn contains_id(self, id: RegionsType) -> bool {
        if let Some(mask) = Self::from_id(id) {
            self.contains(mask)
        } else {
            false
        }
    }

    pub fn insert_id(&mut self, id: RegionsType) -> bool {
        if let Some(mask) = Self::from_id(id) {
            self.insert(mask);
            true
        } else {
            false
        }
    }
}
