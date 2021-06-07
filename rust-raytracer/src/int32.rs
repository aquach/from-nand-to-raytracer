use std::convert::TryFrom;
use std::fmt;

// Jack doesn't have these, so we need to reimplement them.
fn arith_rightshift(x: i16, n: i16) -> i16 {
    if x == 0 {
        return 0;
    }

    if n == 0 {
        return x;
    }

    let mut r = x;

    if x > 0 {
        let i = [ 1, 2, 4, 8, 16, 32, 64, 128, 256 ];
        r = r / i[usize::try_from(n).unwrap()];
    } else {
        for _ in 0..n {
            let divided = r / 2;
            r = if r & 1 == 0 {
                divided
            } else {
                divided - 1
            };
            if r == -1 {
                return r;
            }
        }
    }

    r
}

fn leftshift(x: i16, n: i16) -> i16 {
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
        let vj = v[j];
        for i in 0..8 {
            // Perform signed 16-bit math that will never overflow because we only put u4s into it!
            // u4 * u4 = u8, which fits in a i16.
            // We can't do use u8s because u8 * u8 = u16 and u16s don't fit in i16.
            let t = u[i] * vj + w[i + j] + k;
            w[i + j] = t & 0x0F;
            k = arith_rightshift(t, 4);
        }
        w[j + 8] = k;
    }

    w
}

// Number of leading zeros of x, pretending that it's a u4.
fn nlz_u4(x: i16) -> i16 {
    let mut r = 0;

    for shift in (0..4).rev() {
        if arith_rightshift(x, shift) == 0 {
            r += 1
        } else {
            break;
        }
    }

    r
}

// Upper 12 bits of each i16 are empty.
fn u4_array_div_u4_array(u: &[i16; 16], v: &[i16; 8], v_size: usize) -> [i16; 16] {
    let base = 16; // Each chunk is 4 bits.

    let mut q = [0i16; 16];

    if v_size == 1 {
        // Simple algorithm for one-chunk divisor.

        // Rolling remainder.
        let mut k = 0;

        for j in (0..16).rev() {
            let val = k * base + u[j];
            q[j] = val / v[0];
            k = val - q[j] * v[0];
        }
        return q;
    }

    // Shift divisor so that high bit is set.
    let shift = nlz_u4(v[v_size - 1]);

    let mut vn = [0i16; 8];

    for i in (1..v_size).rev() {
        vn[i] = (leftshift(v[i], shift) | arith_rightshift(v[i - 1], 4 - shift)) & 0x0F;
    }

    vn[0] = leftshift(v[0], shift) & 0x0F;

    let mut dividend = [0i16; 17];
    dividend[16] = arith_rightshift(u[15], 4 - shift);
    for i in (1..16).rev() {
        dividend[i] = (leftshift(u[i], shift) | arith_rightshift(u[i - 1], 4 - shift)) & 0x0F;
    }
    dividend[0] = leftshift(u[0], shift) & 0x0F;

    // Crank out the quotient one digit at a time, from most significant to least.
    for j in (0..=(16 - v_size)).rev() {
        // Compute estimate qhat of q[j].
        let val = dividend[j + v_size] * base + dividend[j + v_size - 1];
        let mut qhat = val / vn[v_size - 1];
        let mut rhat = val - qhat * vn[v_size - 1];

        loop {
            if qhat >= base || qhat * vn[v_size - 2] > base * rhat + dividend[j + v_size - 2] {
                qhat = qhat - 1;
                rhat = rhat + vn[v_size - 1];
                if rhat < base {
                    continue;
                }
            }
            break;
        }

        // Multiply and subtract.
        let mut carry: i16 = 0;

        for i in 0..v_size {
            let multiplied = qhat * vn[i];
            let t = dividend[i + j] - carry - (multiplied & 0x0F);

            // Simulate wrap-around math. We track the carry below independently.
            dividend[i + j] = t & 0x0F;

            carry = arith_rightshift(multiplied, 4) - arith_rightshift(t, 4);
        }

        let t = dividend[j + v_size] - carry;
        dividend[j + v_size] = t;

        q[j] = qhat;

        if t < 0 {
            carry = 0;
            q[j] = q[j] - 1;
            for i in 0..v_size {
                let t = dividend[i + j] + vn[i] + carry;
                dividend[i + j] = t & 0x0F;
                carry = arith_rightshift(t, 4)
            }
            dividend[j + v_size] = dividend[j + v_size] + carry;
        }
    }

    q
}

#[derive(Clone, Copy)]
pub struct Int32 {
    pub parts: [i16; 4], // Upper byte of each i16 is empty, which helps annoying overflow issues.
}

impl Int32 {
    pub fn from(i: i16) -> Int32 {
        let i_low = i & 0xFF;
        let i_high = arith_rightshift(i, 8) & 0xFF;
        let smear = if i < 0 { 0xFF } else { 0 };
        let r = Int32 {
            parts: [i_low, i_high, smear, smear],
        };
        r.validate();
        r
    }

