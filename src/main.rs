use std::{convert::TryFrom, fs};

use smush_lut::Lut3dLinear;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = std::path::PathBuf::from(&args[1]);
    let output = std::path::PathBuf::from(&args[2]);

    // TODO: Add better error handling.
    // TODO: Add better argument parsing (clap?).

    let lut_linear: Option<Lut3dLinear> = match input.extension().unwrap().to_str().unwrap() {
        "nutexb" => {
            smush_lut::read_lut_from_nutexb(&input)
        }
        "cube" => {
            let contents = fs::read_to_string(&input).unwrap();

            // TODO: this should by try_from
            let cube = smush_lut::CubeLut3d::from_text(&contents);

            // TODO: this should by try_from
            Some(cube.into())
        }
        _ => {
            // Assume anything else is some form of supported image format.
            let img = image::open(&input).unwrap().into_rgba8();
            Lut3dLinear::try_from(&img).ok()
        }
    };

    let lut_linear = lut_linear.unwrap();

    match output.extension().unwrap().to_str().unwrap() {
        "nutexb" => {
            smush_lut::linear_lut_to_nutexb(lut_linear, &output).unwrap();
        }
        "cube" => {
            // TODO: this should by try_from
            smush_lut::linear_lut_to_cube(lut_linear, &output).unwrap();
        }
        _ => {
            // Assume anything else is some form of supported image format.
            let img = image::RgbaImage::try_from(lut_linear).unwrap();
            img.save(output).unwrap();
        }
    }
}
