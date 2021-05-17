use crate::Element;
use crate::Ray;
use crate::Vec3;

type Fixed = crate::fixed::Q16_16;

#[derive(Debug, Clone, Copy)]
// One-sided plane.
pub struct Plane {
    pub origin: Vec3,
    pub normal: Vec3,
    pub color: Fixed,
}

impl Element for Plane {
    fn intersect(&self, ray: &Ray) -> Option<Fixed> {
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

    fn color(&self) -> Fixed {
        self.color
    }
}

#[cfg(test)]
mod test {
    use crate::vector::Vec3;
    use crate::Fixed;
    use crate::Ray;
    use crate::Element;
    use super::Plane;

    #[test]
    fn test_plane_intersect() {
        let plane = Plane {
            origin: Vec3 {
                x: Fixed::from(0),
                y: Fixed::from(0),
                z: Fixed::from(-5),
            },
            normal: Vec3 {
                x: Fixed::from(0),
                y: Fixed::from(0),
                z: Fixed::from(-1),
            },
            color: Fixed::from(0),
        };

        let ray = Ray {
            origin: Vec3 {
                x: Fixed::from(0),
                y: Fixed::from(0),
                z: Fixed::from(0),
            },
            direction: Vec3 {
                x: Fixed::from(0),
                y: Fixed::from(0),
                z: Fixed::from(-1),
            },
        };

        assert!(plane
            .intersect(&ray)
            .filter(|d| d.cmp(&Fixed::from(5)) == 0)
            .is_some());
    }
}
