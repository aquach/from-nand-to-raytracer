use std::convert::TryFrom;
use std::fmt;

const MINVAL_I16: i16 = -32768;

// Jack has no right shift (left shift is a standard multiply).
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

fn leftshift_i16(x: i16, n: i16) -> i16 {
    let mut r = x;

    for _ in 0..n {
        r = r * 2;
    }

    r
}

// Top 12 bits of each i16 should be empty.
// It should really be u8 but we're simulating Jack which only has i16.
fn u4_array_mul_u4_array(u: &[i16; 8], v: &[i16; 8]) -> [i16; 16] {
    let mut w = [0i16; 16];

    for j in 0..8 {
        let mut k = 0;
        for i in 0..8 {
            // Perform signed 16-bit math that will never overflow because we only put u4s into it!
            // u4 * u4 = u8 < i16.
            // We can't do u8 * u8 = u16 because u16 < i16.
            let t = u[i] * v[j] + w[i + j] + k;
            w[i + j] = t & 0x0F;
            k = rightshift_i16(t, 4);
        }
        w[j + 8] = k;
    }

    w
}

// Number of leading zeros of x, pretending that it's a u8.
fn nlz_u8(x: i16) -> i16 {
    let mut r = 0;

    for shift in (0..8).rev() {
        if rightshift_i16(x, shift) == 0 {
            r += 1
        } else {
            break;
        }
    }

    r
}

// Upper byte of each i16 is empty.
fn u8_array_div_u8_array(u: &[i16; 8], v: &[i16; 4], v_size: usize) -> [i16; 8] {
    let base = 256; // Each chunk is 8 bits.

    let mut q = [255i16; 8];

    if v_size == 1 {
        // Simple algorithm for one-chunk divisor.

        // Rolling remainder.
        let mut k = 0;

        for j in (0..8).rev() {
            let val = k * base + u[j];
            q[j] = val / v[0];
            k = val - q[j] * v[0];
        }
        return q;
    }

    // Shift divisor so that high bit is set.
    let shift = nlz_u8(v[v_size - 1]); // - 16;

    println!("nlz: {}", shift);

    let mut vn = [0i16; 4];

    for i in (1..v_size).rev() {
        vn[i] = leftshift_i16(v[i], shift) | rightshift_i16(v[i - 1], 8 - shift);
    }

    vn[0] = leftshift_i16(v[0], shift) & 0xFF;

    println!("v before: {:?}, v after: {:?}", v, vn);

    let mut un = [0i16; 9];
    un[8] = rightshift_i16(u[7], 16 - shift);
    for i in (1..8).rev() {
        un[i] = (leftshift_i16(u[i], shift) | rightshift_i16(u[i - 1], 8 - shift)) & 0xFF;
    }
    un[0] = leftshift_i16(u[0], shift) & 0xFF;

    println!("u before: {:?}, u after: {:?}", u, un);

    for j in (0..=(8 - v_size)).rev() {
        // Compute estimate qhat of q[j].
        let val = un[j + v_size] * base + un[j + v_size - 1];
        let mut qhat = val / vn[v_size - 1];
        let mut rhat = val - qhat * vn[v_size - 1];

        println!(
            "j: {} val: {}, qhat: {}, divisor: {}, rhat: {}",
            j,
            val,
            qhat,
            vn[v_size - 1],
            rhat
        );

        loop {
            if qhat >= base
                || i32::from(qhat) * i32::from(vn[v_size - 2])
                    > i32::from(base) * i32::from(rhat) + i32::from(un[j + v_size - 2])
            {
                qhat = qhat - 1;
                rhat = rhat + vn[v_size - 1];
                if rhat < base {
                    continue;
                }
            }
            break;
        }

        // Multiply and subtract.
        let mut k = 0;
        let mut t;

        for i in 0..v_size {
            let p = qhat * vn[i];
            t = un[i + j] - k - (p & 0xFF);
            un[i + j] = t;
            k = (p >> 8) - (t >> 8);
        }

        t = un[j + v_size] - k;
        un[j + v_size] = t;

        q[j] = qhat;
        if t < 0 {
            q[j] = q[j] - 1;
            k = 0;
            for i in 0..v_size {
                t = un[i + j] + vn[i] + k;
                un[i + j] = t;
                k = t >> 8;
            }
            un[j + v_size] = un[j + v_size] + k;
        }
    }

    q
}

#[derive(Clone, Copy)]
pub struct Int32 {
    parts: [i16; 4], // Upper byte of each i16 is empty, which helps annoying overflow issues.
}

impl Int32 {
    pub fn from(i: i16) -> Int32 {
        let i_low = i & 0xFF;
        let i_high = rightshift_i16(i, 8);
        let smear = if i < 0 { 0xFF } else { 0 };
        let r = Int32 {
            parts: [i_low, i_high, smear, smear],
        };
        r.validate();
        r
    }

