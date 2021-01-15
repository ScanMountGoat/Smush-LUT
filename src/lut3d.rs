use crate::swizzle::swizzle;

/// A 3d LUT with unswizzled data in row major order.
/// Values are written to data using a nested ZYX loops with X being the innermost loop.
pub struct Lut3dLinear {
    size: u32,
    data: Vec<u8>
}

impl From<Lut3dSwizzled> for Lut3dLinear {
    /// Deswizzles the data in lut to create a `Lut3dLinear` of identical size. 
    fn from(lut: Lut3dSwizzled) -> Self {
        let mut data = vec![0u8; lut.data.len()];
        swizzle(&lut.data, &mut data, true);
        Lut3dLinear { size: lut.size, data}
    }
}

/// A 3d LUT with swizzled data.
pub struct Lut3dSwizzled {
    size: u32,
    data: Vec<u8>
}

impl From<Lut3dLinear> for Lut3dSwizzled {
    /// Swizzles the data in lut to create a `Lut3dLinear` of identical size. 
    fn from(lut: Lut3dLinear) -> Self {
        let mut data = vec![0u8; lut.data.len()];
        swizzle(&lut.data, &mut data, false);
        Lut3dSwizzled { size: lut.size, data}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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