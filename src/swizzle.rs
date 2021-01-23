// TODO: Add the swizzle code to a lib.

// color_grading_lut.nutexb
// Every 4096 bytes is a quadrant of the 3D RGB volume.
// R [0,255], G [0,121], B [0,121]: 0 to 4096
// R [0,255], G [0,121], B [140,255]: 4096 to 8192
// R [0,255], G [140,255], B [0,121]: 8192 to 12288
// R [0,255], G [140,255], B [140,255]: 12288 to 16384

pub fn swizzle(source: &[u8], destination: &mut [u8], deswizzle: bool) {
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

                // Use the following 2's complement identity:
                // offset + !mask + 1 = offset - mask
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
    fn swizzle_primaries() {
        let data = crate::create_default_lut();
        let mut swizzled = vec![0u8; crate::image_size(16, 16, 16, 4)];
        swizzle(&data, &mut swizzled, false);

        // Check primary colors to test that the XYZ masks are correct.
        // Black swizzled address: 0 (0000 0000 0000 0000)
        assert_eq!(&[0u8, 0u8, 0u8, 255u8], &swizzled[0..4]);

        // Red swizzled address: 300 (0000 0001 0010 1100)
        assert_eq!(&[255u8, 0u8, 0u8, 255u8], &swizzled[300..304]);

        // Green swizzled address: 8400 (0010 0000 1101 0000)
        assert_eq!(&[0u8, 255u8, 0u8, 255u8], &swizzled[8400..8404]);

        // Blue swizzled address: 7680 (0001 1110 0000 0000)
        assert_eq!(&[0u8, 0u8, 255u8, 255u8], &swizzled[7680..7684]);
    }

    #[test]
    fn swizzle_first_row() {
        let data = crate::create_default_lut();
        let mut swizzled = [0u8; crate::image_size(16, 16, 16, 4)];
        swizzle(&data, &mut swizzled, false);

        // Test the increasing R values for the 16 pixels of the first row.
        assert_eq!(&[0u8, 0u8, 0u8, 255u8], &swizzled[0..4]);
        assert_eq!(&[15u8, 0u8, 0u8, 255u8], &swizzled[4..8]);
        assert_eq!(&[30u8, 0u8, 0u8, 255u8], &swizzled[8..12]);
        assert_eq!(&[46u8, 0u8, 0u8, 255u8], &swizzled[12..16]);

        assert_eq!(&[64u8, 0u8, 0u8, 255u8], &swizzled[32..36]);
        assert_eq!(&[82u8, 0u8, 0u8, 255u8], &swizzled[36..40]);
        assert_eq!(&[101u8, 0u8, 0u8, 255u8], &swizzled[40..44]);
        assert_eq!(&[121u8, 0u8, 0u8, 255u8], &swizzled[44..48]);

        assert_eq!(&[140u8, 0u8, 0u8, 255u8], &swizzled[256..260]);
        assert_eq!(&[158u8, 0u8, 0u8, 255u8], &swizzled[260..264]);
        assert_eq!(&[176u8, 0u8, 0u8, 255u8], &swizzled[264..268]);
        assert_eq!(&[193u8, 0u8, 0u8, 255u8], &swizzled[268..272]);

        assert_eq!(&[209u8, 0u8, 0u8, 255u8], &swizzled[288..292]);
        assert_eq!(&[224u8, 0u8, 0u8, 255u8], &swizzled[292..296]);
        assert_eq!(&[240u8, 0u8, 0u8, 255u8], &swizzled[296..300]);
        assert_eq!(&[255u8, 0u8, 0u8, 255u8], &swizzled[300..304]);
    }

    #[test]
    fn swizzle_black_white() {
        let data = crate::create_default_lut();
        let mut swizzled = vec![0u8; crate::image_size(16, 16, 16, 4)];
        swizzle(&data, &mut swizzled, false);

        // Black swizzled address: 0 (0000 0000 0000 0000)
        assert_eq!(&[0u8, 0u8, 0u8, 255u8], &swizzled[0..4]);

        // White swizzled address: 16380 (0011 1111 1111 1100)
        assert_eq!(&[255u8, 255u8, 255u8, 255u8], &swizzled[16380..16384]);
    }

    #[test]
    fn test_swizzle_deswizzle() {
        // Make sure deswizzling and then swizzling again is 1:1.
        // This ensures textures will be saved correctly.
        let original = crate::create_default_lut();
        let mut deswizzled = vec![0u8; crate::image_size(16, 16, 16, 4)];
        swizzle(&original, &mut deswizzled, true);

        let mut reswizzled = vec![0u8; crate::image_size(16, 16, 16, 4)];
        swizzle(&deswizzled, &mut reswizzled, false);

        let matching = original
            .iter()
            .zip(reswizzled.iter())
            .filter(|&(a, b)| a == b)
            .count();
        assert_eq!(matching, crate::image_size(16, 16, 16, 4));
    }
}