    pub fn do_add(&mut self, other: &Int32) {
        self.parts[0] += other.parts[0];
        self.parts[1] += other.parts[1];
        self.parts[2] += other.parts[2];
        self.parts[3] += other.parts[3];

        while self.parts[0] >= 256 {
            self.parts[0] -= 256;
            self.parts[1] += 1;
        }

        while self.parts[1] >= 256 {
            self.parts[1] -= 256;
            self.parts[2] += 1;
        }

        while self.parts[2] >= 256 {
            self.parts[2] -= 256;
            self.parts[3] += 1;
        }

        while self.parts[3] >= 256 {
            self.parts[3] -= 256;
        }

        self.validate();
    }

    pub fn do_sub(&mut self, other: &Int32) {
        let mut neg = *other;
        neg.do_neg();
        self.do_add(&neg);
    }

    pub fn do_mul(&mut self, other: &Int32) {
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

    pub fn do_mul_right_shift_bytes(&mut self, other: &Int32, right_shift_bytes: usize) {
        assert!(right_shift_bytes <= 3);

        let self_parts: &[i16; 4];
        let other_parts: &[i16; 4];

        let mut abs1: Int32;
        let mut abs2: Int32;

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

        let self_parts_expanded: [i16; 8] = [
            self_parts[0] & 0x0F,
            rightshift_i16(self_parts[0], 4),
            self_parts[1] & 0x0F,
            rightshift_i16(self_parts[1], 4),
            self_parts[2] & 0x0F,
            rightshift_i16(self_parts[2], 4),
            self_parts[3] & 0x0F,
            rightshift_i16(self_parts[3], 4),
        ];
        let other_parts_expanded: [i16; 8] = [
            other_parts[0] & 0x0F,
            rightshift_i16(other_parts[0], 4),
            other_parts[1] & 0x0F,
            rightshift_i16(other_parts[1], 4),
            other_parts[2] & 0x0F,
            rightshift_i16(other_parts[2], 4),
            other_parts[3] & 0x0F,
            rightshift_i16(other_parts[3], 4),
        ];

        let result = u4_array_mul_u4_array(&self_parts_expanded, &other_parts_expanded);

        self.parts[0] =
            result[(right_shift_bytes * 2) + 0] + result[(right_shift_bytes * 2) + 1] * 16;
        self.parts[1] =
            result[(right_shift_bytes * 2) + 2] + result[(right_shift_bytes * 2) + 3] * 16;
        self.parts[2] =
            result[(right_shift_bytes * 2) + 4] + result[(right_shift_bytes * 2) + 5] * 16;
        self.parts[3] =
            result[(right_shift_bytes * 2) + 6] + result[(right_shift_bytes * 2) + 7] * 16;

        if (right_shift_bytes * 2) + 8 < 16 && result[(right_shift_bytes * 2) + 8] != 0 {
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

    pub fn do_div(&mut self, other: &Int32) {
        self.do_left_shift_bytes_div(0, other);
    }

    pub fn do_left_shift_bytes_div(&mut self, left_shift_bytes: usize, other: &Int32) {
        assert!(left_shift_bytes <= 3);

        let self_parts: &[i16; 4];
        let other_parts: &[i16; 4];

        let mut abs1: Int32;
        let mut abs2: Int32;

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

        let self_parts_shifted: [i16; 8] = [
            if left_shift_bytes == 0 {
                self_parts[0 - left_shift_bytes]
            } else {
                0
            },
            if left_shift_bytes <= 1 {
                self_parts[1 - left_shift_bytes]
            } else {
                0
            },
            if left_shift_bytes <= 2 {
                self_parts[2 - left_shift_bytes]
            } else {
                0
            },
            if left_shift_bytes <= 3 {
                self_parts[3 - left_shift_bytes]
            } else {
                0
            },
            if left_shift_bytes >= 1 {
                self_parts[4 - left_shift_bytes]
            } else {
                0
            },
            if left_shift_bytes >= 2 {
                self_parts[5 - left_shift_bytes]
            } else {
                0
            },
            if left_shift_bytes >= 3 {
                self_parts[6 - left_shift_bytes]
            } else {
                0
            },
            0,
        ];

        let divisor_size = if other_parts[3] > 0 {
            4
        } else if other_parts[2] > 0 {
            3
        } else if other_parts[1] > 0 {
            2
        } else if other_parts[0] > 0 {
            1
        } else {
            panic!("Divide by zero trying to divide {} by {}", self, other);
        };

        println!(
            "Dividing {:?} by {:?}, divisor size {}",
            self_parts_shifted, other_parts, divisor_size
        );
        println!("result expected to be negative: {}", is_result_neg);

        let result = u8_array_div_u8_array(&self_parts_shifted, other_parts, divisor_size);

        println!("result: {:?}", result);

        self.parts[0] = result[0];
        self.parts[1] = result[1];
        self.parts[2] = result[2];
        self.parts[3] = result[3];

        if is_result_neg {
            self.do_neg();
        }

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

        let mut x = Int32::from(5);
        for _ in 0..20 {
            let mut inv = *self;
            inv.do_div(&x);

            x.do_add(&inv);
            x.do_div(&Int32::from(2));
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
        self.parts[0] == 0 && self.parts[1] == 0 && self.parts[2] == 0 && self.parts[3] == 0
    }

    pub fn is_negative(&self) -> bool {
        rightshift_i16(self.parts[3], 7) == 1
    }

    pub fn is_positive(&self) -> bool {
        !self.is_zero() && rightshift_i16(self.parts[3], 7) == 0
    }

    pub fn cmp(&self, other: &Int32) -> i16 {
        let mut r = *self;
        r.do_sub(&other);
        if r.is_zero() {
            0
        } else if r.is_negative() {
            -1
        } else {
            1
        }
    }

    fn validate(&self) {
        assert!(self.parts[0] >= 0);
        assert!(self.parts[0] <= 255);
        assert!(self.parts[1] >= 0);
        assert!(self.parts[1] <= 255);
        assert!(self.parts[2] >= 0);
        assert!(self.parts[2] <= 255);
        assert!(self.parts[3] >= 0);
        assert!(self.parts[3] <= 255);
    }

    pub fn to_i32(&self) -> i32 {
        (i32::from(self.parts[0]) << 0)
            + (i32::from(self.parts[1]) << 8)
            + (i32::from(self.parts[2]) << 16)
            + (i32::from(self.parts[3]) << 24)
    }
}

impl fmt::Debug for Int32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Int32[{} ({}, {}, {}, {})]",
            self.to_i32(),
            self.parts[0],
            self.parts[1],
            self.parts[2],
            self.parts[3],
        )
    }
}

impl fmt::Display for Int32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_i32(),)
    }
}

