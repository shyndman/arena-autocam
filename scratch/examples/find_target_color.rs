use std::collections::HashMap;
use std::time::Instant;

use image::{GenericImageView, GrayImage, Luma};
use imageproc::region_labelling::{connected_components, Connectivity};
use palette::Srgb;

use crate::example::is_close_to_target_color;

const IMAGE_WIDTH: u32 = 300;
const IMAGE_HEIGHT: u32 = 300;

fn main() {
    let img = image::open("sample_data/green_thimble2.png").unwrap();
    let (w, h) = img.dimensions();
    assert!(w == IMAGE_WIDTH && h == IMAGE_HEIGHT);

    let mut detections = GrayImage::from_raw(
        IMAGE_WIDTH,
        IMAGE_HEIGHT,
        vec![0; (IMAGE_WIDTH * IMAGE_HEIGHT) as usize],
    )
    .unwrap();

    let start_ts = Instant::now();
    let pixels = img.to_rgb32f();

    for x in 0..w {
        for y in 0..h {
            let [r, g, b] = pixels.get_pixel(x, y).0;
            let p = Srgb::new(r, g, b);

            if is_close_to_target_color(p) {
                detections.put_pixel(x, y, Luma([255]));
            }
        }
    }

    let background_color = Luma([0u8]);
    let component_image =
        connected_components(&detections, Connectivity::Eight, background_color);
    let mut components = HashMap::new();
    for x in 0..IMAGE_WIDTH {
        for y in 0..IMAGE_HEIGHT {
            let p = component_image.get_pixel(x, y);
            let comp_id = p.0[0];
            if comp_id != 0 {
                (&mut components)
                    .entry(comp_id)
                    .and_modify(|c: &mut Component| c.add_pixel((x, y)))
                    .or_insert(Component::from_pixel(comp_id, (x, y)));
            }
        }
    }

    let mut target_components: Vec<&Component> =
        components.values().filter(|c| c.count > 30).collect();
    target_components.sort();

    println!("Done in {}ms", start_ts.elapsed().as_millis());
    // println!("Component counts:\n{:#?}", components);
    println!("Target components:\n{:#?}", target_components);

    // DirBuilder::new()
    //     .recursive(true)
    //     .create("target/debug/")
    //     .unwrap();
    // detections.save("target/debug/detections.png").unwrap();
    // components.save("target/debug/components.png").unwrap();
}

#[derive(Debug, Eq, PartialEq)]
struct Component {
    id: u32,
    top: u32,
    right: u32,
    bottom: u32,
    left: u32,
    count: u32,
}
impl Component {
    fn from_pixel(id: u32, (x, y): (u32, u32)) -> Self {
        Self {
            id,
            top: y,
            right: x,
            bottom: y,
            left: x,
            count: 1,
        }
    }

    fn add_pixel(&mut self, (x, y): (u32, u32)) {
        if x < self.left {
            self.left = x;
        } else if x > self.right {
            self.right = x;
        }
        if y < self.top {
            self.top = y;
        } else if y > self.bottom {
            self.bottom = y;
        }
        self.count += 1;
    }
}

impl PartialOrd for &Component {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for &Component {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.count.cmp(&self.count)
    }
}

#[allow(unused)]
mod example {
    use once_cell::sync::Lazy;
    use palette::rgb::Rgb;
    use palette::*;

    static MODE: DistanceMode = DistanceMode::Lab;

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
