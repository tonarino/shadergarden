use std::{
    io::Cursor,
    path::Path,
};

use glium::texture::{
    RawImage2d,
    Texture2d,
};
use image::{
    ImageBuffer,
    ImageFormat,
    Rgba,
};

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

pub fn write_png(texture: &Texture2d, path: &Path) {
    let mut buffer =
        ImageBuffer::new(texture.width() as u32, texture.height() as u32);

    let sink: Vec<Vec<(u8, u8, u8, u8)>> = texture.read();

    for (y, row) in sink.iter().enumerate() {
        for (x, pixel) in row.iter().enumerate() {
            buffer.put_pixel(
                x as u32,
                y as u32,
                Rgba([pixel.0, pixel.1, pixel.2, pixel.3]),
            );
        }
    }

    buffer.save(&path).expect("Could not write frame");
    println!("Saved frame {}", path.display());
}
