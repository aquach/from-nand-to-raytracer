use crate::Element;
use crate::int32::Int32;
use crate::Number;
use crate::Ray;
use crate::Vec3;

#[derive(Debug, Clone, Copy)]
// One-sided plane.
pub struct Plane {
    pub origin: Vec3,
    pub normal: Vec3,
    pub color: Number,
    pub checkerboarded: bool,
}

impl Element for Plane {
    fn intersect(&self, ray: &Ray) -> Option<Number> {
        let denom = self.normal.dot(&ray.direction);

        if !denom.is_positive() {
            return None;
        }

        let mut ray_to_origin = self.origin;
        ray_to_origin.do_sub(&ray.origin);

        let mut distance = ray_to_origin.dot(&self.normal);
        distance.do_div(&denom);

        if !distance.is_negative() {
            Some(distance)
        } else {
            None
        }
    }

    fn color(&self, hit_point: &Vec3) -> Number {
        if self.checkerboarded {
            let mut x_axis = self.normal;
            x_axis.do_cross(&Vec3 {
                x: Number::from(0),
                y: Number::from(0),
                z: Number::from(1),
            });
            if x_axis.dist_sq().is_zero() {
                x_axis.do_cross(&Vec3 {
                    x: Number::from(0),
                    y: Number::from(1),
                    z: Number::from(0),
                });
            }

            let mut y_axis = self.normal;
            y_axis.do_cross(&x_axis);

            let SCALE = Number::from(1);

            let x = {
                let mut v = hit_point.dot(&x_axis);
                v.do_mul(&SCALE);
                v.do_add(&Number::from(1000));
                v.to_int32()
            };
            let y = {
                let mut v = hit_point.dot(&y_axis);
                v.do_mul(&SCALE);
                v.do_add(&Number::from(1000));
                v.to_int32()
            };

            let mut sum = x;
            sum.do_add(&y);

            let mut halftwice = sum;
            halftwice.do_div(&Int32::from(2));
            halftwice.do_mul(&Int32::from(2));

            if sum.cmp(&halftwice) == 0 {
                let mut n = Number::from(90);
                n.do_div(&Number::from(100));
                n
            } else {
                let mut n = Number::from(3);
                n.do_div(&Number::from(100));
                n
            }
        } else {
            self.color
        }
    }

    fn surface_normal(&self, _: &Vec3) -> Vec3 {
        let mut c = self.normal;
        c.do_scale(&Number::from(-1));
        c
    }
}

#[cfg(test)]
mod test {
    use super::Plane;
    use crate::vector::Vec3;
    use crate::Element;
    use crate::Number;
    use crate::Ray;

    #[test]
    fn test_plane_intersect() {
        let plane = Plane {
            origin: Vec3 {
                x: Number::from(0),
                y: Number::from(0),
                z: Number::from(-5),
            },
            normal: Vec3 {
                x: Number::from(0),
                y: Number::from(0),
                z: Number::from(-1),
            },
            color: Number::from(0),
            checkerboarded: false
        };

        let ray = Ray {
            origin: Vec3 {
                x: Number::from(0),
                y: Number::from(0),
                z: Number::from(0),
            },
            direction: Vec3 {
                x: Number::from(0),
                y: Number::from(0),
                z: Number::from(-1),
            },
        };

        assert!(plane
            .intersect(&ray)
            .filter(|d| d.cmp(&Number::from(5)) == 0)
            .is_some());
    }
}
