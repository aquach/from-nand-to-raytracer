use crate::Vec3;
use crate::Number;

#[derive(Debug, Clone, Copy)]
pub struct Directional {
    pub direction: Vec3,
    pub color: Number,
}
