use image::{Rgba, RgbaImage};
use std::io::{Read, Write};
use std::path::Path;
use std::{
    fs::{self, File},
    io::Cursor,
};
use image::GenericImageView;

// The dimensions and format are constant, so just include the footer.
static NUTEXB_FOOTER: &[u8] = include_bytes!("footer.bin");

mod swizzle;

pub fn write_nutexb<P: AsRef<Path> + ?Sized>(
    img: &RgbaImage,
    path: &P,
) -> Result<(), Box<dyn std::error::Error>> {
    let data = read_lut_from_image(img);
    let mut swizzled_data = [0u8; image_size(16, 16, 16, 4)];
    swizzle::swizzle(&data, &mut swizzled_data, false);

    let mut buffer = File::create(path)?;
    buffer.write(&swizzled_data)?;
    buffer.write(NUTEXB_FOOTER)?;
    Ok(())
}

/// Attempts to read the color grading LUT data from the given path.
pub fn read_lut<P: AsRef<Path>>(path: P) -> Option<RgbaImage> {
    // TODO: Also parse the footer.
    // TODO: It may be better to add this functionality to the nutexb library once it's more finalized.

    // Read the swizzled image data.
    let mut file = Cursor::new(fs::read(path).ok()?);
    let mut swizzled = [0u8; image_size(16, 16, 16, 4)];
    file.read_exact(&mut swizzled).ok()?;

    // Deswizzle and store into an RGBA buffer.
    let mut deswizzled = [0u8; image_size(16, 16, 16, 4)];
    swizzle::swizzle(&swizzled, &mut deswizzled, true);
    RgbaImage::from_raw(256, 16, deswizzled.to_vec())
}

fn read_lut_from_image(img: &RgbaImage) -> Vec<u8> {
    let mut data = Vec::new();
    for (_,_, Rgba(bytes)) in img.view(0, 0, 256, 16).pixels() {
        data.extend_from_slice(&bytes);
    }
    data
}

pub fn write_lut_to_img(img: &mut RgbaImage) {
    // Undo the postprocessing to ensure the gradient steps refer to the proper colors.
    let darken = |u: u8| (u as f32 / 1.4f32) as u8;
    for pixel in img.pixels_mut() {
        *pixel = image::Rgba([
            darken(pixel[0]),
            darken(pixel[1]),
            darken(pixel[2]),
            pixel[3],
        ]);
    }

    let neutral_lut_linear = create_neutral_lut();

    // TODO: Find a cleaner way to write this.
    let bpp = 4;
    for z in 0..16 {
        for y in 0..16 {
            for x in 0..16 {
                let offset = ((z * 16 * 16) + (y * 16) + x) * bpp;
                if let [r, g, b, a] = &neutral_lut_linear[offset..offset + 4] {
                    let pixel = image::Rgba([*r, *g, *b, *a]);
                    img.put_pixel((x + (z * 16)) as u32, y as u32, pixel);
                }
            }
        }
    }
}

const fn image_size(width: usize, height: usize, depth: usize, bpp: usize) -> usize {
    width * height * depth * bpp
}

fn create_neutral_lut() -> [u8; image_size(16, 16, 16, 4)] {
    // Create a non swizzled 16x16x16 RGB LUT.
    let gradient_values = [
        0u8, 15u8, 30u8, 46u8, 64u8, 82u8, 101u8, 121u8, 140u8, 158u8, 176u8, 193u8, 209u8, 224u8,
        240u8, 255u8,
    ];

    let bpp = 4;
    let width = 16;
    let height = 16;
    let depth = 16;

    let mut result = [0u8; image_size(16, 16, 16, 4)];
    for z in 0..depth {
        for y in 0..height {
            for x in 0..width {
                let offset = ((z * width * height) + (y * width) + x) * bpp;
                result[offset] = gradient_values[x];
                result[offset + 1] = gradient_values[y];
                result[offset + 2] = gradient_values[z];
                result[offset + 3] = 255u8;
            }
        }
    }
    result
}

