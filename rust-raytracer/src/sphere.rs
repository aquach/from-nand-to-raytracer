use crate::Element;
use crate::Ray;
use crate::Vec3;

type Fixed = crate::fixed::Q16_16;

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: Fixed,
    pub color: Fixed,
}

impl Element for Sphere {
    fn intersect(&self, ray: &Ray) -> Option<Fixed> {
        let mut to_center = self.center;
        to_center.do_sub(&ray.origin);

        let projected_onto_ray_dist = to_center.dot(&ray.direction);
        let mut projected_onto_ray_dist_sq = projected_onto_ray_dist;
        projected_onto_ray_dist_sq.do_mul(&projected_onto_ray_dist);

        let mut radius_sq = self.radius;
        radius_sq.do_mul(&self.radius);

        let mut opposite_sq = to_center.dist_sq();
        opposite_sq.do_sub(&projected_onto_ray_dist_sq);

        if opposite_sq.cmp(&radius_sq) > 0 {
            return None;
        }

        let mut thickness = radius_sq;
        thickness.do_sub(&opposite_sq);
        thickness.do_sqrt();

        let mut t0 = projected_onto_ray_dist;
        t0.do_sub(&thickness);

        let mut t1 = projected_onto_ray_dist;
        t1.do_add(&thickness);

        if t0.is_negative() && t1.is_negative() {
            return None;
        }

        Some(if t0.is_less_than(&t1) { t0 } else { t1 })
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
    use super::Sphere;

    #[test]
    fn test_sphere_intersect() {
        let sphere = Sphere {
            center: Vec3 {
                x: Fixed::from(0),
                y: Fixed::from(0),
                z: Fixed::from(-5),
            },
            radius: Fixed::from(2),
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

        assert!(sphere
            .intersect(&ray)
            .filter(|d| d.cmp(&Fixed::from(3)) == 0)
            .is_some()
            );
    }
}
