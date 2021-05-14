use std::fmt;

const FLOAT_BITS: u32 = 7;
const SCALE_FACTOR: i16 = i16::pow(2, FLOAT_BITS);

#[derive(Debug, Clone, Copy, Eq)]
pub struct Fixed16(i16);

impl Fixed16 {
    pub fn from(i: i16) -> Fixed16 {
        Fixed16(i * SCALE_FACTOR)
    }

    pub fn int_part(&self) -> i16 {
        self.0 / SCALE_FACTOR
    }

    pub fn frac_part(&self) -> i16 {
        self.0 - SCALE_FACTOR * self.int_part()
    }

    pub fn is_zero(&self) -> bool {
        self.0 == 0
    }
}

impl std::ops::Div<Fixed16> for Fixed16 {
    type Output = Fixed16;
    fn div(self, r: Fixed16) -> Fixed16 {
        println!("{} {}", self.0, r.0);
        jj  Fixed16(self.0 * SCALE_FACTOR / r.0)
    }
}

impl std::ops::Sub<Fixed16> for Fixed16 {
    type Output = Fixed16;
    fn sub(self, r: Fixed16) -> Fixed16 {
        Fixed16(self.0 - r.0)
    }
}

impl std::ops::Add<Fixed16> for Fixed16 {
    type Output = Fixed16;
    fn add(self, r: Fixed16) -> Fixed16 {
        Fixed16(self.0 + r.0)
    }
}

impl std::ops::Mul<Fixed16> for Fixed16 {
    type Output = Fixed16;
    fn mul(self, r: Fixed16) -> Fixed16 {
        Fixed16(self.0 * r.0 / SCALE_FACTOR)
    }
}

impl PartialEq for Fixed16 {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl fmt::Display for Fixed16 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Fixed16({} int, {} fraction)",
            self.int_part(),
            self.frac_part()
        )
    }
}

#[cfg(test)]
mod test {
    use super::Fixed16;

    #[test]
    fn fixed() {
        // assert_eq!(true, (Fixed16::from(6) / Fixed16::from(10) - Fixed16::from(3) / Fixed16::from(5)).is_zero());
        // assert_eq!(true, (Fixed16::from(6) / Fixed16::from(100) - Fixed16::from(3) / Fixed16::from(50)).is_zero());
        // assert_eq!(Fixed16::from(42), Fixed16::from(6) * Fixed16::from(7));
        assert_eq!(Fixed16::from(10), Fixed16::from(12) - Fixed16::from(2));
        assert_eq!(Fixed16::from(10), Fixed16::from(2) + Fixed16::from(8));
        assert_eq!(Fixed16::from(10), Fixed16::from(9) + Fixed16::from(1) / Fixed16::from(2) + Fixed16::from(1) / Fixed16::from(2));
        println!("{}", (Fixed16::from(6) / Fixed16::from(100)));
    }
}
