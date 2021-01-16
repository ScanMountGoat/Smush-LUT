use std::convert::TryFrom;

use image::RgbaImage;

use crate::swizzle::swizzle;

/// A 3D RGBA LUT with unswizzled data in row major order.
/// Values are written to data using a nested ZYX loops with X being the innermost loop.
#[derive(Debug, PartialEq)]
pub struct Lut3dLinear {
    size: u32,
    data: Vec<u8>,
}

impl Lut3dLinear {
    /// The dimension of the LUT for each axis. A LUT with size 16 will have 16x16x16 RGBA values.
    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn new(size: u32, data: Vec<u8>) -> Lut3dLinear {
        Lut3dLinear { size, data }
    }
}

impl AsRef<[u8]> for Lut3dLinear {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl From<Lut3dSwizzled> for Lut3dLinear {
    /// Deswizzles the data in value to create a `Lut3dLinear` of identical size.
    fn from(value: Lut3dSwizzled) -> Self {
        let mut data = vec![0u8; value.data.len()];
        swizzle(&value.data, &mut data, true);
        Lut3dLinear {
            size: value.size,
            data,
        }
    }
}

// TODO: Implement for non reference as well?
impl TryFrom<& RgbaImage> for Lut3dLinear {
    type Error = &'static str;

    /// Tries to convert an image with slices in z arranged horizontally along the top of the image.
    /// For example, a 16x16x16 LUT image must have dimensions at least 256x16 pixels.
    fn try_from(value: &RgbaImage) -> Result<Self, Self::Error> {
        if value.width() != value.height() * value.height() {
            Err("Invalid dimensions. Expected width to equal height * height.")
        } else {
            let data = value.as_flat_samples().samples.to_vec();
            let lut = Lut3dLinear {
                size: value.height(),
                data,
            };
            Ok(lut)
        }
    }
}

impl TryFrom<Lut3dLinear> for RgbaImage {
    type Error = &'static str;

    fn try_from(value: Lut3dLinear) -> Result<Self, Self::Error> {
        RgbaImage::from_raw(value.size * value.size, value.size, value.data)
            .ok_or("Error creating RgbaImage.")
    }
}

/// A 3D RGBA LUT with swizzled data.
#[derive(Debug, PartialEq)]
pub struct Lut3dSwizzled {
    size: u32,
    data: Vec<u8>,
}

// TODO: Wrap this into another trait to store size, data by ref, etc?
impl Lut3dSwizzled {
    /// The dimension of the LUT for each axis. A LUT with size 16 will have 16x16x16 RGBA values.
    pub fn size(&self) -> u32 {
        self.size
    }
    
    pub fn new(size: u32, data: Vec<u8>) -> Lut3dSwizzled {
        Lut3dSwizzled { size, data }
    }
}

impl AsRef<[u8]> for Lut3dSwizzled {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl From<Lut3dLinear> for Lut3dSwizzled {
    /// Swizzles the data in value to create a `Lut3dLinear` of identical size.
    fn from(value: Lut3dLinear) -> Self {
        let mut data = vec![0u8; value.data.len()];
        swizzle(&value.data, &mut data, false);
        Lut3dSwizzled {
            size: value.size,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_to_rgba() {
        let data = crate::create_neutral_lut().to_vec();
        let linear = Lut3dLinear { size: 16u32, data };

        let img = RgbaImage::try_from(linear).unwrap();

        assert_eq!(256u32, img.width());
        assert_eq!(16u32, img.height());

        // Make sure the pixel values were copied correctly.
        let data = crate::create_neutral_lut().to_vec();
        let matching = data
            .iter()
            .zip(img.as_flat_samples().samples)
            .filter(|&(a, b)| a == b)
            .count();
        assert_eq!(matching, data.len());
    }

    #[test]
    fn rgba_to_linear() {
        let data = crate::create_neutral_lut().to_vec();
        let img = RgbaImage::from_raw(256, 16, data).unwrap();
        let linear = Lut3dLinear::try_from(&img).unwrap();

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
        let linear = Lut3dLinear::try_from(&img);

        assert_eq!(
            linear,
            Err("Invalid dimensions. Expected width to equal height * height.")
        );
    }

    #[test]
    fn linear_to_swizzled() {
        // Test that the data is correctly swizzled when converting.
        let data = crate::create_neutral_lut().to_vec();
        let linear = Lut3dLinear { size: 16u32, data };
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

        let swizzled = Lut3dSwizzled {
            size: 16u32,
            data: swizzled_data.to_vec(),
        };
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
