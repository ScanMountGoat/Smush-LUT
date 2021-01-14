use image::RgbaImage;
use std::io::{Read, Write};
use std::path::Path;
use std::{
    fs::{self, File},
    io::Cursor,
};

// The dimensions and format are constant, so just include the footer.
static NUTEXB_FOOTER: &[u8] = include_bytes!("footer.bin");

pub fn write_nutexb<P: AsRef<Path> + ?Sized>(
    img: &RgbaImage,
    path: &P,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Find a better way to do this.
    let mut data = Vec::new();
    let width = 16;
    let height = 16;
    let depth = 16;
    for z in 0..depth {
        for y in 0..height {
            for x in 0..width {
                let image::Rgba([r, g, b, a]) = img.get_pixel((z * width) + x, y);
                data.push(*r);
                data.push(*g);
                data.push(*b);
                data.push(*a);
            }
        }
    }

    let mut swizzled_data = [0u8; image_size(16, 16, 16, 4)];
    swizzle(&data, &mut swizzled_data, false);

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
    swizzle(&swizzled, &mut deswizzled, true);
    RgbaImage::from_raw(256, 16, deswizzled.to_vec())
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

// TODO: Add the swizzle code to a lib.

// color_grading_lut.nutexb
// Every 4096 bytes is a quadrant of the 3D RGB volume.
// R [0,255], G [0,121], B [0,121]: 0 to 4096
// R [0,255], G [0,121], B [140,255]: 4096 to 8192
// R [0,255], G [140,255], B [0,121]: 8192 to 12288
// R [0,255], G [140,255], B [140,255]: 12288 to 16384

// Red (255,0,0,255) swizzled address: 300 (0000 0001 0010 1100)
// Green (0,255,0,255) swizzled address: 8400 (0010 0000 1101 0000)
// Blue (0,0,255,255) swizzled address: 7680 (0001 1110 0000 0000)
// White (255,255,255,255) swizzled address: 16380 (0011 1111 1111 1100)

fn swizzle(source: &[u8], destination: &mut [u8], deswizzle: bool) {
    // The bit masking trick to increment the offset is taken from here:
    // https://fgiesen.wordpress.com/2011/01/17/texture-tiling-and-swizzling/
    // The masks allow "skipping over" certain bits when incrementing.
    // The first row of the base layer, for example, has addresses
    // 0, 4, 8, 12, ..., 32, 36, 40, 44, ..., 256, 260, 264, 268, ..., 288, 292, 296, 300
    let x_mask = 0b0000_0001_0010_1100i32;
    let y_mask = 0b0010_0000_1101_0000i32;
    let z_mask = 0b0001_1110__0000_0000i32;

    let bpp = 4;
    let width = 16;
    let height = 16;
    let depth = 16;

    let mut offset_x = 0i32;
    let mut offset_y = 0i32;
    let mut offset_z = 0i32;

    // TODO: There's probably an error condition where this doesn't work.
    // TODO: Check for invalid offsets after swizzling.
    for z in 0..depth {
        for y in 0..height {
            for x in 0..width {
                // The bit patterns don't overlap, so just sum the offsets.
                // TODO: The offset calculations can be simplified since this is in a loop.
                let src = (offset_x + offset_y + offset_z) as usize;
                let dst = ((z * width * height) + (y * width) + x) * bpp;

                // Swap the offets for swizzling or deswizzling.
                // TODO: The condition doesn't need to be in the inner loop.
                if deswizzle {
                    (&mut destination[dst..dst + bpp]).copy_from_slice(&source[src..src + bpp]);
                } else {
                    (&mut destination[src..src + bpp]).copy_from_slice(&source[dst..dst + bpp]);
                }

                offset_x = (offset_x - x_mask) & x_mask;
            }
            offset_y = (offset_y - y_mask) & y_mask;
        }
        offset_z = (offset_z - z_mask) & z_mask;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swizzle_deswizzle() {
        // Make sure deswizzling and then swizzling again is 1:1.
        // This ensures textures will be saved correctly.
        let original = create_neutral_lut();
        let mut deswizzled = [0u8; image_size(16, 16, 16, 4)];
        swizzle(&original, &mut deswizzled, true);

        let mut reswizzled = [0u8; image_size(16, 16, 16, 4)];
        swizzle(&deswizzled, &mut reswizzled, false);

        let matching = original
            .iter()
            .zip(reswizzled.iter())
            .filter(|&(a, b)| a == b)
            .count();
        assert_eq!(matching, image_size(16, 16, 16, 4));
    }
}
