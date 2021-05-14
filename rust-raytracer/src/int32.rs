use std::fmt;
use std::num::Wrapping;

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

const MINVAL_I8: i8 = -128;

fn rightshift_i8(x: i8, n: i8) -> i8 {
    let mut r = x;

    for _ in 0..n {
        r = if r >= 0 {
            r / 2
        } else if r < -1 {
            r / 2 - MINVAL_I8
        } else {
            r - MINVAL_I8
        }
    }

    r
}

// Top byte of each i16 should be empty.
// It should really be u8 but we're simulating Jack which only has i16.
fn mulmns(u: &[i16; 4], v: &[i16; 4]) -> [i16; 8] {
    let mut w = [0i16; 8];

    for j in 0..4 {
        let mut k = 0;
        for i in 0..4 {
            // Perform 16-bit math that will never overflow because we only put i8s into it!
            let t = u[i] * v[j] + w[i + j] + k;
            w[i + j] = t & 0xFF;
            k = rightshift_i16(t, 8);
        }
        w[j + 4] = k;
    }

    w
}

#[derive(Debug, Clone, Copy)]
struct Int32 {
    parts: [i16; 4], // Upper byte of each i16 is empty.
}

impl Int32 {
    pub fn do_multiply(&mut self, other: &Int32) {
        let meparts: &[i16; 4];
        let otherparts: &[i16; 4];
        let result: [i16; 8];

        if !self.is_negative() && !other.is_negative() {
            meparts = &self.parts;
            otherparts = &other.parts;
            result = mulmns(&meparts, &otherparts);
        } else if self.is_negative() {
            let mut abs = self.clone();
            abs.do_abs();
            meparts = &abs.parts;
            otherparts = &other.parts;
            result = mulmns(&meparts, &otherparts);
        } else {
            meparts = &self.parts;
            let mut abs = other.clone();
            abs.do_abs();
            otherparts = &abs.parts;
            result = mulmns(&meparts, &otherparts);
        }

        if result[4] != 0 || result[5] != 0 || result[6] != 0 || result[7] != 0 {
            panic!("Overflow multiplying {} by {}", self, other);
        }
        self.parts[0] = result[0];
        self.parts[1] = result[1];
        self.parts[2] = result[2];
        self.parts[3] = result[3];
    }

    pub fn do_abs(&mut self) {}

    pub fn do_neg(&mut self) {
        self.parts[0] = result[0];
        self.parts[1] = result[1];
        self.parts[2] = result[2];
        self.parts[3] = result[3];
    }

    pub fn is_zero(&self) -> bool {
        self.parts[0] == 0 && self.parts[1] == 0
    }

    pub fn is_negative(&self) -> bool {
        self.parts[1] < 0
    }
}

impl fmt::Display for Int32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Int32({}, {}, {}, {})",
            self.parts[0],
            self.parts[1],
            self.parts[2],
            self.parts[3],
        )
    }
}

fn main() {}

#[cfg(test)]
mod test {
    // use super::mult32bit;
    use super::mulmns;
    use super::rightshift_i16;

    fn test_mult(x: i16, y: i16) {
        let x_low = x & 0xFF;
        let x_high = rightshift_i16(x, 8);

        let y_low = y & 0xFF;
        let y_high = rightshift_i16(y, 8);

        let [r0, r1, r2, r3, r4, r5, r6, r7] =
            mulmns(&[x_low, x_high, 0, 0], &[y_low, y_high, 0, 0]);

        let v = (i64::from(r0) << 0)
            + (i64::from(r1) << 8)
            + (i64::from(r2) << 16)
            + (i64::from(r3) << 24)
            + (i64::from(r4) << 32)
            + (i64::from(r5) << 40)
            + (i64::from(r6) << 48)
            + (i64::from(r7) << 56);

        println!(
            "{} ({} {}) * {} ({} {}) = {} ({} {} {} {} {} {} {} {})",
            x, x_low, x_high, y, y_low, y_high, v, r0, r1, r2, r3, r4, r5, r6, r7,
        );

        assert_eq!(v, i64::from(x) * i64::from(y));
    }

    #[test]
    fn test_mulmns() {
        test_mult(4, 3);
        test_mult(100, 3);
        test_mult(100, 350);
        test_mult(10, 5000);
        test_mult(5000, 5000);
        test_mult(32600, 32600);
        test_mult(2, -2);
        test_mult(4, -2);
        test_mult(-4, -3);

        //         let (high, low) = mulmns(2, -2);
        //         let v = (i32::from(high) << 16) + i32::from(low);
        //         assert_eq!(v, 2 * -2);

        //         let (high, low) = mulmns(4, -3);
        //         let v = (i32::from(high) << 16) + i32::from(low);
        //         assert_eq!(v, 4 * -3);

        //         let (high, low) = mulmns(-4, 3);
        //         let v = (i32::from(high) << 16) + i32::from(low);
        //         assert_eq!(v, -4 * 3);

        //         let (high, low) = mulmns(10, 5000);
        //         let v = (i32::from(high) << 16) + i32::from(low);
        //         assert_eq!(v, 10 * 5000);

        //         let (high, low) = mulmns(5000, 5000);
        //         let v = (i32::from(high) << 16) + i32::from(low);
        //         assert_eq!(v, 5000 * 5000);
    }
}
