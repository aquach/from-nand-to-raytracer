use crate::int32::Int32;
use std::convert::TryFrom;
use std::fmt;

lazy_static! {
    static ref SLOTS_FOR_FRAC: usize = 2;
    static ref SCALE_FACTOR: Int32 = {
        let mut x = Int32::from(256);
        let y = x;
        x.do_mul(&y);
        x
    };
    static ref SCALE_FACTOR_SQRT_SQRT: Int32 = {
        let mut s = SCALE_FACTOR.clone();
        s.do_sqrt();
        s.do_sqrt();
        s
    };
    static ref SCALE_FACTOR_SQRT_SQRT_SQRT_POW_3: Int32 = {
        let mut s = SCALE_FACTOR.clone();
        s.do_sqrt();
        s.do_sqrt();
        s.do_sqrt();

        let t = s;

        s.do_mul(&t);
        s.do_mul(&t);

        s
    };
    pub static ref PI: Number = {
        let mut x = Int32::from(561);
        let y = Int32::from(367);
        x.do_mul(&y);
        Number(x)
    };
}

#[derive(Clone, Copy)]
pub struct Number(Int32);

impl Number {
    pub fn from(i: i16) -> Number {
        assert!(SCALE_FACTOR.to_i32() == i32::pow(256, u32::try_from(*SLOTS_FOR_FRAC).unwrap()));

        let mut r = Int32::from(i);
        r.do_left_shift_slots(*SLOTS_FOR_FRAC);
        Number(r)
    }

    pub fn to_int32(mut self) -> Int32 {
        self.0.do_right_shift_slots(*SLOTS_FOR_FRAC);
        self.0
    }

    pub fn from_i16_frac(i: i16) -> Number {
      let mut v = Int32::from(i);
      v.do_mul(&Int32::from(2));
      Number(v)
    }

