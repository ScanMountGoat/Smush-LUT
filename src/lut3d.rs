use std::convert::{TryFrom, TryInto};

use image::RgbaImage;
use nutexb::{NutexbFormat, ToNutexb};

use crate::{create_default_lut_f32, interp::trilinear, CubeLut3d, index3d, create_identity_lut_f32};

/// A 3D RGBA LUT with unswizzled data in row major order.
/// Values are written to data using a nested ZYX loops with X being the innermost loop.
// TODO: It makes sense to just use float here instead.
#[derive(Debug, PartialEq)]
pub struct Lut3dLinear {
    /// The dimensions for each axis.
    pub size: usize,
    pub data: Vec<f32>,
}

impl Lut3dLinear {
    pub fn empty_rgba(size: usize) -> Self {
        Self {
            size,
            data: vec![0.0f32; size * size * size * 4],
        }
    }

    pub fn from_rgba(size: usize, data: Vec<u8>) -> Self {
        Self {
            size,
            data: data.into_iter().map(|u| u as f32 / 255.0).collect(),
        }
    }

    pub fn to_rgba(&self) -> Vec<u8> {
        self.data.iter().map(|f| (f * 255.0) as u8).collect()
    }

    pub fn default_stage() -> Self {
        Self {
            size: 16,
            data: create_default_lut_f32(),
        }
    }

    pub fn identity() -> Self {
        Self  {
            size: 16,
            data: create_identity_lut_f32()
        }
    }

    pub fn set_rgba(&mut self, x: usize, y: usize, z: usize, rgba: [f32; 4]) {
        let i = index3d(x, y, z, self.size, self.size);
        self.data[i * 4..i * 4 + 4].copy_from_slice(&rgba);
    }

    pub fn sample_rgba_trilinear(&self, x: f32, y: f32, z: f32) -> [f32; 4] {
        let mut result = [0.0; 4];

        // TODO: Does this work for an empty lut?
        for c in 0..4 {
            let x0 = ((x * (self.size - 1) as f32) as usize).clamp(0, self.size - 1);
            let x1 = ((x * (self.size - 1) as f32).ceil() as usize).clamp(0, self.size - 1);

            let y0 = ((y * (self.size - 1) as f32) as usize).clamp(0, self.size - 1);
            let y1 = ((y * (self.size - 1) as f32).ceil() as usize).clamp(0, self.size - 1);

            let z0 = ((z * (self.size - 1) as f32) as usize).clamp(0, self.size - 1);
            let z1 = ((z * (self.size - 1) as f32).ceil() as usize).clamp(0, self.size - 1);

            let f000 = self.data[index3d(x0, y0, z0, self.size, self.size) * 4 + c];
            let f001 = self.data[index3d(x1, y0, z0, self.size, self.size) * 4 + c];
            let f010 = self.data[index3d(x0, y1, z0, self.size, self.size) * 4 + c];
            let f011 = self.data[index3d(x1, y1, z0, self.size, self.size) * 4 + c];
            let f100 = self.data[index3d(x0, y0, z1, self.size, self.size) * 4 + c];
            let f101 = self.data[index3d(x1, y0, z1, self.size, self.size) * 4 + c];
            let f110 = self.data[index3d(x0, y1, z1, self.size, self.size) * 4 + c];
            let f111 = self.data[index3d(x1, y1, z1, self.size, self.size) * 4 + c];

            // TODO: Does this correctly clamp to edge?
            result[c] = trilinear(
                (x, y, z),
                0.0,
                1.0,
                0.0,
                1.0,
                0.0,
                1.0,
                [f000, f001, f010, f011, f100, f101, f110, f111],
            );
        }

        result
    }
}

impl From<CubeLut3d> for Lut3dLinear {
    fn from(value: CubeLut3d) -> Self {
        let mut data = Vec::new();

        for (r, g, b) in value.data {
            // Always use 1.0 for alpha to match in game nutexb LUTs.
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(1.0);
        }

        Lut3dLinear {
            size: value.size as usize,
            data,
        }
    }
}

impl TryFrom<RgbaImage> for Lut3dLinear {
    type Error = &'static str;

    /// Tries to convert an image with slices in z arranged horizontally along the top of the image.
    /// For example, a 16x16x16 LUT image must have dimensions at least 256x16 pixels.
    fn try_from(value: RgbaImage) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl TryFrom<&RgbaImage> for Lut3dLinear {
    type Error = &'static str;

    /// Tries to convert an image with slices in z arranged horizontally along the top of the image.
    /// For example, a 16x16x16 LUT image must have dimensions at least 256x16 pixels.
    fn try_from(value: &RgbaImage) -> Result<Self, Self::Error> {
        if value.width() != value.height() * value.height() {
            Err("Invalid dimensions. Expected width to equal height * height.")
        } else {
            Ok(Lut3dLinear::from_rgba(
                value.height() as usize,
                value.as_flat_samples().samples.to_vec(),
            ))
        }
    }
}

impl TryFrom<Lut3dLinear> for RgbaImage {
    type Error = &'static str;