    pub fn from_i32(i: i32) -> Int32 {
        let r = Int32 {
            parts: [
                i16::try_from(i & 0xFF).unwrap(),
                i16::try_from((i >> 8) & 0xFF).unwrap(),
                i16::try_from((i >> 16) & 0xFF).unwrap(),
                i16::try_from((i >> 24) & 0xFF).unwrap(),
            ],
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
            arith_rightshift(self_parts[0], 4),
            self_parts[1] & 0x0F,
            arith_rightshift(self_parts[1], 4),
            self_parts[2] & 0x0F,
            arith_rightshift(self_parts[2], 4),
            self_parts[3] & 0x0F,
            arith_rightshift(self_parts[3], 4),
        ];
        let other_parts_expanded: [i16; 8] = [
            other_parts[0] & 0x0F,
            arith_rightshift(other_parts[0], 4),
            other_parts[1] & 0x0F,
            arith_rightshift(other_parts[1], 4),
            other_parts[2] & 0x0F,
            arith_rightshift(other_parts[2], 4),
            other_parts[3] & 0x0F,
            arith_rightshift(other_parts[3], 4),
        ];

        let result = u4_array_mul_u4_array(&self_parts_expanded, &other_parts_expanded);

        let right_shift_bytes_2 = right_shift_bytes * 2;

        self.parts[0] =
            result[right_shift_bytes_2 + 0] + result[right_shift_bytes_2 + 1] * 16;
        self.parts[1] =
            result[right_shift_bytes_2 + 2] + result[right_shift_bytes_2 + 3] * 16;
        self.parts[2] =
            result[right_shift_bytes_2 + 4] + result[right_shift_bytes_2 + 5] * 16;
        self.parts[3] =
            result[right_shift_bytes_2 + 6] + result[right_shift_bytes_2 + 7] * 16;

        if right_shift_bytes_2 + 8 < 16 && result[right_shift_bytes_2 + 8] != 0 {
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

        if other.is_zero() {
            panic!("Divide by zero trying to divide {} by {}", self, other);
        }

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

        let self_parts_expanded: [i16; 16] = [
            self_parts_shifted[0] & 0x0F,
            arith_rightshift(self_parts_shifted[0], 4),
            self_parts_shifted[1] & 0x0F,
            arith_rightshift(self_parts_shifted[1], 4),
            self_parts_shifted[2] & 0x0F,
            arith_rightshift(self_parts_shifted[2], 4),
            self_parts_shifted[3] & 0x0F,
            arith_rightshift(self_parts_shifted[3], 4),
            self_parts_shifted[4] & 0x0F,
            arith_rightshift(self_parts_shifted[4], 4),
            self_parts_shifted[5] & 0x0F,
            arith_rightshift(self_parts_shifted[5], 4),
            self_parts_shifted[6] & 0x0F,
            arith_rightshift(self_parts_shifted[6], 4),
            self_parts_shifted[7] & 0x0F,
            arith_rightshift(self_parts_shifted[7], 4),
        ];
        let other_parts_expanded: [i16; 8] = [
            other_parts[0] & 0x0F,
            arith_rightshift(other_parts[0], 4),
            other_parts[1] & 0x0F,
            arith_rightshift(other_parts[1], 4),
            other_parts[2] & 0x0F,
            arith_rightshift(other_parts[2], 4),
            other_parts[3] & 0x0F,
            arith_rightshift(other_parts[3], 4),
        ];

        let mut divisor_size = 255;

        for i in (0..8).rev() {
            if other_parts_expanded[i] > 0 {
                divisor_size = i + 1;
                break;
            }
        }

        assert_ne!(divisor_size, 255);

        let result =
            u4_array_div_u4_array(&self_parts_expanded, &other_parts_expanded, divisor_size);

        self.parts[0] = result[0] + result[1] * 16;
        self.parts[1] = result[2] + result[3] * 16;
        self.parts[2] = result[4] + result[5] * 16;
        self.parts[3] = result[6] + result[7] * 16;

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

    fn initial_sqrt_guess(&self) -> Int32 {
        if self.parts[3] > 0 {
            let mut x = Int32::from(11);
            x.do_mul(&Int32::from(4096));
            return x;
        }
        if self.parts[2] > 0 {
            let mut x = Int32::from(11);
            x.do_mul(&Int32::from(256));
            return x;
        }
        if self.parts[1] > 0 {
            let mut x = Int32::from(11);
            x.do_mul(&Int32::from(16));
            return x;
        }

        return Int32::from(11);
    }

    pub fn do_sqrt(&mut self) {
        if self.is_negative() {
            panic!();
        }

        if self.is_zero() {
            return;
        }

        let mut guess = self.initial_sqrt_guess();
        for _ in 0..20 {
            let mut inv = *self;
            inv.do_div(&guess);

            let mut new_guess = guess;
            new_guess.do_add(&inv);
            new_guess.do_div(&Int32::from(2));

            if new_guess.cmp(&guess) == 0 {
                break;
            }

            guess = new_guess;
        }

        self.parts = guess.parts;
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
        self.parts[3] >= 128
    }

    pub fn is_positive(&self) -> bool {
        !self.is_zero() && self.parts[3] < 128
    }

    pub fn do_zero(&mut self) {
        self.parts[0] = 0;
        self.parts[1] = 0;
        self.parts[2] = 0;
        self.parts[3] = 0;
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
        assert!(
            self.parts[0] >= 0,
            "Encountered bad state: {:?}",
            self.parts
        );
        assert!(
            self.parts[0] <= 255,
            "Encountered bad state: {:?}",
            self.parts
        );
        assert!(
            self.parts[1] >= 0,
            "Encountered bad state: {:?}",
            self.parts
        );
        assert!(
            self.parts[1] <= 255,
            "Encountered bad state: {:?}",
            self.parts
        );
        assert!(
            self.parts[2] >= 0,
            "Encountered bad state: {:?}",
            self.parts
        );
        assert!(
            self.parts[2] <= 255,
            "Encountered bad state: {:?}",
            self.parts
        );
        assert!(
            self.parts[3] >= 0,
            "Encountered bad state: {:?}",
            self.parts
        );
        assert!(
            self.parts[3] <= 255,
            "Encountered bad state: {:?}",
            self.parts
        );
    }

    pub fn to_i32(&self) -> i32 {
        (i32::from(self.parts[0]) << 0)
            + (i32::from(self.parts[1]) << 8)
            + (i32::from(self.parts[2]) << 16)
            + (i32::from(self.parts[3]) << 24)
    }

    pub fn is_even(&self) -> bool {
        (self.parts[0] & 0x1) == 0
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
    use super::nlz_u4;
    use super::Int32;
    use rstest::*;

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
    #[case(2355744, 1534)]
    fn test_div(#[case] x: i32, #[case] y: i32) {
        let xi = Int32::from_i32(x);
        let yi = Int32::from_i32(y);

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
    fn test_nlz_u4() {
        assert_eq!(nlz_u4(0), 4);
        assert_eq!(nlz_u4(1), 3);
        assert_eq!(nlz_u4(2), 2);
        assert_eq!(nlz_u4(5), 1);
        assert_eq!(nlz_u4(8), 0);
        assert_eq!(nlz_u4(9), 0);
        assert_eq!(nlz_u4(15), 0);
    }

    #[rstest]
    #[case(4, 3)]
    #[case(100, 3)]
    #[case(10000, 3)]
    #[case(10000, 1)]
    #[case(-15, 2)]
    #[case(-14, 2)]
    #[case(-13, 2)]
    #[case(-12, 2)]
    #[case(-11, 2)]
    #[case(-15, 1)]
    #[case(-14, 1)]
    #[case(-13, 1)]
    #[case(-12, 1)]
    #[case(-11, 1)]
    #[case(-5, 1)]
    #[case(-4, 1)]
    #[case(-3, 1)]
    #[case(-2, 1)]
    #[case(-1, 1)]
    #[case(-5, 2)]
    #[case(-4, 2)]
    #[case(-3, 2)]
    #[case(-2, 2)]
    #[case(-1, 2)]
    #[case(0, 1)]
    #[case(1, 0)]
    #[case(15, 0)]
    #[case(0, 4)]
    fn test_arith_rightshift(#[case] x: i16, #[case] s: i16) {
        let actual = super::arith_rightshift(x, s);
        let expected = x >> s;
        assert_eq!(
            actual, expected,
            "{} >> {} = {} but got {}",
            x, s, expected, actual
        );
    }

    #[rstest]
    #[case(-2147483648)]
    #[case(-1024)]
    #[case(-256)]
    #[case(-255)]
    #[case(-1)]
    #[case(0)]
    #[case(1)]
    #[case(255)]
    #[case(256)]
    #[case(1024)]
    #[case(2147483647)]
    fn test_is_negative(#[case] x: i32) {
        let actual = Int32::from_i32(x).is_negative();
        let expected = x < 0;
        assert_eq!(
            actual, expected,
            "{} < 0 = {} but got {}",
            x, expected, actual
        );
    }

    #[rstest]
    #[case(-2147483648)]
    #[case(-1024)]
    #[case(-256)]
    #[case(-255)]
    #[case(-1)]
    #[case(0)]
    #[case(1)]
    #[case(255)]
    #[case(256)]
    #[case(1024)]
    #[case(2147483647)]
    fn test_is_positive(#[case] x: i32) {
        let actual = Int32::from_i32(x).is_positive();
        let expected = x > 0;
        assert_eq!(
            actual, expected,
            "{} < 0 = {} but got {}",
            x, expected, actual
        );
    }

    #[rstest]
    #[case(-2147483648)]
    #[case(-1024)]
    #[case(-256)]
    #[case(-255)]
    #[case(-1)]
    #[case(0)]
    #[case(1)]
    #[case(255)]
    #[case(256)]
    #[case(1024)]
    #[case(2147483647)]
    fn test_is_even(#[case] x: i32) {
        let actual = Int32::from_i32(x).is_even();
        let expected = x % 2 == 0;
        assert_eq!(
            actual, expected,
            "is {} even? {}, but got {}",
            x, expected, actual
        );
    }
}
