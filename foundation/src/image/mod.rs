mod color_regions;
mod connected_components;

pub use color_regions::*;
pub use connected_components::*;
use image::{ImageBuffer, Pixel};

pub type Image<P> = ImageBuffer<P, Vec<<P as Pixel>::Subpixel>>;
