use std::convert::TryFrom;

use image::{GenericImageView, Rgba, RgbaImage};

use crate::swizzle::swizzle;

/// A 3d LUT with unswizzled data in row major order.
/// Values are written to data using a nested ZYX loops with X being the innermost loop.
#[derive(Debug, PartialEq)]
pub struct Lut3dLinear {
    size: u32,
    data: Vec<u8>
}

impl From<Lut3dSwizzled> for Lut3dLinear {
    /// Deswizzles the data in value to create a `Lut3dLinear` of identical size. 
    fn from(value: Lut3dSwizzled) -> Self {
        let mut data = vec![0u8; value.data.len()];
        swizzle(&value.data, &mut data, true);
        Lut3dLinear { size: value.size, data}
    }
}

impl TryFrom<RgbaImage> for Lut3dLinear {
    type Error = &'static str;

    /// Tries to convert an image with slices in z arranged horizontally along the top of the image. 
    /// For example, a 16x16x16 LUT image must have dimensions at least 256x16 pixels. 
    fn try_from(value: RgbaImage) -> Result<Self, Self::Error> {
        if value.width() != value.height() * value.height() {
            Err("Invalid dimensions. Expected width to equal height * height.")
        } else {
            let data = read_lut_from_image(&value);
            let lut = Lut3dLinear { size: value.height(), data };
            Ok(lut)
        }
    }
}

fn read_lut_from_image(img: &RgbaImage) -> Vec<u8> {
    // TODO: It might be possible to just use raw because the image dimensions are known.
    let mut data = Vec::new();
    for Rgba(bytes) in img.pixels() {
        data.extend_from_slice(bytes);
    }
    data
}

/// A 3d LUT with swizzled data.
#[derive(Debug, PartialEq)]
pub struct Lut3dSwizzled {
    size: u32,
    data: Vec<u8>
}

impl From<Lut3dLinear> for Lut3dSwizzled {
    /// Swizzles the data in value to create a `Lut3dLinear` of identical size. 
    fn from(value: Lut3dLinear) -> Self {
        let mut data = vec![0u8; value.data.len()];
        swizzle(&value.data, &mut data, false);
        Lut3dSwizzled { size: value.size, data}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_to_linear() {
        let data = crate::create_neutral_lut().to_vec();
        let img = RgbaImage::from_raw(256, 16, data).unwrap();
        let linear = Lut3dLinear::try_from(img).unwrap();

        assert_eq!(16u32, linear.size);
        assert_eq!(crate::image_size(16, 16, 16, 4), linear.data.len());

        // Make sure the pixel values were copied correctly.
        let data = crate::create_neutral_lut().to_vec();
        let matching = data
            .iter()
            .zip(linear.data.iter())
            .filter(|&(a, b)| a == b)
            .count();
        assert_eq!(matching, data.len());
    }

    #[test]
    fn rgba_to_linear_invalid_dimensions() {
        // The width should be height^2. 
        let data = crate::create_neutral_lut().to_vec();
        let img = RgbaImage::from_raw(128, 32, data).unwrap();
        let linear = Lut3dLinear::try_from(img);

        assert_eq!(linear, Err("Invalid dimensions. Expected width to equal height * height."));
    }

    #[test]
    fn linear_to_swizzled() {
        // Test that the data is correctly swizzled when converting.
        let data = crate::create_neutral_lut().to_vec();
        let linear = Lut3dLinear { size: 16u32, data};
        let swizzled: Lut3dSwizzled = linear.into();

        assert_eq!(16u32, swizzled.size);

        // Black swizzled address: 0 (0000 0000 0000 0000)
        assert_eq!(&[0u8, 0u8, 0u8, 255u8], &swizzled.data[0..4]);

        // Red swizzled address: 300 (0000 0001 0010 1100)
        assert_eq!(&[255u8, 0u8, 0u8, 255u8], &swizzled.data[300..304]);

        // Green swizzled address: 8400 (0010 0000 1101 0000)
        assert_eq!(&[0u8, 255u8, 0u8, 255u8], &swizzled.data[8400..8404]);

        // Blue swizzled address: 7680 (0001 1110 0000 0000)
        assert_eq!(&[0u8, 0u8, 255u8, 255u8], &swizzled.data[7680..7684]);
    }

    #[test]
    fn swizzled_to_linear() {
        // Test that the data is correctly deswizzled when converting.
        let data = crate::create_neutral_lut();
        let mut swizzled_data = [0u8; crate::image_size(16, 16, 16, 4)];
        swizzle(&data, &mut swizzled_data, false);

        let swizzled = Lut3dSwizzled { size: 16u32, data: swizzled_data.to_vec()};
        let linear: Lut3dLinear = swizzled.into();

        assert_eq!(16u32, linear.size);

        let matching = data
            .iter()
            .zip(linear.data.iter())
            .filter(|&(a, b)| a == b)
            .count();
        assert_eq!(matching, crate::image_size(16, 16, 16, 4));
    }
}