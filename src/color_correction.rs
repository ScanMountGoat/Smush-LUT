use crate::{interp::trilinear, Lut3dLinear};

// Calculate the final stage LUT for a LUT applied to a stage screenshot.
// See lut.ipynb for the underlying math and a Python sample.
pub fn correct_lut(lut_edit: &Lut3dLinear, lut_stage: &Lut3dLinear) -> Lut3dLinear {
    let mut lut_final = vec![0.0f32; lut_edit.size * lut_edit.size * lut_edit.size * 4];

    // TODO: Tests to make sure this works?
    // TODO: Figure out ways to make this more efficient.
    for z_index in 0..lut_edit.size {
        for y_index in 0..lut_edit.size {
            for x_index in 0..lut_edit.size {
                // Sample each point in the lut.
                // fx = f(srgb)
                let fx = x_index as f32 / 15.0;
                let fy = y_index as f32 / 15.0;
                let fz = z_index as f32 / 15.0;

                // srgb = finv(fx)
                let x = f_inv(fx);
                let y = f_inv(fy);
                let z = f_inv(fz);

                let mut result = lut(fx, fy, fz, &lut_stage.data);
                result[0] = g(result[0], x);
                result[1] = g(result[1], y);
                result[2] = g(result[2], z);

                result[0] = srgb(result[0]);
                result[1] = srgb(result[1]);
                result[2] = srgb(result[2]);

                result = lut(result[0], result[1], result[2], &lut_edit.data);

                result[0] = linear(result[0]);
                result[1] = linear(result[1]);
                result[2] = linear(result[2]);

                result[0] = g_x_inv(result[0], x);
                result[1] = g_x_inv(result[1], y);
                result[2] = g_x_inv(result[2], z);

                // Alpha is always 1.0.
                result[3] = 1.0;

                let i = index3d(x_index, y_index, z_index);
                for c in 0..4 {
                    lut_final[i * 4 + c] = result[c];
                }
            }
        }
    }

    Lut3dLinear {
        size: 16,
        data: lut_final,
    }
}
// TODO: How to test this since there may be some error?
// TODO: Tests for 3d indexing?
fn index3d(x: usize, y: usize, z: usize) -> usize {
    z * 16 * 16 + y * 16 + x
}

// TODO: Don't assume the size is 16x16x16.
// TODO: Make this a method on Lut3dLinear?
// TODO: Tests?
fn lut(x: f32, y: f32, z: f32, values: &[f32]) -> [f32; 4] {
    let mut result = [0.0; 4];

    // TODO: Technically we don't need to calculate alpha since it's always 1.0.
    for i in 0..4 {
        let x0 = ((x * 15.0) as usize).clamp(0, 15);
        let x1 = ((x * 15.0).ceil() as usize).clamp(0, 15);

        let y0 = ((y * 15.0) as usize).clamp(0, 15);
        let y1 = ((y * 15.0).ceil() as usize).clamp(0, 15);

        let z0 = ((z * 15.0) as usize).clamp(0, 15);
        let z1 = ((z * 15.0).ceil() as usize).clamp(0, 15);

        let f000 = values[index3d(x0, y0, z0) * 4 + i];
        let f001 = values[index3d(x1, y0, z0) * 4 + i];
        let f010 = values[index3d(x0, y1, z0) * 4 + i];
        let f011 = values[index3d(x1, y1, z0) * 4 + i];
        let f100 = values[index3d(x0, y0, z1) * 4 + i];
        let f101 = values[index3d(x1, y0, z1) * 4 + i];
        let f110 = values[index3d(x0, y1, z1) * 4 + i];
        let f111 = values[index3d(x1, y1, z1) * 4 + i];

        result[i] = trilinear(
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

fn g(x: f32, srgb: f32) -> f32 {
    (((x - srgb) * 0.99961 + srgb) * 1.3703).powf(2.2)
}

// g is only invertible if we fix x to create a function g_x.
// We're cheating slightly here by making x a parameter.
// Creating a shared function just makes the code cleaner.
fn g_x_inv(fx: f32, x: f32) -> f32 {
    (((fx.max(0.0).powf(1.0 / 2.2) / 1.3703) - x) / 0.99961) + x
}

fn f(srgb: f32) -> f32 {
    srgb * 0.9375 + 0.03125
}

fn f_inv(xi: f32) -> f32 {
    ((xi - 0.03125) / 0.9375).max(0.0)
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
