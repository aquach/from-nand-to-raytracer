#[macro_use]
extern crate lazy_static;
extern crate image;

mod elements;
mod f64;
mod fixed;
mod int32;
mod lights;
mod ray;
mod vector;

use crate::ray::Ray;
use elements::plane::Plane;
use elements::sphere::Sphere;
use image::{DynamicImage, GenericImage};
use lights::directional::Directional;
use std::convert::TryInto;
use vector::Vec3;

const DO_DITHERING: bool = true;

// pub type Number = f64::Number;
// use crate::f64::PI;

pub type Number = fixed::Number;
use crate::fixed::PI;

#[derive(Debug)]
pub struct Intersection<'a> {
    distance_from_origin: Number,
    object: &'a Box<dyn Element>,
}

pub trait Element: std::fmt::Debug {
    fn intersect(&self, ray: &Ray) -> Option<Number>;
    fn color(&self, hit_point: &Vec3) -> Number;
    fn surface_normal(&self, hit_point: &Vec3) -> Vec3;
}

#[derive(Debug)]
pub struct Scene {
    pub width: i16,
    pub height: i16,
    pub fov: Number,
    pub elements: Vec<Box<dyn Element>>,
    pub lights: Vec<Directional>,
}

impl Scene {
    pub fn create_prime_ray(&self, pixel_x: i16, pixel_y: i16) -> Ray {
        assert!(self.width > self.height);

        let scene_width = Number::from(self.width);
        let scene_height = Number::from(self.height);

        let one = Number::from(1);
        let two = Number::from(2);

        let mut half = Number::from(1);
        half.do_div(&two);

        let mut fov_adjustment = self.fov;
        fov_adjustment.do_mul(&PI);
        fov_adjustment.do_div(&Number::from(180));
        fov_adjustment.do_div(&two);
        fov_adjustment.do_tan();

        let mut aspect_ratio = scene_width;
        aspect_ratio.do_div(&scene_height);

        // println!(
        //     "x: {} y: {} w: {} h: {} fov: {} aspect: {}",
        //     pixel_x, pixel_y, scene_width, scene_height, fov_adjustment, aspect_ratio
        // );

        let mut sensor_x = Number::from(pixel_x);
        sensor_x.do_add(&half);
        sensor_x.do_div(&scene_width);
        sensor_x.do_mul(&two);
        sensor_x.do_sub(&one);
        sensor_x.do_mul(&aspect_ratio);
        sensor_x.do_mul(&fov_adjustment);

        let mut sensor_y = Number::from(pixel_y);
        sensor_y.do_add(&half);
        sensor_y.do_div(&scene_height);
        sensor_y.do_neg();
        sensor_y.do_mul(&two);
        sensor_y.do_add(&one);
        sensor_y.do_mul(&fov_adjustment);

        let mut direction = Vec3 {
            x: sensor_x,
            y: sensor_y,
            z: Number::from(-1),
        };
        direction.do_normalize();

        Ray {
            origin: Vec3 {
                x: Number::from(0),
                y: Number::from(0),
                z: Number::from(0),
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
                    .filter(|i| i.distance_from_origin.is_less_than(&d))
                    .is_some()
                {
                    intersection = Some(Intersection {
                        distance_from_origin: d,
                        object: elem,
                    })
                }
            }
        }

        intersection
    }
}

