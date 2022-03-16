use image::RgbaImage;
use nutexb::NutexbFile;
use std::convert::TryFrom;
use std::error::Error;
use std::fs::File;
use std::path::Path;

pub use self::cube::CubeLut3d;
pub use lut3d::Lut3dLinear;

mod cube;
mod interp;
mod lut3d;

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
    path: &P,
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
    Ok(Lut3dLinear::new(
        nutexb.footer.depth,
        nutexb.deswizzled_data()?,
    ))
}

const fn image_size(width: usize, height: usize, depth: usize, bpp: usize) -> usize {
    width * height * depth * bpp
}

/// Create a linear (not swizzled) 16x16x16 RGB LUT used as the default stage LUT.
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

/// Converts the data in `lut_linear` to the .cube format and writes it to `output`.
pub fn linear_lut_to_cube(
    lut_linear: &Lut3dLinear,
    output: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let cube = CubeLut3d::from(lut_linear);
    let mut file = File::create(output)?;
    cube.write(&mut file)?;
    Ok(())
}
