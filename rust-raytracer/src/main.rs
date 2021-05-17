#[macro_use]
extern crate lazy_static;
extern crate image;

mod fixed;
mod int32;
mod vector;

use image::{DynamicImage, GenericImage};
use std::convert::TryInto;
use std::fmt;
use vector::Vec3;

type Fixed = fixed::Q16_16;

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: Fixed,
    pub color: Fixed,
}

pub trait Intersectable {
    fn intersect(&self, ray: &Ray) -> bool;
}

impl Intersectable for Sphere {
    fn intersect(&self, ray: &Ray) -> bool {
        let mut to_center = self.center;
        to_center.do_sub(&ray.origin);

        let mut projected_onto_ray_dist_sq = to_center.dot(&ray.direction);
        let d = projected_onto_ray_dist_sq;
        projected_onto_ray_dist_sq.do_mul(&d);

        let mut radius_sq = self.radius;
        radius_sq.do_mul(&self.radius);

        to_center.dist_sq().is_less_than(&radius_sq)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Scene {
    pub width: i16,
    pub height: i16,
    pub fov: Fixed,
    pub sphere: Sphere,
}

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl fmt::Display for Ray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ray({} -> {})", self.origin, self.direction,)
    }
}

pub fn create_prime_ray(pixel_x: i16, pixel_y: i16, scene: &Scene) -> Ray {
    assert!(scene.width > scene.height);

    let scene_width = Fixed::from(scene.width);
    let scene_height = Fixed::from(scene.height);

    let one = Fixed::from(1);
    let two = Fixed::from(2);

    let mut half = Fixed::from(1);
    half.do_div(&two);

    let mut fov_adjustment = scene.fov;
    fov_adjustment.do_mul(&fixed::PI);
    fov_adjustment.do_div(&Fixed::from(180));
    fov_adjustment.do_div(&two);
    println!("{}", fov_adjustment);
    fov_adjustment.do_tan();
    println!("{}", fov_adjustment);

    let mut aspect_ratio = scene_width;
    aspect_ratio.do_div(&scene_height);

    println!(
        "x: {} y: {} w: {} h: {} fov: {} aspect: {}",
        pixel_x, pixel_y, scene_width, scene_height, fov_adjustment, aspect_ratio
    );

    let mut sensor_x = Fixed::from(pixel_x);
    println!("{}", sensor_x);
    sensor_x.do_add(&half);
    println!("{}", sensor_x);
    sensor_x.do_div(&scene_width);
    println!("{}", sensor_x);
    sensor_x.do_mul(&two);
    println!("{}", sensor_x);
    sensor_x.do_sub(&one);
    println!("{}", sensor_x);
    sensor_x.do_mul(&aspect_ratio);
    println!("{}", sensor_x);
    sensor_x.do_mul(&fov_adjustment);
    println!("{}", sensor_x);

    let mut sensor_y = Fixed::from(pixel_y);
    sensor_y.do_add(&half);
    sensor_y.do_div(&scene_height);
    sensor_y.do_neg();
    sensor_y.do_add(&one);
    sensor_y.do_mul(&two);
    sensor_y.do_mul(&fov_adjustment);

    let mut direction = Vec3 {
        x: sensor_x,
        y: sensor_y,
        z: Fixed::from(-1),
    };
    direction.do_normalize();

    Ray {
        origin: Vec3 {
            x: Fixed::from(0),
            y: Fixed::from(0),
            z: Fixed::from(0),
        },
        direction: direction,
    }
}

pub fn gamma_correct(pixels: &mut Vec<Vec<Fixed>>) {}

pub fn dither(pixels: &mut Vec<Vec<Fixed>>) {}

pub fn render(scene: &Scene) -> Vec<Vec<Fixed>> {
    let black = Fixed::from(0);

    let mut pixels = vec![];

    for y in 0..scene.height {
        let mut row = vec![];
        for x in 0..scene.width {
            let ray = create_prime_ray(x, y, scene);
            println!("{} {} => {}", x, y, ray);

            if scene.sphere.intersect(&ray) {
                row.push(scene.sphere.color);
            } else {
                row.push(black);
            }
        }

        pixels.push(row);
    }

    pixels
}

pub fn to_rgba(color: &Fixed) -> image::Rgba<u8> {
    let c = (color.to_f64() * 255f64).round() as u8;
    image::Rgba([c, c, c, 255])
}

fn main() {
    let scene = Scene {
        width: 512,
        height: 256,
        fov: Fixed::from(90),
        sphere: Sphere {
            center: Vec3 {
                x: Fixed::from(0),
                y: Fixed::from(0),
                z: Fixed::from(-5),
            },
            radius: Fixed::from(1),
            color: {
                let mut c = Fixed::from(6);
                c.do_div(&Fixed::from(10));
                c
            },
        },
    };

    let mut pixels = render(&scene);
    gamma_correct(&mut pixels);
    dither(&mut pixels);

    let mut image = DynamicImage::new_rgb8(
        scene.width.try_into().unwrap(),
        scene.height.try_into().unwrap(),
    );
    for (y, row) in pixels.iter().enumerate() {
        for (x, color) in row.iter().enumerate() {
            image.put_pixel(
                x.try_into().unwrap(),
                y.try_into().unwrap(),
                to_rgba(&color),
            );
        }
    }

    image.save("render.png").unwrap();
}
