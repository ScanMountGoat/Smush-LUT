use crate::Lut3dLinear;

// Calculate the final stage LUT for a LUT applied to a stage screenshot.
// See lut.ipynb for the underlying math and a 1D Python implementation.
pub fn correct_lut(lut_edit: &Lut3dLinear, lut_stage: &Lut3dLinear) -> Lut3dLinear {
    let mut lut_final = Lut3dLinear::empty_rgba(lut_edit.size);

    // TODO: Tests to make sure this works?
    // TODO: Figure out ways to make this more efficient.
    for z_index in 0..lut_edit.size {
        for y_index in 0..lut_edit.size {
            for x_index in 0..lut_edit.size {
                // Sample each point in the lut.
                // fx = f(srgb)
                // TODO: Test on empty lut?
                let fx = [
                    x_index as f32 / (lut_edit.size - 1) as f32,
                    y_index as f32 / (lut_edit.size - 1) as f32,
                    z_index as f32 / (lut_edit.size - 1) as f32,
                ];

                // srgb = finv(fx)
                let x = fx.map(f_inv);

                let mut result = lut_stage.sample_rgba_trilinear(fx[0], fx[1], fx[2]);

                for c in 0..3 {
                    result[c] = srgb(g(result[c], x[c]));
                }

                result = lut_edit.sample_rgba_trilinear(result[0], result[1], result[2]);

                for c in 0..3 {
                    result[c] = g_x_inv(linear(result[c]), x[c]);
                }

                // Alpha is always 1.0.
                result[3] = 1.0;

                lut_final.set_rgba(x_index, y_index, z_index, result);
            }
        }
    }

    lut_final
}

fn g(xi: f32, x: f32) -> f32 {
    (((xi - x) * 0.99961 + x) * 1.3703).max(0.0).powf(2.2)
}

// g is only invertible if we fix x to create a function g_x.
// We're cheating slightly here by making x a parameter.
// Creating a shared function just makes the code cleaner.
fn g_x_inv(xi: f32, x: f32) -> f32 {
    (((xi.max(0.0).powf(1.0 / 2.2) / 1.3703) - x) / 0.99961) + x
}

fn f(srgb: f32) -> f32 {
    srgb * 0.9375 + 0.03125
}

fn f_inv(fx: f32) -> f32 {
    (fx - 0.03125) / 0.9375
}

fn srgb(linear: f32) -> f32 {
    if linear <= 0.0031308 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

fn linear(srgb: f32) -> f32 {
    if srgb <= 0.04045 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

// TODO: Test cases based on debugging in RenderDoc.
// Test The linear <-> srgb conversions and post processing.
#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    #[test]
    fn srgb_linear_inverse() {
        // Check that these functions are inverses of each other.
        for x in 0..255 {
            let f = x as f32 / 255.0;
            assert_relative_eq!(f, linear(srgb(f)), epsilon = 0.0001f32);
            assert_relative_eq!(f, srgb(linear(f)), epsilon = 0.0001f32);
        }
    }

    #[test]
    fn f_f_inv() {
        // Check that these functions are inverses of each other.
        for x in 0..255 {
            let x = x as f32 / 255.0;
            assert_relative_eq!(x, f(f_inv(x)), epsilon = 0.0001f32);
            assert_relative_eq!(x, f_inv(f(x)), epsilon = 0.0001f32);
        }
    }

    #[test]
    fn g_g_x_inv() {
        // Check that these functions are inverses of each other.
        for x in 0..255 {
            let fx = x as f32 / 255.0;
            let fx = 0.5;
            let x = f_inv(fx);
            assert_relative_eq!(x, g(g_x_inv(fx, x), x), epsilon = 0.0001f32);
            assert_relative_eq!(x, g_x_inv(g(fx, x), x), epsilon = 0.0001f32);
        }
    }
}
