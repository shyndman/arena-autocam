use std::fs::DirBuilder;

use image::{GenericImage, GenericImageView};
use palette::Srgb;

use crate::example::{distance_to_target_color, is_close_to_target_color};

fn main() {
    let mut img = image::open("sample_data/green_thimble1.png").unwrap();
    let (w, h) = img.dimensions();

    let pixels = img.to_rgb32f();
    for x in 0..w {
        for y in 0..h {
            let [r, g, b] = pixels.get_pixel(x, y).0;
            let p = Srgb::new(r, g, b);

            if is_close_to_target_color(p) {
                println!(
                    "({:>3},{:>3}) r:{:.2},g:{:.2},b:{:.2} distance: {:?}",
                    x,
                    y,
                    r,
                    g,
                    b,
                    distance_to_target_color(p)
                );
                img.put_pixel(x, y, image::Rgba([0, 255, 0, 255]));
            }
        }
    }

    DirBuilder::new()
        .recursive(true)
        .create("target/debug/")
        .unwrap();
    img.save("target/debug/diff.png").unwrap();
}

#[allow(unused)]
mod example {
    use once_cell::sync::Lazy;
    use palette::rgb::Rgb;
    use palette::*;

    static MODE: DistanceMode = DistanceMode::Rgb;

    const TARGET_GREEN_RGB: Rgb = Rgb::new(
        0x49 as f32 / 255.0,
        0xa4 as f32 / 255.0,
        0x7e as f32 / 255.0,
    );

    static TARGET_GREEN_LAB: Lazy<Lab> = Lazy::new(|| TARGET_GREEN_RGB.into_color());

    const TARGET_SHADOW_GREEN_RGB: Rgb = Rgb::new(
        0x17 as f32 / 255.0,
        0x6c as f32 / 255.0,
        0x4c as f32 / 255.0,
    );

    static TARGET_SHADOW_GREEN_LAB: Lazy<Lab> =
        Lazy::new(|| TARGET_SHADOW_GREEN_RGB.into_color());

    #[derive(PartialEq)]
    enum DistanceMode {
        Rgb,
        Lab,
    }

    pub fn is_close_to_target_color(c: Srgb) -> bool {
        let ((d1, d2), threshold) = if MODE == DistanceMode::Lab {
            (distance_to_target_color(c), 10.0)
        } else {
            (distance_to_target_color(c), 0.15)
        };
        d1 < threshold || d2 < threshold
    }

    pub fn distance_to_target_color(c: Srgb) -> (f32, f32) {
        if MODE == DistanceMode::Lab {
            (
                perceptual_distance_from_target_color(
                    c,
                    once_cell::sync::Lazy::force(&TARGET_GREEN_LAB),
                ),
                perceptual_distance_from_target_color(
                    c,
                    once_cell::sync::Lazy::force(&TARGET_SHADOW_GREEN_LAB),
                ),
            )
        } else {
            (
                rgb_distance_from_target_color(c, &TARGET_GREEN_RGB),
                rgb_distance_from_target_color(c, &TARGET_SHADOW_GREEN_RGB),
            )
        }
    }

    fn perceptual_distance_from_target_color(c: Srgb, t: &Lab) -> f32 {
        t.get_color_difference(c.into_color())
    }

    fn rgb_distance_from_target_color(c: Srgb, t: &Rgb) -> f32 {
        ((t.red - c.red).powf(2.0) +
            (t.green - c.green).powf(2.0) +
            (t.blue - c.blue).powf(2.0))
        .sqrt()
    }
}
