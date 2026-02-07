use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FurnitureMask: u32 {
        const SOFA = 0b0001;
        const TABLE = 0b0010;
        const LAMP = 0b0100;
        const ALL = u32::MAX;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FurnitureType {
    Sofa,
    Table,
    Lamp,
}