pub fn render(scene: &Scene) -> Vec<Vec<Number>> {
    let black = Number::from(0);

    let mut pixels = vec![];
    pixels.resize_with(scene.height.try_into().unwrap(), || {
        let mut row = vec![];
        row.resize(scene.width.try_into().unwrap(), Number::from(0));
        row
    });

    let mut dither_pixels = vec![];
    dither_pixels.resize_with(scene.height.try_into().unwrap(), || {
        let mut row = vec![];
        row.resize(scene.width.try_into().unwrap(), Number::from(0));
        row
    });

    for y in 0..scene.height {
        for x in 0..scene.width {
            let ray = scene.create_prime_ray(x, y);

            let intersection = scene.trace(&ray);

            let color = if let Some(i) = intersection {
                let mut hit_point = ray.origin;
                let mut offset = ray.direction;
                offset.do_scale(&i.distance_from_origin);
                hit_point.do_add(&offset);

                let surface_normal = i.object.surface_normal(&hit_point);

                let mut color = Number::from(0);

                for light in &scene.lights {
                    let mut direction_to_light = light.direction;
                    direction_to_light.do_scale(&Number::from(-1));

                    let mut shadow_bias = direction_to_light;
                    let mut epsilon = Number::from(1);
                    epsilon.do_div(&Number::from(20));
                    shadow_bias.do_scale(&epsilon);

                    let mut origin = hit_point;
                    origin.do_add(&shadow_bias);

                    let shadow_ray = Ray {
                        origin: origin,
                        direction: direction_to_light,
                    };
                    let in_light = scene.trace(&shadow_ray).is_none();

                    if in_light {
                        let mut light_power = surface_normal.dot(&direction_to_light);
                        if light_power.is_negative() {
                            light_power = Number::from(0);
                        }

                        let mut added_color = light.color;
                        added_color.do_mul(&light_power);
                        added_color.do_div(&PI);
                        added_color.do_mul(&i.object.color(&hit_point));

                        color.do_add(&added_color);
                    }
                }

                color
            } else {
                black
            };

            let xi: usize = x.try_into().unwrap();
            let yi: usize = y.try_into().unwrap();

            pixels[yi][xi].do_add(&color);

            // Poor math's gamma correction :sob: sqrt() is much easier than 1/2.2
            pixels[yi][xi].do_sqrt();

            // Bring in the value from the dithering. We want to dither _after_ gamma correction.
            // Dithered pixels represent gamma-encoded values already.
            pixels[yi][xi].do_add(&dither_pixels[yi][xi]);

            // Perform dithering.
            if DO_DITHERING {
                let mut half = Number::from(1);
                half.do_div(&Number::from(2));

                let is_white = pixels[yi][xi].cmp(&half) >= 0;
                let new_color = if is_white {
                    Number::from(1)
                } else {
                    Number::from(0)
                };
                let mut quant_error = pixels[yi][xi];
                quant_error.do_sub(&new_color);
                quant_error.do_div(&Number::from(16));

                if x + 1 < scene.width {
                    let mut quant_error_7 = quant_error;
                    quant_error_7.do_mul(&Number::from(7));
                    dither_pixels[yi][xi + 1].do_add(&quant_error_7);
                }

                if y + 1 < scene.height {
                    if x >= 1 {
                        let mut quant_error_3 = quant_error;
                        quant_error_3.do_mul(&Number::from(3));
                        dither_pixels[yi + 1][xi - 1].do_add(&quant_error_3);
                    }

                    let mut quant_error_5 = quant_error;
                    quant_error_5.do_mul(&Number::from(5));
                    dither_pixels[yi + 1][xi].do_add(&quant_error_5);

                    if x + 1 < scene.width {
                        dither_pixels[yi + 1][xi + 1].do_add(&quant_error);
                    }
                }

                pixels[yi][xi] = new_color;
            }
        }
        eprint!(".");
    }

    pixels
}

pub fn to_rgba(color: &Number) -> image::Rgba<u8> {
    let c = (color.to_f64() * 255f64).round() as u8;
    image::Rgba([c, c, c, 255])
}

fn main() {
    let scene = Scene {
        width: 512,
        height: 256,
        fov: Number::from(90),
        elements: vec![
            Box::new(Sphere {
                center: Vec3 {
                    x: Number::from(-6),
                    y: {
                        let mut r = Number::from(-1);
                        r.do_div(&Number::from(2));
                        r
                    },
                    z: Number::from(-5),
                },
                radius: {
                    let mut r = Number::from(3);
                    r.do_div(&Number::from(2));
                    r
                },
                color: {
                    let mut c = Number::from(8);
                    c.do_div(&Number::from(10));
                    c
                },
            }),
            Box::new(Sphere {
                center: Vec3 {
                    x: Number::from(-1),
                    y: Number::from(-1),
                    z: Number::from(-5),
                },
                radius: Number::from(1),
                color: {
                    let mut c = Number::from(6);
                    c.do_div(&Number::from(10));
                    c
                },
            }),
            Box::new(Sphere {
                center: Vec3 {
                    x: Number::from(2),
                    y: Number::from(0),
                    z: Number::from(-3),
                },
                radius: Number::from(2),
                color: {
                    let mut c = Number::from(10);
                    c.do_div(&Number::from(10));
                    c
                },
            }),
            Box::new(Plane {
                origin: Vec3 {
                    x: Number::from(0),
                    y: Number::from(0),
                    z: Number::from(-25),
                },
                normal: Vec3 {
                    x: Number::from(0),
                    y: Number::from(0),
                    z: Number::from(-1),
                },
                color: Number::from(1),
                checkerboarded: false,
            }),
            Box::new(Plane {
                origin: Vec3 {
                    x: Number::from(0),
                    y: Number::from(-2),
                    z: Number::from(0),
                },
                normal: Vec3 {
                    x: Number::from(0),
                    y: Number::from(-1),
                    z: Number::from(0),
                },
                color: Number::from(-1),
                checkerboarded: true,
            }),
        ],
        lights: vec![
            Directional {
                direction: {
                    let mut v = Vec3 {
                        x: Number::from(0),
                        y: Number::from(-1),
                        z: Number::from(-1),
                    };
                    v.do_normalize();
                    v
                },
                color: {
                    let mut c = Number::from(5);
                    c.do_div(&Number::from(100));
                    c
                },
            },
            Directional {
                direction: {
                    let mut v = Vec3 {
                        x: Number::from(-1),
                        y: Number::from(-1),
                        z: Number::from(0),
                    };
                    v.do_normalize();
                    v
                },
                color: {
                    let mut c = Number::from(50);
                    c.do_div(&Number::from(100));
                    c
                },
            },
            Directional {
                direction: {
                    let mut v = Vec3 {
                        x: {
                            let mut v = Number::from(1);
                            v.do_div(&Number::from(2));
                            v
                        },
                        y: Number::from(-1),
                        z: Number::from(0),
                    };
                    v.do_normalize();
                    v
                },
                color: Number::from(1),
            },
        ],
    };

    let pixels = render(&scene);

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
