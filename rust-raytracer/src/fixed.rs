use crate::int32::Int32;
use std::convert::TryFrom;
use std::fmt;

lazy_static! {
    static ref BYTES_FOR_FRAC: usize = 2;
    static ref SCALE_FACTOR: Int32 = {
        let mut x = Int32::from(256);
        let y = x;
        x.do_mul(&y);
        x
    };
    pub static ref PI: Q16_16 = {
        let mut x = Int32::from(561);
        let y = Int32::from(367);
        x.do_mul(&y);
        Q16_16(x)
    };
}

#[derive(Clone, Copy)]
pub struct Q16_16(Int32);

impl Q16_16 {
    pub fn from(i: i16) -> Q16_16 {
        assert!(SCALE_FACTOR.to_i32() == i32::pow(256, u32::try_from(*BYTES_FOR_FRAC).unwrap()));

        let mut r = Int32::from(i);
        r.do_left_shift_bytes(*BYTES_FOR_FRAC);
        Q16_16(r)
    }

    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    pub fn is_negative(&self) -> bool {
        self.0.is_negative()
    }

    pub fn is_positive(&self) -> bool {
        self.0.is_positive()
    }

    pub fn do_add(&mut self, other: &Q16_16) {
        self.0.do_add(&other.0);
    }

    pub fn do_sub(&mut self, other: &Q16_16) {
        self.0.do_sub(&other.0);
    }

    pub fn do_mul(&mut self, other: &Q16_16) {
        self.0.do_mul_right_shift_bytes(&other.0, *BYTES_FOR_FRAC);
    }

    pub fn do_div(&mut self, other: &Q16_16) {
        self.0.do_left_shift_bytes_div(*BYTES_FOR_FRAC, &other.0);
    }

    pub fn do_sqrt(&mut self) {
        self.0.do_sqrt();
        let mut s = SCALE_FACTOR.clone();
        s.do_sqrt();
        self.0.do_mul(&s);
    }

    pub fn do_neg(&mut self) {
        self.0.do_neg();
    }

    pub fn do_abs(&mut self) {
        self.0.do_abs();
    }

    pub fn to_f64(&self) -> f64 {
        f64::from(self.0.to_i32()) / f64::from(SCALE_FACTOR.to_i32())
    }

    pub fn do_sin(&mut self) {}

    pub fn do_cos(&mut self) {}

    pub fn do_tan(&mut self) {
        // TODO CHEATING
        let result = f64::tan(self.to_f64());
        self.0 = Int32::from((result * f64::from(1 << (8 * *BYTES_FOR_FRAC - 1))).trunc() as i16);
        self.0.do_mul(&Int32::from(2));
    }

    pub fn cmp(&self, other: &Q16_16) -> i16 {
        self.0.cmp(&other.0)
    }

    pub fn is_less_than(&self, other: &Q16_16) -> bool {
        self.0.cmp(&other.0) < 0
    }
}

impl fmt::Debug for Q16_16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Q16_16({})", self.to_f64())
    }
}

impl fmt::Display for Q16_16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_f64(),)
    }
}

#[cfg(test)]
mod test {
    use super::Q16_16;

    fn test_one_mul(x: i16, y: i16) {
        let xi = Q16_16::from(x);
        let yi = Q16_16::from(y);

        let mut result = xi;
        result.do_mul(&yi);

        let actual = result.to_f64();
        let expected = f64::from(x) * f64::from(y);
        assert_eq!(
            actual, expected,
            "{} x {} = {} but got {}",
            x, y, expected, actual
        );
    }

    fn test_one_add(x: i16, y: i16) {
        let xi = Q16_16::from(x);
        let yi = Q16_16::from(y);

        let mut result = xi;
        result.do_add(&yi);

        let actual = result.to_f64();
        let expected = f64::from(x) + f64::from(y);
        assert_eq!(
            actual, expected,
            "{} + {} = {} but got {}",
            x, y, expected, actual
        );
    }

    fn test_one_sub(x: i16, y: i16) {
        let xi = Q16_16::from(x);
        let yi = Q16_16::from(y);

        let mut result = xi;
        result.do_sub(&yi);

        println!("{} - {} = {}", xi, yi, result);

        assert_eq!(result.to_f64(), f64::from(x) - f64::from(y));
    }

    fn test_one_div(x: i16, y: i16) {
        let xi = Q16_16::from(x);
        let yi = Q16_16::from(y);

        let mut result = xi;
        result.do_div(&yi);

        let actual = result.to_f64();
        let expected = f64::from(x) / f64::from(y);

        assert!(
            f64::abs(result.to_f64() - f64::from(x) / f64::from(y)) <= 0.01,
            "{} / {} = {} but got {}",
            x,
            y,
            expected,
            actual
        );
    }

