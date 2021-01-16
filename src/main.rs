use std::{fs, path::Path};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = std::path::PathBuf::from(&args[1]);

    // TODO: Add better error handling.

    match input.extension().unwrap().to_str().unwrap() {
        "nutexb" => {
            let mut output = input.clone();
            output.set_extension("png");

            let img = smush_lut::nutexb_to_image(&input).unwrap();
            img.save(output).unwrap();
        }
        "cube" => {
            let contents = fs::read_to_string(&input).unwrap();

            // TODO: This can be try_from.
            let cube = smush_lut::CubeLut3d::from_text(&contents);
            // TODO: modify the input path?
            smush_lut::cube_to_nutexb(cube, &Path::new("color_grading_lut.nutexb")).unwrap();
        }
        _ => {
            let img = image::open(&input).unwrap().into_rgba8();
            // TODO: modify the input path?
            smush_lut::img_to_nutexb(&img, &Path::new("color_grading_lut.nutexb")).unwrap();
        }
    }
}
