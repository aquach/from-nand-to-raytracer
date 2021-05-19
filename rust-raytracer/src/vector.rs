use crate::Number;
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: Number,
    pub y: Number,
    pub z: Number,
}

impl Vec3 {
    pub fn do_normalize(&mut self) {
        let mut x = self.x;
        x.do_mul(&self.x);

        let mut y = self.y;
        y.do_mul(&self.y);

        let mut z = self.z;
        z.do_mul(&self.z);

        x.do_add(&y);
        x.do_add(&z);

        x.do_sqrt();

        self.x.do_div(&x);
        self.y.do_div(&x);
        self.z.do_div(&x);
    }

    pub fn do_add(&mut self, other: &Vec3) {
        self.x.do_add(&other.x);
        self.y.do_add(&other.y);
        self.z.do_add(&other.z);
    }

    pub fn do_scale(&mut self, s: &Number) {
        self.x.do_mul(&s);
        self.y.do_mul(&s);
        self.z.do_mul(&s);
    }

    pub fn do_sub(&mut self, other: &Vec3) {
        self.x.do_sub(&other.x);
        self.y.do_sub(&other.y);
        self.z.do_sub(&other.z);
    }

    pub fn dist_sq(&self) -> Number {
        self.dot(self)
    }

    pub fn dot(&self, other: &Vec3) -> Number {
        let mut xx = self.x;
        xx.do_mul(&other.x);

        let mut yy = self.y;
        yy.do_mul(&other.y);

        let mut zz = self.z;
        zz.do_mul(&other.z);

        xx.do_add(&yy);
        xx.do_add(&zz);

        xx
    }
}

impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vec3({}, {}, {})", self.x, self.y, self.z)
    }
}