    fn test_one_sqrt(x: Q16_16) {
        let mut result = x;
        result.do_sqrt();
        let actual = result.to_f64();
        let expected = f64::sqrt(x.to_f64());

        assert!(
            f64::abs(actual - expected) / (expected + 0.001) <= 0.01,
            "sqrt({}) = {} but got {}",
            x,
            expected,
            actual
        );
    }

    #[test]
    fn test_part() {
        let mut x = Q16_16::from(3);
        let y = Q16_16::from(2);
        x.do_div(&y);
    }

    #[test]
    fn test_add() {
        test_one_add(4, 3);
        test_one_add(100, 3);
        test_one_add(100, 350);
        test_one_add(10, 5000);
        test_one_add(5000, 6000);
        test_one_add(13082, 10082);
        test_one_add(16000, 16000);
        test_one_add(2, -2);
        test_one_add(4, -2);
        test_one_add(-4, -3);
        test_one_add(-4, 1000);
        test_one_add(1508, -1600);
        test_one_add(0, -1);
        test_one_add(-1, 0);
        test_one_add(0, 0);
        test_one_add(1, 0);
        test_one_add(0, 1);
    }

    #[test]
    fn test_sub() {
        test_one_sub(4, 3);
        test_one_sub(100, 3);
        test_one_sub(100, 350);
        test_one_sub(10, 5000);
        test_one_sub(5000, 6000);
        test_one_sub(13082, 23082);
        test_one_sub(32600, 32600);
        test_one_sub(2, -2);
        test_one_sub(4, -2);
        test_one_sub(-4, -3);
        test_one_sub(-4, 1000);
        test_one_sub(1508, -1600);
        test_one_sub(0, -1);
        test_one_sub(-1, 0);
        test_one_sub(0, 0);
        test_one_sub(1, 0);
        test_one_sub(0, 1);
    }

    #[test]
    fn test_mul() {
        test_one_mul(4, 3);
        test_one_mul(100, 3);
        test_one_mul(40, 38);
        test_one_mul(10, 3030);
        test_one_mul(128, 255);
        test_one_mul(2, -2);
        test_one_mul(4, -2);
        test_one_mul(-4, -3);
        test_one_mul(5082, 0);
        test_one_mul(0, 5082);
    }

    #[test]
    fn test_div() {
        test_one_div(4, 3);
        test_one_div(100, 3);
        test_one_div(100, 350);
        test_one_div(10, 5000);
        test_one_div(10098, 594);
        test_one_div(10099, 594);
        test_one_div(10097, 594);
        test_one_div(5000, 5000);
        test_one_div(32600, 32600);
        test_one_div(2, -2);
        test_one_div(4, -2);
        test_one_div(-4, -3);
        test_one_div(0, 5082);
    }

    #[test]
    fn test_is_negative() {
        assert_eq!(Q16_16::from(0).is_negative(), false);

        assert_eq!(Q16_16::from(-1).is_negative(), true);
        assert_eq!(Q16_16::from(-2).is_negative(), true);
        assert_eq!(Q16_16::from(-30000).is_negative(), true);

        assert_eq!(Q16_16::from(1).is_negative(), false);
        assert_eq!(Q16_16::from(2).is_negative(), false);
        assert_eq!(Q16_16::from(30000).is_negative(), false);
    }

    #[test]
    fn test_neg() {
        let mut x = Q16_16::from(0);
        x.do_neg();
        assert_eq!(x.to_f64(), 0f64);

        let mut x = Q16_16::from(1);
        x.do_neg();
        assert_eq!(x.to_f64(), -1f64);

        let mut x = Q16_16::from(-5);
        x.do_neg();
        assert_eq!(x.to_f64(), 5f64);

        let mut x = Q16_16::from(30000);
        x.do_neg();
        assert_eq!(x.to_f64(), -30000f64);

        let mut x = Q16_16::from(-30000);
        x.do_neg();
        assert_eq!(x.to_f64(), 30000f64);

        let mut x = Q16_16::from(256);
        x.do_neg();
        assert_eq!(x.to_f64(), -256f64);
    }

    #[test]
    fn test_abs() {
        let mut x = Q16_16::from(0);
        x.do_abs();
        assert_eq!(x.to_f64(), 0f64);

        let mut x = Q16_16::from(1);
        x.do_abs();
        assert_eq!(x.to_f64(), 1f64);

        let mut x = Q16_16::from(-5);
        x.do_abs();
        assert_eq!(x.to_f64(), 5f64);

        let mut x = Q16_16::from(30000);
        x.do_abs();
        assert_eq!(x.to_f64(), 30000f64);

        let mut x = Q16_16::from(-30000);
        x.do_abs();
        assert_eq!(x.to_f64(), 30000f64);
    }

    #[test]
    fn test_sqrt() {
        test_one_sqrt(Q16_16::from(0));
        test_one_sqrt(Q16_16::from(1));
        test_one_sqrt(Q16_16::from(9));
        test_one_sqrt(Q16_16::from(15));
        test_one_sqrt(Q16_16::from(30000));
        test_one_sqrt(Q16_16::from(25000));
    }
}
