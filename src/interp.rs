// https://en.wikipedia.org/wiki/Trilinear_interpolation
pub fn linear(x: f32, x0: f32, x1: f32, f0: f32, f1: f32) -> f32 {
    let factor = (x - x0) / (x1 - x0);
    (1.0 - factor) * f0 + factor * f1
}

pub fn bilinear(xy: (f32, f32), x0: f32, x1: f32, y0: f32, y1: f32, fxy: [f32; 4]) -> f32 {
    let (x, y) = xy;

    // Interpolate twice in x and once in y.
    // Binary indices are fzyx in row-major order.
    let r1 = linear(x, x0, x1, fxy[0b00], fxy[0b01]);
    let r2 = linear(x, x0, x1, fxy[0b10], fxy[0b11]);

    linear(y, y0, y1, r1, r2)
}

pub fn trilinear(
    xyz: (f32, f32, f32),
    x0: f32,
    x1: f32,
    y0: f32,
    y1: f32,
    z0: f32,
    z1: f32,
    fxyz: [f32; 8],
) -> f32 {
    let (x, y, z) = xyz;

    // Interpolate two xy planes.
    // Binary indices are fzyx in row-major order.
    let face0 = bilinear(
        (x, y),
        x0,
        x1,
        y0,
        y1,
        [fxyz[0b000], fxyz[0b001], fxyz[0b010], fxyz[0b011]],
    );
    let face1 = bilinear(
        (x, y),
        x0,
        x1,
        y0,
        y1,
        [fxyz[0b100], fxyz[0b101], fxyz[0b110], fxyz[0b111]],
    );

    // Interpolate the planes in z
    linear(z, z0, z1, face0, face1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_interpolation() {
        assert_eq!(-1.0, linear(-1.0, 0.0, 1.0, 0.0, 1.0));
        assert_eq!(0.0, linear(0.0, 0.0, 1.0, 0.0, 1.0));
        assert_eq!(0.5, linear(0.5, 0.0, 1.0, 0.0, 1.0));
        assert_eq!(1.0, linear(1.0, 0.0, 1.0, 0.0, 1.0));
        assert_eq!(2.0, linear(2.0, 0.0, 1.0, 0.0, 1.0));
    }

    #[test]
    fn bilinear_interpolation() {
        // Test corners.
        let xy = [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)];
        let values = [1.0, 2.0, 3.0, 4.0];

        for i in 0..values.len() {
            assert_eq!(values[i], bilinear(xy[i], 0.0, 1.0, 0.0, 1.0, values));
        }

        // Test center.
        assert_eq!(
            values.iter().sum::<f32>() / values.len() as f32,
            bilinear((0.5, 0.5), 0.0, 1.0, 0.0, 1.0, values)
        );
    }

    #[test]
    fn trilinear_interpolation() {
        // Test corners.
        let xyz = [
            (0.0, 0.0, 0.0),
            (1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            (1.0, 1.0, 0.0),
            (0.0, 0.0, 1.0),
            (1.0, 0.0, 1.0),
            (0.0, 1.0, 1.0),
            (1.0, 1.0, 1.0),
        ];
        let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];

        for i in 0..values.len() {
            assert_eq!(
                values[i],
                trilinear(xyz[i], 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, values)
            );
        }

        // Test center.
        assert_eq!(
            values.iter().sum::<f32>() / values.len() as f32,
            trilinear((0.5, 0.5, 0.5), 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, values)
        )
    }
}
