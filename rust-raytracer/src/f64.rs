use std::fmt;
use crate::int32::Int32;

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub struct Number(f64);

#[allow(dead_code)]
pub const PI: Number = Number(std::f64::consts::PI);

#[allow(dead_code)]
impl Number {
    pub fn from(i: i16) -> Number {
        Number(f64::from(i))
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0.0
    }

    pub fn is_negative(&self) -> bool {
        self.0 < 0.0
    }

    pub fn is_positive(&self) -> bool {
        self.0 > 0.0
    }

    pub fn do_add(&mut self, other: &Number) {
        self.0 = self.0 + other.0;
    }

    pub fn do_sub(&mut self, other: &Number) {
        self.0 = self.0 - other.0;
    }

    pub fn do_mul(&mut self, other: &Number) {
        self.0 = self.0 * other.0;
    }

    pub fn do_div(&mut self, other: &Number) {
        self.0 = self.0 / other.0;
    }

    pub fn do_sqrt(&mut self) {
        self.0 = f64::sqrt(self.0);
    }

    pub fn do_neg(&mut self) {
        self.0 = -self.0;
    }

    pub fn do_abs(&mut self) {
        self.0 = f64::abs(self.0);
    }

    pub fn to_f64(&self) -> f64 {
        self.0
    }
    pub fn to_int32(mut self) -> Int32 {
        Int32::from_i32(self.0 as i32)
    }

    pub fn cmp(&self, other: &Number) -> i16 {
        let d = self.0 - other.0;
        if d < 0.0 {
            -1
        } else if d > 0.0 {
            1
        } else {
            0
        }
    }

    pub fn is_less_than(&self, other: &Number) -> bool {
        self.0 < other.0
    }
}

impl fmt::Debug for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Number({})", self.to_f64())
    }
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_f64())
    }
}
