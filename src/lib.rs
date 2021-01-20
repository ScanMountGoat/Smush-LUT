use image::RgbaImage;
use std::path::Path;
use std::{
    convert::TryFrom,
    io::{Read, Write},
};
use std::{
    fs::{self, File},
    io::Cursor,
};

pub use self::cube::CubeLut3d;
pub use lut3d::{Lut3dLinear, Lut3dSwizzled};

mod cube;
mod lut3d;
mod swizzle;

// The dimensions and format are constant, so just include the footer.
static NUTEXB_FOOTER: &[u8] = include_bytes!("footer.bin");

/// Convert an image with dimensions ((size * size), size) to a Nutexb LUT.
pub fn img_to_nutexb<P: AsRef<Path>>(
    img: &RgbaImage,
    path: &P,
) -> Result<(), Box<dyn std::error::Error>> {
    let linear = Lut3dLinear::try_from(img)?;
    let swizzled: Lut3dSwizzled = linear.into();

    let mut buffer = File::create(path)?;
    buffer.write(swizzled.as_ref())?;
    buffer.write(NUTEXB_FOOTER)?;
    Ok(())
}

/// Convert a `Lut3dLinear` lut to Nutexb.
pub fn linear_lut_to_nutexb<P: AsRef<Path>>(
    lut: Lut3dLinear,
    path: &P,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: This only works for size 16.
    let swizzled: Lut3dSwizzled = lut.into();

    let mut buffer = File::create(path)?;
    buffer.write(swizzled.as_ref())?;
    buffer.write(NUTEXB_FOOTER)?;
    Ok(())
}

/// Attempts to read the color grading LUT data from the given path.
pub fn read_lut_from_nutexb<P: AsRef<Path>>(nutexb: P) -> Option<Lut3dLinear> {
    // TODO: Also parse the footer.
    // TODO: It may be better to add this functionality to the nutexb library once it's more finalized.

    // Read the swizzled image data.
    let mut file = Cursor::new(fs::read(nutexb).ok()?);
    let mut data = vec![0u8; image_size(16, 16, 16, 4)];
    file.read_exact(&mut data).ok()?;

    let swizzled = Lut3dSwizzled::new(16, data);
    Some(swizzled.into())
}

pub fn write_neutral_lut_to_img(img: &mut RgbaImage) {
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

fn create_neutral_lut() -> Vec<u8> {
    // Create a non swizzled 16x16x16 RGB LUT.
    let gradient_values = [
        0u8, 15u8, 30u8, 46u8, 64u8, 82u8, 101u8, 121u8, 140u8, 158u8, 176u8, 193u8, 209u8, 224u8,
        240u8, 255u8,
    ];

    let bpp = 4;
    let width = 16;
    let height = 16;
    let depth = 16;

    let mut result = vec![0u8; image_size(16, 16, 16, 4)];
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

pub fn linear_lut_to_cube(lut_linear: Lut3dLinear, output: &std::path::PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let cube = CubeLut3d::from(lut_linear);
    let mut file = File::create(output)?;
    cube.write(&mut file)?;
    Ok(())
}
