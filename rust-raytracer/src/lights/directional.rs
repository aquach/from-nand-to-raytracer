use crate::Vec3;

type Fixed = crate::fixed::Q16_16;

#[derive(Debug, Clone, Copy)]
pub struct Directional {
    pub direction: Vec3,
    pub color: Fixed,
}
