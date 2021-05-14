use std::convert::TryFrom;
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

// Top bytes of each i16 should be empty.
// It should really be u8 but we're simulating Jack which only has i16.
fn mulmns(u: [i16; 4], v: [i16; 4]) -> [i16; 8] {
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

//fn mult32bit(x: i16, y: i16) -> (i16, i16) {
//    let x_low = x & 255;
//    let x_high = rightshift(x, 8);

//    let y_low = y & 255;
//    let y_high = rightshift(y, 8);

//    let xlyl = x_low * y_low;
//    let xlyh = x_low * y_high;
//    let xhyl = x_high * y_low;
//    let xhyh = x_high * y_high;

//    let mid16 = xlyh + xhyl;
//    let mid16_low = mid16 & 255;
//    let mid16_high = rightshift(mid16, 8);

//    println!(
//        "x: {} y: {} xl: {} yl: {} xh: {} yh: {} xlyh: {} xhyl: {} mid16: {} mid16_low: {} mid16_high: {}",
//        x,
//        y,
//        x_low,
//        y_low,
//        x_high,
//        y_high,
//        xlyh,
//        xhyl,
//        mid16,
//        mid16_low,
//        mid16_high
//    );

//    //   0    2
//    //  255 254
//    //
//    //       1012
//    //  0
//    //  510

//    println!("{}", Wrapping(xhyh));
//    println!("{}", Wrapping(mid16) * Wrapping(256));
//    println!("{}", Wrapping(mid16) * Wrapping(256) + Wrapping(xlyl));

//    (xhyh, xlyl + i16::wrapping_mul(mid16, 256))
//}

fn main() {}

#[cfg(test)]
mod test {
    // use super::mult32bit;
    use super::mulmns;
    use super::rightshift_i16;
    use std::convert::TryFrom;

    // #[test]
    // fn test_mult16bit() {
    //     let (high, low) = mult32bit(4, 3);
    //     assert_eq!(high, 0);
    //     assert_eq!(low, 12);
    //     let v = (i32::from(high) << 16) + i32::from(low);
    //     assert_eq!(v, 4 * 3);

    //     let (high, low) = mult32bit(2, -2);
    //     assert_eq!(high, 0);
    //     assert_eq!(low, -4);
    //     let v = (i32::from(high) << 16) + i32::from(low);
    //     assert_eq!(v, 2 * -2);

    //     let (high, low) = mult32bit(4, -3);
    //     assert_eq!(high, 0);
    //     assert_eq!(low, -12);
    //     let v = (i32::from(high) << 16) + i32::from(low);
    //     assert_eq!(v, 4 * -3);

    //     let (high, low) = mult32bit(-4, 3);
    //     let v = (i32::from(high) << 16) + i32::from(low);
    //     assert_eq!(v, -4 * 3);

    //     let (high, low) = mult32bit(10, 5000);
    //     let v = (i32::from(high) << 16) + i32::from(low);
    //     assert_eq!(v, 10 * 5000);

    //     let (high, low) = mult32bit(5000, 5000);
    //     let v = (i32::from(high) << 16) + i32::from(low);
    //     assert_eq!(v, 5000 * 5000);
    // }

    fn test_mult(x: i16, y: i16) {
        let x_low = x & 0xFF;
        let x_high = rightshift_i16(x, 8);

        let y_low = y & 0xFF;
        let y_high = rightshift_i16(y, 8);

        let [r0, r1, r2, r3, r4, r5, r6, r7] = mulmns([x_low, x_high, 0, 0], [y_low, y_high, 0, 0]);

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
