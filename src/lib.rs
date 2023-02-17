use image::RgbaImage;
use nutexb::NutexbFile;
use std::convert::TryFrom;
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub use cube::CubeLut3d;
pub use lut3d::Lut3dLinear;

mod color_correction;
mod cube;
mod interp;
mod lut3d;

pub use color_correction::correct_lut;

/// Convert an image with dimensions ((size * size), size) to a Nutexb LUT.
pub fn write_img_to_nutexb<P: AsRef<Path>>(
    img: &RgbaImage,
    path: &P,
) -> Result<(), Box<dyn std::error::Error>> {
    let linear = Lut3dLinear::try_from(img)?;
    write_lut_to_nutexb(&linear, path)
}

/// Convert a `Lut3dLinear` lut to Nutexb.
pub fn write_lut_to_nutexb<P: AsRef<Path>>(
    lut: &Lut3dLinear,
    path: P,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: This only works for size 16?
    NutexbFile::create(lut, "color_grading_lut")?.write_to_file(path)
}

/// Attempts to read the color grading LUT data from the given path.
/// The final LUT will not be valid if `nutexb` does contain a 16x16x16 RGBA 3D LUT texture.  
/// The conversion will fail if `nutexb` does not contain at least 16384 bytes of data.
pub fn read_nutexb_lut<P: AsRef<Path>>(path: P) -> Result<Lut3dLinear, Box<dyn Error>> {
    // TODO: Error if dimensions aren't supported?
    let nutexb = NutexbFile::read_from_file(path)?;
    Ok(Lut3dLinear::from_rgba(
        nutexb.footer.depth as usize,
        nutexb.deswizzled_data()?,
    ))
}

fn index3d(x: usize, y: usize, z: usize, width: usize, height: usize) -> usize {
    z * width * height + y * width + x
}

fn create_identity_lut_f32(size: usize) -> Vec<f32> {
    let channels = 4;

    let mut result = vec![0.0; size * size * size * channels];
    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let offset = index3d(x, y, z, size, size) * channels;
                result[offset] = x as f32 / (size - 1) as f32;
                result[offset + 1] = y as f32 / (size - 1) as f32;
                result[offset + 2] = z as f32 / (size - 1) as f32;
                result[offset + 3] = 1.0;
            }
        }
    }

    result
}

/// Create a 16x16x16 RGB LUT used as the default stage LUT.
/// This applies a subtle contrast/saturation adjustment.
pub fn create_default_lut() -> Vec<u8> {
    let gradient_values = [
        0u8, 15u8, 30u8, 46u8, 64u8, 82u8, 101u8, 121u8, 140u8, 158u8, 176u8, 193u8, 209u8, 224u8,
        240u8, 255u8,
    ];

    let bpp = 4;
    let width = 16;
    let height = 16;
    let depth = 16;

    let mut result = vec![0u8; width * height * depth * bpp];
    for z in 0..depth {
        for y in 0..height {
            for x in 0..width {
                let offset = index3d(x, y, z, 16, 16) * 4;
                result[offset] = gradient_values[x];
                result[offset + 1] = gradient_values[y];
                result[offset + 2] = gradient_values[z];
                result[offset + 3] = 255u8;
            }
        }
    }

    result
}

pub fn create_default_lut_f32() -> Vec<f32> {
    create_default_lut()
        .into_iter()
        .map(|u| u as f32 / 255.0)
        .collect()
}

/// Converts the data in `lut_linear` to the .cube format and writes it to `output`.
pub fn linear_lut_to_cube<P: AsRef<Path>>(
    lut_linear: &Lut3dLinear,
    output: P,
) -> Result<(), Box<dyn std::error::Error>> {
    let cube = CubeLut3d::from(lut_linear);
    let mut file = File::create(output)?;
    cube.write(&mut file)?;
    Ok(())
}
