use crate::Lut3dLinear;

pub fn correct_lut(lut_edit: &Lut3dLinear, lut_stage: &Lut3dLinear) -> Lut3dLinear {
    // Calculate the final stage LUT for a LUT applied to a stage screenshot.
    let mut lut_final = Lut3dLinear::empty_rgba(lut_edit.size);

    // TODO: Figure out ways to make this more efficient.
    for z_index in 0..lut_edit.size {
        for y_index in 0..lut_edit.size {
            for x_index in 0..lut_edit.size {
                // TODO: Make functions over [f32; 4] so this can match the docs.
                // Sample each point xi = f(x) in the lut.
                // TODO: Test on empty lut?
                let xi = [
                    x_index as f32 / (lut_edit.size - 1) as f32,
                    y_index as f32 / (lut_edit.size - 1) as f32,
                    z_index as f32 / (lut_edit.size - 1) as f32,
                ];

                // result = lut_stage(xi)
                let mut result = lut_stage.sample_rgba_trilinear(xi[0], xi[1], xi[2]);

                // result = srgb(g_x(lut_stage(xi)))
                let x = xi.map(f_inv);
                for c in 0..3 {
                    result[c] = srgb(g_x(result[c], x[c]));
                }

                // result = lut_edit(srgb(g_x(lut_stage(xi))))
                result = lut_edit.sample_rgba_trilinear(result[0], result[1], result[2]);

                // result = g_x_inv(linear(lut_edit(srgb(g_x(lut_stage(xi))))))
                for c in 0..3 {
                    result[c] = g_x_inv(linear(result[c]), x[c]);
                }

                // Alpha is always 1.0.
                result[3] = 1.0;

                // lut_final(xi) = g_x_inv(linear(lut_edit(srgb(g_x(lut_stage(xi))))))
                // https://github.com/ScanMountGoat/Smush-LUT/blob/master/color_correction.md
                lut_final.set_rgba(x_index, y_index, z_index, result);
            }
        }
    }

    lut_final
}

fn g_x(xi: f32, x: f32) -> f32 {
    (((xi - x) * 0.99961 + x) * 1.3703).max(0.0).powf(2.2)
}

// g is only invertible if we fix x to create a function g_x.
// We're cheating slightly here by making x a parameter.
// Creating a shared function just makes the code cleaner.
fn g_x_inv(xi: f32, x: f32) -> f32 {
    (((xi.max(0.0).powf(1.0 / 2.2) / 1.3703) - x) / 0.99961) + x
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
#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;

    fn f(srgb: f32) -> f32 {
        srgb * 0.9375 + 0.03125
    }

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
            let x = f_inv(fx);
            assert_relative_eq!(fx, g_x(g_x_inv(fx, x), x), epsilon = 0.0001f32);
            assert_relative_eq!(fx, g_x_inv(g_x(fx, x), x), epsilon = 0.0001f32);
        }
    }

    #[test]
    fn correct_identity_lut() {
        let lut_edit = Lut3dLinear::identity();
        let lut_stage = Lut3dLinear::identity();

        // TODO: Investigate if it's possible to reduce this error.
        let corrected = correct_lut(&lut_edit, &lut_stage);
        assert_relative_eq!(corrected.data[..], lut_edit.data[..], epsilon = 0.1f32);
    }
}
