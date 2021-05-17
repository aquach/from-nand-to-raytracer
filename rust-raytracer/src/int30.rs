use std::convert::TryFrom;
use std::fmt;

const MINVAL_I16: i16 = -32768;

fn rightshift_i16(x: i16, n: i16) -> i16 {
    let mut r = x;

    for _ in 0..n {
        r = if r >= 0 {
            r / 2
        } else if r < -1 {
            r / 2 - MINVAL_I16
        } else {
            r - MINVAL_I16
        }
    }

    r
}

// Top 12 bits of each i16 should be 0.  We need to perform an unsigned multiply here which doubles
// the bit width, but we only have i16s in Jack, so we need to use u4 * u4 = u8 to fit in a i16.
fn mulmns(u: &[i16; 8], v: &[i16; 8]) -> [i16; 16] {
    let mut w = [0i16; 16];

    for j in 0..8 {
        let mut k = 0;
        for i in 0..8 {
            // Perform 8-bit math that will never overflow because u8 < i16.
            let t = u[i] * v[j] + w[i + j] + k;
            w[i + j] = t & 0x0F;
            k = rightshift_i16(t, 4);
        }
        w[j + 8] = k;
    }

    w
}

#[derive(Clone, Copy)]
pub struct Int30 {
    // Upper bit of each i16 is empty. We handle sign "inside" and there's no unsigned type in
    // Jack, so we're effectively treating it like a u15.
    parts: [i16; 2],
}

impl Int30 {
    pub fn from(i: i16) -> Int30 {
        let r = Int30 {
            parts: [
                i & 32768,
                rightshift_i16(i, 15),
            ],
        };
        r.validate();
        r
    }

    pub fn do_add(&mut self, other: &Int30) {
        self.parts[0] += other.parts[0];
        self.parts[1] += other.parts[1];

        while self.parts[0] >= 256 {
            self.parts[0] -= 256;
            self.parts[1] += 1;
        }

        while self.parts[1] >= 256 {
            self.parts[1] -= 256;
            self.parts[2] += 1;
        }

        self.validate();
    }

    pub fn do_sub(&mut self, other: &Int30) {
        let mut neg = *other;
        neg.do_neg();
        self.do_add(&neg);
    }

    pub fn do_mul(&mut self, other: &Int30) {
        self.do_mul_right_shift_bytes(other, 0);
    }

    pub fn do_right_shift_bytes(&mut self, bytes: usize) {
        assert!(bytes <= 3);
        if bytes == 0 {
            return;
        }

        let is_neg = self.is_negative();
        if is_neg {
            self.do_neg();
        }

        self.parts[0] = self.parts[bytes];
        self.parts[1] = if bytes + 1 <= 3 {
            self.parts[bytes + 1]
        } else {
            0
        };
        self.parts[2] = if bytes + 2 <= 3 {
            self.parts[bytes + 2]
        } else {
            0
        };
        self.parts[3] = 0;

        if is_neg {
            self.do_neg();
        }

        self.validate();
    }

    pub fn do_left_shift_bytes(&mut self, bytes: usize) {
        assert!(bytes <= 3);
        if bytes == 0 {
            return;
        }

        self.parts[3] = self.parts[3 - bytes];
        self.parts[2] = if 2 >= bytes { self.parts[2 - bytes] } else { 0 };
        self.parts[1] = if 1 >= bytes { self.parts[1 - bytes] } else { 0 };
        self.parts[0] = 0;

        self.validate();
    }

    pub fn do_mul_right_shift_bytes(&mut self, other: &Int30, right_shift_bytes: usize) {
        assert!(right_shift_bytes <= 3);

        let self_parts: &[i16; 8];
        let other_parts: &[i16; 8];

        let mut abs1: Int30;
        let mut abs2: Int30;

        let is_result_neg = self.is_negative() ^ other.is_negative();

        if self.is_negative() {
            abs1 = *self;
            abs1.do_abs();
            self_parts = &abs1.parts;
        } else {
            self_parts = &self.parts;
        }

        if other.is_negative() {
            abs2 = other.clone();
            abs2.do_abs();
            other_parts = &abs2.parts;
        } else {
            other_parts = &other.parts;
        }

        println!("{:?} {:?}", self_parts, other_parts);
        let result = mulmns(&self_parts, &other_parts);

        self.parts[0] = result[right_shift_bytes + 0];
        self.parts[1] = result[right_shift_bytes + 1];
        self.parts[2] = result[right_shift_bytes + 2];
        self.parts[3] = result[right_shift_bytes + 3];

        if (right_shift_bytes + 4 < 8 && result[right_shift_bytes + 4] != 0)
            || (right_shift_bytes + 5 < 8 && result[right_shift_bytes + 5] != 0)
            || (right_shift_bytes + 6 < 8 && result[right_shift_bytes + 6] != 0)
            || (right_shift_bytes + 7 < 8 && result[right_shift_bytes + 7] != 0)
        {
            panic!(
                "Overflow occurred multiplying {} by {} (and then right shift by {}). Result before shift: {:?}",
                self, other, right_shift_bytes, result
            );
        }

        if is_result_neg {
            self.do_neg();
        }

        self.validate();
    }