#[cfg(test)]
mod test {
    use super::nlz_u8;
    use super::Int32;

    fn test_one_mul(x: i16, y: i16) {
        let xi = Int32::from(x);
        let yi = Int32::from(y);

        let mut result = xi;
        result.do_mul(&yi);

        let actual = result.to_i32();
        let expected = i32::from(x) * i32::from(y);
        assert_eq!(
            actual, expected,
            "{} x {} = {} but got {}",
            x, y, expected, actual
        );
    }

    fn test_one_add(x: i16, y: i16) {
        let xi = Int32::from(x);
        let yi = Int32::from(y);

        let mut result = xi;
        result.do_add(&yi);

        let actual = result.to_i32();
        let expected = i32::from(x) + i32::from(y);
        assert_eq!(
            actual, expected,
            "{} + {} = {} but got {}",
            x, y, expected, actual
        );
    }

    fn test_one_sub(x: i16, y: i16) {
        let xi = Int32::from(x);
        let yi = Int32::from(y);

        let mut result = xi;
        result.do_sub(&yi);

        let actual = result.to_i32();
        let expected = i32::from(x) - i32::from(y);
        assert_eq!(
            actual, expected,
            "{} - {} = {} but got {}",
            x, y, expected, actual
        );
    }

    fn test_one_div(x: i16, y: i16) {
        let xi = Int32::from(x);
        let yi = Int32::from(y);

        let mut result = xi;
        result.do_div(&yi);

        let actual = result.to_i32();
        let expected = i32::from(x) / i32::from(y);
        assert_eq!(
            actual, expected,
            "{} / {} = {} but got {}",
            x, y, expected, actual
        );
    }

    fn test_one_cmp(x: i16, y: i16) {
        let xi = Int32::from(x);
        let yi = Int32::from(y);

        let actual = xi.cmp(&yi);

        let actual_cmp = if actual > 0 {
            "greater than"
        } else if actual == 0 {
            "equal to"
        } else {
            "less than"
        };

        let delta = x - y;
        let expected_cmp = if delta > 0 {
            "greater than"
        } else if delta == 0 {
            "equal to"
        } else {
            "less than"
        };

        assert_eq!(
            expected_cmp, actual_cmp,
            "Expected {} to be {} {} but it was {}",
            x, expected_cmp, y, actual_cmp
        );
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
        test_one_mul(255, 255);
    }

    #[test]
    fn test_div() {
        test_one_div(4, 3);
        test_one_div(100, 3);
        test_one_div(10000, 3);
        test_one_div(10000, 1);
        test_one_div(5000, 1000);
        test_one_div(100, 350);
        test_one_div(10, 5000);
        test_one_div(10098, 594);
        test_one_div(10099, 594);
        test_one_div(10097, 594);
        test_one_div(13, 2);
        test_one_div(13, 6);
        test_one_div(2382, 124);
        test_one_div(5000, 5000);
        test_one_div(32600, 32600);
        test_one_div(2, -2);
        test_one_div(4, -2);
        test_one_div(-4, -3);
        test_one_div(0, 5082);
    }

