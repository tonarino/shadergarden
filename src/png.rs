use std::{
    io::Cursor,
    path::Path,
};

use glium::texture::RawImage2d;
use image::ImageFormat;

/// Pretty neat macro right here. Takes an image path, loads
/// it, and converts it to a raw_image.
#[macro_export]
macro_rules! include_png {
    ($path:literal) => {{
        let bytes = include_bytes!($path);
        crate::png::image_from_bytes(bytes)
    }};
}

pub fn image_from_bytes<'a>(bytes: Vec<u8>) -> RawImage2d<'a, u8> {
    let cursor = Cursor::new(&bytes);
    let image = image::load(cursor, ImageFormat::Png).unwrap().to_rgba8();
    let image_dimensions = image.dimensions();

    RawImage2d::from_raw_rgba_reversed(&image.into_raw(), image_dimensions)
}

pub fn load_png(path: &Path) -> RawImage2d<u8> {
    let bytes = std::fs::read(path).expect("Could not read input image");
    image_from_bytes(bytes)
}