    pub fn do_div(&mut self, other: &Int30) {
        // CHEATING
        let result = self.to_i32() / other.to_i32();

        self.parts[0] = i16::try_from(result & 0xFF).unwrap();
        self.parts[1] = i16::try_from((result >> 8) & 0xFF).unwrap();
        self.parts[2] = i16::try_from((result >> 16) & 0xFF).unwrap();
        self.parts[3] = i16::try_from((result >> 24) & 0xFF).unwrap();

        self.validate();
    }

    pub fn do_left_shift_bytes_div(&mut self, left_shift_bytes: usize, other: &Int30) {
        // CHEATING
        let result =
            (i64::from(self.to_i32()) << (left_shift_bytes * 8)) / i64::from(other.to_i32());

        self.parts[0] = i16::try_from(result & 0xFF).unwrap();
        self.parts[1] = i16::try_from((result >> 8) & 0xFF).unwrap();
        self.parts[2] = i16::try_from((result >> 16) & 0xFF).unwrap();
        self.parts[3] = i16::try_from((result >> 24) & 0xFF).unwrap();

        self.validate();
    }

    pub fn do_abs(&mut self) {
        if self.is_negative() {
            self.do_neg();
        }
        self.validate();
    }

    pub fn do_sqrt(&mut self) {
        if self.is_zero() {
            return;
        }

        let mut x = Int30::from(5);
        for _ in 0..15 {
            let mut inv = *self;
            inv.do_div(&x);

            x.do_add(&inv);
            x.do_div(&Int30::from(2));
        }

        self.parts = x.parts;
    }

    pub fn do_neg(&mut self) {
        self.parts[0] = !self.parts[0] & 0xFF;
        self.parts[1] = !self.parts[1] & 0xFF;
        self.parts[2] = !self.parts[2] & 0xFF;
        self.parts[3] = !self.parts[3] & 0xFF;
        self.parts[0] += 1;

        if self.parts[0] >= 256 {
            self.parts[0] -= 256;
            self.parts[1] += 1;
        }

        if self.parts[1] >= 256 {
            self.parts[1] -= 256;
            self.parts[2] += 1;
        }

        if self.parts[2] >= 256 {
            self.parts[2] -= 256;
            self.parts[3] += 1;
        }

        if self.parts[3] >= 256 {
            self.parts[3] -= 256;
        }

        self.validate();
    }

    pub fn is_zero(&self) -> bool {
        self.parts.iter().all(|v| *v == 0)
    }

    pub fn is_negative(&self) -> bool {
        rightshift_i16(self.parts[7], 7) == 1
    }

    fn validate(&self) {
        for (i, v) in self.parts.iter().enumerate() {
            assert!(*v >= 0 && *v <= 255, "index {} in {} was out of bounds: value was {}", i, self, v);
        }
    }

    pub fn to_i32(&self) -> i32 {
        (i32::from(self.parts[0]) << 0)
            + (i32::from(self.parts[1]) << 4)
            + (i32::from(self.parts[2]) << 8)
            + (i32::from(self.parts[3]) << 12)
            + (i32::from(self.parts[4]) << 16)
            + (i32::from(self.parts[5]) << 20)
            + (i32::from(self.parts[6]) << 24)
            + (i32::from(self.parts[7]) << 28)
    }
}

impl fmt::Debug for Int30 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Int30[{} ({}, {}, {}, {})]",
            self.to_i32(),
            self.parts[0],
            self.parts[1],
            self.parts[2],
            self.parts[3],
        )
    }
}

impl fmt::Display for Int30 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_i32(),)
    }
}

#[cfg(test)]
mod test {
    use super::Int30;

    fn test_one_mul(x: i16, y: i16) {
        let xi = Int30::from(x);
        let yi = Int30::from(y);

        let mut result = xi;
        result.do_mul(&yi);

        println!("{} * {} = {}", xi, yi, result);

        let actual = result.to_i32();
        let expected = i32::from(x) * i32::from(y);
        assert_eq!(
            actual, expected,
            "{} x {} = {} but got {}",
            x, y, expected, actual
        );
    }

