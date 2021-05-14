extern crate image;
mod fixed;

use image::{DynamicImage, GenericImageView};
use fixed::Fixed16;

pub struct Point {
    pub x: Fixed16,
    pub y: Fixed16,
    pub z: Fixed16
}

pub struct Sphere {
    pub center: Point,
    pub radius: Fixed16,
    pub color: Fixed16,
}

pub struct Scene {
    pub width: u32,
    pub height: u32,
    pub fov: Fixed16,
    pub sphere: Sphere,
}

fn main() {
    let scene = Scene {
        width: 800,
        height: 600,
        fov: Fixed16::from(90),
        sphere: Sphere {
            center: Point {
                x: Fixed16::from(0),
                y: Fixed16::from(0),
                z: Fixed16::from(-5),
            },
            radius: Fixed16::from(1),
            color: Fixed16::from(6) / Fixed16::from(10),
        },
    };

    let img: DynamicImage = DynamicImage::new_rgb8(scene.width, scene.height);

    assert_eq!(scene.width, img.width());
    assert_eq!(scene.height, img.height());
}
