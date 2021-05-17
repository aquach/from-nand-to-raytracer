#[macro_use]
extern crate lazy_static;
extern crate image;

mod fixed;
mod int32;
mod plane;
mod ray;
mod sphere;
mod vector;

use crate::ray::Ray;
use image::{DynamicImage, GenericImage};
use plane::Plane;
use sphere::Sphere;
use std::convert::TryInto;
use vector::Vec3;

type Fixed = fixed::Q16_16;

pub struct Intersection<'a> {
    distance_from_camera: Fixed,
    object: &'a Box<dyn Element>,
}

pub trait Element: std::fmt::Debug {
    fn intersect(&self, ray: &Ray) -> Option<Fixed>;
    fn color(&self) -> Fixed;
}

pub trait Light: std::fmt::Debug {}

#[derive(Debug)]
pub struct Scene {
    pub width: i16,
    pub height: i16,
    pub fov: Fixed,
    pub elements: Vec<Box<dyn Element>>,
    pub lights: Vec<Box<dyn Light>>,
}

impl Scene {
    pub fn create_prime_ray(&self, pixel_x: i16, pixel_y: i16) -> Ray {
        assert!(self.width > self.height);

        let scene_width = Fixed::from(self.width);
        let scene_height = Fixed::from(self.height);

        let one = Fixed::from(1);
        let two = Fixed::from(2);

        let mut half = Fixed::from(1);
        half.do_div(&two);

        let mut fov_adjustment = self.fov;
        fov_adjustment.do_mul(&fixed::PI);
        fov_adjustment.do_div(&Fixed::from(180));
        fov_adjustment.do_div(&two);
        fov_adjustment.do_tan();

        let mut aspect_ratio = scene_width;
        aspect_ratio.do_div(&scene_height);

        // println!(
        //     "x: {} y: {} w: {} h: {} fov: {} aspect: {}",
        //     pixel_x, pixel_y, scene_width, scene_height, fov_adjustment, aspect_ratio
        // );

        let mut sensor_x = Fixed::from(pixel_x);
        sensor_x.do_add(&half);
        sensor_x.do_div(&scene_width);
        sensor_x.do_mul(&two);
        sensor_x.do_sub(&one);
        sensor_x.do_mul(&aspect_ratio);
        sensor_x.do_mul(&fov_adjustment);

        let mut sensor_y = Fixed::from(pixel_y);
        sensor_y.do_add(&half);
        sensor_y.do_div(&scene_height);
        sensor_y.do_neg();
        sensor_y.do_mul(&two);
        sensor_y.do_add(&one);
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

    pub fn trace(&self, ray: &Ray) -> Option<Intersection> {
        let mut intersection: Option<Intersection> = None;

        for elem in &self.elements {
            if let Some(d) = elem.intersect(ray) {
                if !intersection
                    .as_ref()
                    .filter(|i| i.distance_from_camera.is_less_than(&d))
                    .is_some()
                {
                    intersection = Some(Intersection {
                        distance_from_camera: d,
                        object: elem,
                    })
                }
            }
        }

        intersection
    }
}

pub fn gamma_correct(pixels: &mut Vec<Vec<Fixed>>) {}

pub fn dither(pixels: &mut Vec<Vec<Fixed>>) {}

pub fn render(scene: &Scene) -> Vec<Vec<Fixed>> {
    let black = Fixed::from(0);

    let mut pixels = vec![];

    for y in 0..scene.height {
        println!("Rendering row {}...", y);
        let mut row = vec![];
        for x in 0..scene.width {
            let ray = scene.create_prime_ray(x, y);

            let intersection = scene.trace(&ray);

            if let Some(i) = intersection {
                row.push(i.object.color());
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
        elements: vec![
            Box::new(Sphere {
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
            }),
            Box::new(Plane {
                origin: Vec3 {
                    x: Fixed::from(0),
                    y: Fixed::from(0),
                    z: Fixed::from(-25),
                },
                normal: Vec3 {
                    x: Fixed::from(0),
                    y: Fixed::from(0),
                    z: Fixed::from(-1),
                },
                color: {
                    let mut c = Fixed::from(95);
                    c.do_div(&Fixed::from(100));
                    c
                },
            }),
            Box::new(Plane {
                origin: Vec3 {
                    x: Fixed::from(0),
                    y: Fixed::from(-1),
                    z: Fixed::from(0),
                ,
                normal: Vec3 {
                    x: Fixed::from(0),
                    y: Fixed::from(-1),
                    z: Fixed::from(0),
                },
                color: {
                    let mut c = Fixed::from(70);
                    c.do_div(&Fixed::from(100));
                    c
                },
            }),
        ],
        lights: vec![],
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
