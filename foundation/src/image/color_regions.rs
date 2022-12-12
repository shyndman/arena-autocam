use std::collections::HashMap;

use anyhow::{anyhow, Result};
use image::{GrayImage, Luma, RgbImage};
use palette::{ColorDifference, IntoColor, Lab, LinSrgb, Srgb};

use super::{connected_components, Connectivity};

const BACKGROUND_COLOR: Luma<u8> = Luma([0u8]);

pub fn find_similar_color_regions(image: &RgbImage, colors: &[&Lab]) -> Result<Vec<Region>> {
    let (w, h) = image.dimensions();
    let mut detections = GrayImage::from_raw(w, h, vec![0; (w * h) as usize])
        .ok_or(anyhow!("Unable to construct image"))?;

    // Check every pixel to see if they're close to any of the provided colors
    for x in 0..w {
        for y in 0..h {
            let [r, g, b] = image.get_pixel(x, y).0;
            let p = Srgb::new(r, g, b).into_linear();
            for c in colors {
                if is_close_to(p, c) {
                    detections.put_pixel(x, y, Luma([255]));
                }
            }
        }
    }

    let component_image =
        connected_components(&detections, Connectivity::Eight, BACKGROUND_COLOR);
    let mut components = HashMap::new();
    for x in 0..w {
        for y in 0..h {
            let p = component_image.get_pixel(x, y);
            let comp_id = p.0[0];
            if comp_id != 0 {
                (&mut components)
                    .entry(comp_id)
                    .and_modify(|c: &mut Region| c.add_pixel((x, y)))
                    .or_insert(Region::from_first_pixel(comp_id, (x, y)));
            }
        }
    }

    let target_components: Vec<Region> = {
        let mut t: Vec<Region> = components.values().map(|c| c.clone()).collect();
        t.sort();
        t
    };

    Ok(target_components)
}

fn is_close_to(c: LinSrgb, target: &Lab) -> bool {
    target.get_color_difference(c.into_color()) < 12.0
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Region {
    id: u32,
    top: u32,
    right: u32,
    bottom: u32,
    left: u32,
    pub count: u32,
}
impl Region {
    fn from_first_pixel(id: u32, (x, y): (u32, u32)) -> Self {
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

    pub fn x(&self) -> u32 {
        self.left
    }

    pub fn y(&self) -> u32 {
        self.top
    }

    pub fn width(&self) -> u32 {
        self.right - self.left
    }

    pub fn height(&self) -> u32 {
        self.bottom - self.top
    }
}

impl PartialOrd for Region {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Region {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        other.count.cmp(&self.count)
    }
}
