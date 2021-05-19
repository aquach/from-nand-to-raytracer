use crate::Number;
use crate::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct Directional {
    pub direction: Vec3,
    pub color: Number,
}