    pub fn frac_to_i16(&self) -> i16 {
      let parts = self.0.parts;

      if self.is_negative() {
          return -((255 - parts[1]) * 128 + (256 - parts[0]) / 2);
      } else {
          return parts[1] * 128 + (parts[0] / 2);
      }
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

    pub fn do_add(&mut self, other: &Number) {
        self.0.do_add(&other.0);
    }

    pub fn do_sub(&mut self, other: &Number) {
        self.0.do_sub(&other.0);
    }

    pub fn do_mul(&mut self, other: &Number) {
        if self.is_zero() || other.is_zero() {
            self.0.do_zero();
            return;
        }

        self.0.do_mul_right_shift_slots(&other.0, *SLOTS_FOR_FRAC);
    }

    pub fn do_div(&mut self, other: &Number) {
        if self.is_zero() && !other.is_zero() {
            self.0.do_zero();
            return;
        }

        self.0.do_left_shift_slots_div(*SLOTS_FOR_FRAC, &other.0);
    }

    pub fn do_sqrt(&mut self) {
        self.0.do_mul(&SCALE_FACTOR_SQRT_SQRT);
        self.0.do_sqrt();
        self.0.do_mul(&SCALE_FACTOR_SQRT_SQRT_SQRT_POW_3);
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

    pub fn cmp(&self, other: &Number) -> i16 {
        self.0.cmp(&other.0)
    }

    pub fn is_less_than(&self, other: &Number) -> bool {
        self.0.cmp(&other.0) < 0
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

#[cfg(test)]
mod test {
    use super::Number;
    use rstest::*;

    fn test_one_mul(x: i16, y: i16) {
        let xi = Number::from(x);
        let yi = Number::from(y);

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
        let xi = Number::from(x);
        let yi = Number::from(y);

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
        let xi = Number::from(x);
        let yi = Number::from(y);

        let mut result = xi;
        result.do_sub(&yi);

        println!("{} - {} = {}", xi, yi, result);

        assert_eq!(result.to_f64(), f64::from(x) - f64::from(y));
    }

    #[test]
    fn test_part() {
        let mut x = Number::from(3);
        let y = Number::from(2);
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

    #[rstest]
    #[case(4, 3)]
    #[case(100, 3)]
    #[case(10000, 3)]
    #[case(10000, 1)]
    #[case(5000, 1000)]
    #[case(100, 350)]
    #[case(10, 5000)]
    #[case(10098, 594)]
    #[case(10099, 594)]
    #[case(10097, 594)]
    #[case(13, 2)]
    #[case(13, 6)]
    #[case(2382, 124)]
    #[case(5000, 5000)]
    #[case(32600, 32600)]
    #[case(2, -2)]
    #[case(4, -2)]
    #[case(-4, -3)]
    #[case(0, 5082)]
    fn test_div(#[case] x: i16, #[case] y: i16) {
        let xi = Number::from(x);
        let yi = Number::from(y);

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

    #[test]
    fn test_is_negative() {
        assert_eq!(Number::from(0).is_negative(), false);

        assert_eq!(Number::from(-1).is_negative(), true);
        assert_eq!(Number::from(-2).is_negative(), true);
        assert_eq!(Number::from(-30000).is_negative(), true);

        assert_eq!(Number::from(1).is_negative(), false);
        assert_eq!(Number::from(2).is_negative(), false);
        assert_eq!(Number::from(30000).is_negative(), false);
    }

    #[test]
    fn test_neg() {
        let mut x = Number::from(0);
        x.do_neg();
        assert_eq!(x.to_f64(), 0f64);

        let mut x = Number::from(1);
        x.do_neg();
        assert_eq!(x.to_f64(), -1f64);

        let mut x = Number::from(-5);
        x.do_neg();
        assert_eq!(x.to_f64(), 5f64);

        let mut x = Number::from(30000);
        x.do_neg();
        assert_eq!(x.to_f64(), -30000f64);

        let mut x = Number::from(-30000);
        x.do_neg();
        assert_eq!(x.to_f64(), 30000f64);

        let mut x = Number::from(256);
        x.do_neg();
        assert_eq!(x.to_f64(), -256f64);
    }

    #[test]
    fn test_abs() {
        let mut x = Number::from(0);
        x.do_abs();
        assert_eq!(x.to_f64(), 0f64);

        let mut x = Number::from(1);
        x.do_abs();
        assert_eq!(x.to_f64(), 1f64);

        let mut x = Number::from(-5);
        x.do_abs();
        assert_eq!(x.to_f64(), 5f64);

        let mut x = Number::from(30000);
        x.do_abs();
        assert_eq!(x.to_f64(), 30000f64);

        let mut x = Number::from(-30000);
        x.do_abs();
        assert_eq!(x.to_f64(), 30000f64);
    }

    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(9)]
    #[case(15)]
    #[case(144)]
    #[case(147)]
    #[case(256)]
    #[case(1024)]
    fn test_sqrt(#[case] x: i16) {
        let mut result = Number::from(x);
        result.do_sqrt();
        let actual = result.to_f64();
        let expected = f64::sqrt(f64::from(x));

        assert!(
            f64::abs(actual - expected) / (expected + 0.001) <= 0.01,
            "sqrt({}) = {} but got {}",
            x,
            expected,
            actual
        );
    }

    #[rstest]
    #[case(-20)]
    #[case(-220)]
    #[case(-420)]
    #[case(-640)]
    #[case(-730)]
    #[case(-990)]
    #[case(-1020)]
    #[case(-1220)]
    #[case(-1420)]
    #[case(-1640)]
    #[case(-1730)]
    #[case(-1990)]
    #[case(-1999)]
    #[case(20)]
    #[case(220)]
    #[case(420)]
    #[case(640)]
    #[case(730)]
    #[case(990)]
    #[case(1020)]
    #[case(1220)]
    #[case(1420)]
    #[case(1640)]
    #[case(1730)]
    #[case(1990)]
    #[case(1999)]
    fn test_frac_to_i16(#[case] x: i16) {
        let mut result = Number::from(x);
        result.do_div(&Number::from(1000));

        let actual = result.frac_to_i16();
        let x = f64::from(x) / 1000.0;
        let expected = (x.fract() * 32768.0) as i16;

        assert_eq!(
            actual, expected,
            "fractional part of {} is {}, but got {}",
            x, expected, actual
        );
    }

    #[rstest]
    #[case(0)]
    #[case(250)]
    #[case(1000)]
    #[case(6000)]
    #[case(30292)]
    #[case(32766)]
    #[case(-250)]
    #[case(-1000)]
    #[case(-6000)]
    #[case(-30292)]
    #[case(-32766)]
    fn test_from_i16_frac(#[case] x: i16) {
        let result = Number::from_i16_frac(x);
        let actual = result.to_f64();
        let expected = f64::from(x) / 32768.0;

        assert!(
            f64::abs(actual - expected) / (expected + 0.001) <= 0.01,
            "number from {} / 32768 = {} but got {}",
            x,
            expected,
            actual
        );
    }
}