    fn test_one_add(x: i16, y: i16) {
        let xi = Int30::from(x);
        let yi = Int30::from(y);

        let mut result = xi;
        result.do_add(&yi);

        println!("{} + {} = {}", xi, yi, result);

        assert_eq!(result.to_i32(), i32::from(x) + i32::from(y));
    }

    fn test_one_sub(x: i16, y: i16) {
        let xi = Int30::from(x);
        let yi = Int30::from(y);

        let mut result = xi;
        result.do_sub(&yi);

        println!("{} - {} = {}", xi, yi, result);

        assert_eq!(result.to_i32(), i32::from(x) - i32::from(y));
    }

    fn test_one_div(x: i16, y: i16) {
        let xi = Int30::from(x);
        let yi = Int30::from(y);

        let mut result = xi;
        result.do_div(&yi);

        println!("{} / {} = {}", xi, yi, result);

        assert_eq!(result.to_i32(), i32::from(x) / i32::from(y));
    }

    #[test]
    fn test_add() {
        test_one_add(4, 3);
        test_one_add(100, 3);
        test_one_add(100, 350);
        test_one_add(10, 5000);
        test_one_add(5000, 6000);
        test_one_add(13082, 23082);
        test_one_add(32600, 32600);
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
        test_one_mul(100, 350);
        test_one_mul(10, 5000);
        test_one_mul(5000, 5000);
        test_one_mul(32600, 32600);
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
        assert_eq!(Int30::from(0).is_negative(), false);

        assert_eq!(Int30::from(-1).is_negative(), true);
        assert_eq!(Int30::from(-2).is_negative(), true);
        assert_eq!(Int30::from(-30000).is_negative(), true);

        assert_eq!(Int30::from(1).is_negative(), false);
        assert_eq!(Int30::from(2).is_negative(), false);
        assert_eq!(Int30::from(30000).is_negative(), false);
    }

    #[test]
    fn test_neg() {
        let mut x = Int30::from(0);
        x.do_neg();
        assert_eq!(x.to_i32(), 0);

        let mut x = Int30::from(1);
        x.do_neg();
        assert_eq!(x.to_i32(), -1);

        let mut x = Int30::from(-5);
        x.do_neg();
        assert_eq!(x.to_i32(), 5);

        let mut x = Int30::from(30000);
        x.do_neg();
        assert_eq!(x.to_i32(), -30000);

        let mut x = Int30::from(-30000);
        x.do_neg();
        assert_eq!(x.to_i32(), 30000);

        let mut x = Int30::from(256);
        x.do_neg();
        assert_eq!(x.to_i32(), -256);
    }

    #[test]
    fn test_abs() {
        let mut x = Int30::from(0);
        x.do_abs();
        assert_eq!(x.to_i32(), 0);

        let mut x = Int30::from(1);
        x.do_abs();
        assert_eq!(x.to_i32(), 1);

        let mut x = Int30::from(-5);
        x.do_abs();
        assert_eq!(x.to_i32(), 5);

        let mut x = Int30::from(30000);
        x.do_abs();
        assert_eq!(x.to_i32(), 30000);

        let mut x = Int30::from(-30000);
        x.do_abs();
        assert_eq!(x.to_i32(), 30000);
    }

    #[test]
    fn test_right_shift() {
        let mut x = Int30::from(0);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 0);

        let mut x = Int30::from(1);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 0);

        let mut x = Int30::from(-5);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 0);

        let mut x = Int30::from(30000);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 117);

        let mut x = Int30::from(-30000);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), -117);

        let mut x = Int30::from(256);
        let y = x;
        x.do_mul(&y);
        x.do_mul(&y);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 65536);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 256);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 1);

        let mut x = Int30::from(256);
        x.do_mul(&y);
        x.do_mul(&y);
        x.do_right_shift_bytes(2);
        assert_eq!(x.to_i32(), 256);

        let mut x = Int30::from(256);
        x.do_mul(&y);
        x.do_mul(&y);
        x.do_right_shift_bytes(3);
        assert_eq!(x.to_i32(), 1);
    }

    #[test]
    fn test_left_shift() {
        let mut x = Int30::from(0);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), 0);

        let mut x = Int30::from(1);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), 256);

        let mut x = Int30::from(-5);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), -1280);

        let mut x = Int30::from(30000);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), 7680000);

        let mut x = Int30::from(-30000);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), -7680000);

        let mut x = Int30::from(1);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), 256);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), 65536);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), 16777216);

        let mut x = Int30::from(1);
        x.do_left_shift_bytes(2);
        assert_eq!(x.to_i32(), 65536);

        let mut x = Int30::from(1);
        x.do_left_shift_bytes(3);
        assert_eq!(x.to_i32(), 16777216);
    }
}