    #[test]
    fn test_is_negative() {
        assert_eq!(Int32::from(0).is_negative(), false);

        assert_eq!(Int32::from(-1).is_negative(), true);
        assert_eq!(Int32::from(-2).is_negative(), true);
        assert_eq!(Int32::from(-30000).is_negative(), true);

        assert_eq!(Int32::from(1).is_negative(), false);
        assert_eq!(Int32::from(2).is_negative(), false);
        assert_eq!(Int32::from(30000).is_negative(), false);
    }

    #[test]
    fn test_neg() {
        let mut x = Int32::from(0);
        x.do_neg();
        assert_eq!(x.to_i32(), 0);

        let mut x = Int32::from(1);
        x.do_neg();
        assert_eq!(x.to_i32(), -1);

        let mut x = Int32::from(-5);
        x.do_neg();
        assert_eq!(x.to_i32(), 5);

        let mut x = Int32::from(30000);
        x.do_neg();
        assert_eq!(x.to_i32(), -30000);

        let mut x = Int32::from(-30000);
        x.do_neg();
        assert_eq!(x.to_i32(), 30000);

        let mut x = Int32::from(256);
        x.do_neg();
        assert_eq!(x.to_i32(), -256);
    }

    #[test]
    fn test_abs() {
        let mut x = Int32::from(0);
        x.do_abs();
        assert_eq!(x.to_i32(), 0);

        let mut x = Int32::from(1);
        x.do_abs();
        assert_eq!(x.to_i32(), 1);

        let mut x = Int32::from(-5);
        x.do_abs();
        assert_eq!(x.to_i32(), 5);

        let mut x = Int32::from(30000);
        x.do_abs();
        assert_eq!(x.to_i32(), 30000);

        let mut x = Int32::from(-30000);
        x.do_abs();
        assert_eq!(x.to_i32(), 30000);
    }

    #[test]
    fn test_right_shift() {
        let mut x = Int32::from(0);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 0);

        let mut x = Int32::from(1);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 0);

        let mut x = Int32::from(-5);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 0);

        let mut x = Int32::from(30000);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 117);

        let mut x = Int32::from(-30000);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), -117);

        let mut x = Int32::from(256);
        let y = x;
        x.do_mul(&y);
        x.do_mul(&y);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 65536);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 256);
        x.do_right_shift_bytes(1);
        assert_eq!(x.to_i32(), 1);

        let mut x = Int32::from(256);
        x.do_mul(&y);
        x.do_mul(&y);
        x.do_right_shift_bytes(2);
        assert_eq!(x.to_i32(), 256);

        let mut x = Int32::from(256);
        x.do_mul(&y);
        x.do_mul(&y);
        x.do_right_shift_bytes(3);
        assert_eq!(x.to_i32(), 1);
    }

    #[test]
    fn test_left_shift() {
        let mut x = Int32::from(0);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), 0);

        let mut x = Int32::from(1);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), 256);

        let mut x = Int32::from(-5);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), -1280);

        let mut x = Int32::from(30000);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), 7680000);

        let mut x = Int32::from(-30000);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), -7680000);

        let mut x = Int32::from(1);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), 256);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), 65536);
        x.do_left_shift_bytes(1);
        assert_eq!(x.to_i32(), 16777216);

        let mut x = Int32::from(1);
        x.do_left_shift_bytes(2);
        assert_eq!(x.to_i32(), 65536);

        let mut x = Int32::from(1);
        x.do_left_shift_bytes(3);
        assert_eq!(x.to_i32(), 16777216);
    }

    #[test]
    fn test_cmp() {
        test_one_cmp(1, -1);
        test_one_cmp(0, 0);
        test_one_cmp(1, 1);
        test_one_cmp(-1, -1);

        test_one_cmp(20, -1);
        test_one_cmp(0, 0);
        test_one_cmp(1, 100);
        test_one_cmp(-1, -100);
        test_one_cmp(-5, 5);
        test_one_cmp(200, -380);

        test_one_cmp(-500, -400);
    }

    #[test]
    fn test_nlz_u8() {
        assert_eq!(nlz_u8(0), 8);
        assert_eq!(nlz_u8(1), 7);
        assert_eq!(nlz_u8(8), 4);
        assert_eq!(nlz_u8(9), 4);
        assert_eq!(nlz_u8(15), 4);
        assert_eq!(nlz_u8(16), 3);
        assert_eq!(nlz_u8(50), 2);
        assert_eq!(nlz_u8(120), 1);
        assert_eq!(nlz_u8(201), 0);
    }
}