    fn try_from(value: Lut3dLinear) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}

impl TryFrom<&Lut3dLinear> for RgbaImage {
    type Error = &'static str;

    fn try_from(value: &Lut3dLinear) -> Result<Self, Self::Error> {
        RgbaImage::from_raw(
            (value.size * value.size) as u32,
            value.size as u32,
            value.to_rgba(),
        )
        .ok_or("Error creating RgbaImage.")
    }
}

impl ToNutexb for Lut3dLinear {
    fn width(&self) -> u32 {
        self.size as u32
    }

    fn height(&self) -> u32 {
        self.size as u32
    }

    fn depth(&self) -> u32 {
        self.size as u32
    }

    fn image_data(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(self.to_rgba())
    }

    fn mipmap_count(&self) -> u32 {
        1
    }

    fn layer_count(&self) -> u32 {
        1
    }

    fn image_format(&self) -> Result<nutexb::NutexbFormat, Box<dyn std::error::Error>> {
        Ok(NutexbFormat::R8G8B8A8Unorm)
    }
}

#[cfg(test)]
mod tests {
    use crate::create_default_lut_f32;

    use super::*;

    use indoc::indoc;

    #[test]
    fn cube_to_linear() {
        let text = indoc! {r#"
            # comment

            LUT_3D_SIZE 2

            # comment
            0 0 0
            1 0 0
            0 .75 0
            1 .75 0
            0 .25 1
            1 .25 1
            0 1 1
            1 1 1
        "#};
        let cube = CubeLut3d::from_text(text).unwrap();
        let linear = Lut3dLinear::from(cube);

        assert_eq!(2, linear.size);
        assert_eq!(
            &linear.data,
            &[
                0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.75, 0.0, 1.0, 1.0, 0.75, 0.0, 1.0,
                0.0, 0.25, 1.0, 1.0, 1.0, 0.25, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            ],
        )
    }

    #[test]
    fn linear_to_rgba() {
        let data = crate::create_default_lut();
        let linear = Lut3dLinear::from_rgba(16, data);

        let img = RgbaImage::try_from(linear).unwrap();

        assert_eq!(256u32, img.width());
        assert_eq!(16u32, img.height());

        // Make sure the pixel values were copied correctly.
        let data = crate::create_default_lut();
        assert_eq!(&data, img.as_flat_samples().samples);
    }

    #[test]
    fn linear_ref_to_rgba() {
        let data = crate::create_default_lut();
        let linear = Lut3dLinear::from_rgba(16, data);

        let img = RgbaImage::try_from(&linear).unwrap();

        assert_eq!(256u32, img.width());
        assert_eq!(16u32, img.height());

        // Make sure the pixel values were copied correctly.
        let data = crate::create_default_lut();
        assert_eq!(&data, img.as_flat_samples().samples);
    }

    #[test]
    fn rgba_ref_to_linear() {
        let data = crate::create_default_lut();
        let img = RgbaImage::from_raw(256, 16, data).unwrap();
        let linear = Lut3dLinear::try_from(&img).unwrap();

        assert_eq!(16, linear.size);
        assert_eq!(crate::image_size(16, 16, 16, 4), linear.data.len());

        // Make sure the pixel values were copied correctly.
        let data = create_default_lut_f32();
        assert_eq!(&data, &linear.data);
    }

    #[test]
    fn rgba_to_linear() {
        let data = crate::create_default_lut();
        let img = RgbaImage::from_raw(256, 16, data).unwrap();
        let linear = Lut3dLinear::try_from(img).unwrap();

        assert_eq!(16, linear.size);
        assert_eq!(crate::image_size(16, 16, 16, 4), linear.data.len());

        // Make sure the pixel values were copied correctly.
        let data = create_default_lut_f32();
        assert_eq!(&data, &linear.data);
    }

    #[test]
    fn rgba_to_linear_invalid_dimensions() {
        // The width should be height^2.
        let data = crate::create_default_lut();
        let img = RgbaImage::from_raw(128, 32, data).unwrap();
        let linear = Lut3dLinear::try_from(&img);

        assert_eq!(
            linear,
            Err("Invalid dimensions. Expected width to equal height * height.")
        );
    }
}
