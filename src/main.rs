use std::fs;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = std::path::PathBuf::from(&args[1]);

    // TODO: Add better error handling.

    // Infer whether to read or write the LUT based on the file name.
    // This allows using a single argument to enable drag + drop support.
    if input
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap()
        .contains(".lut.")
    {
        let img = image::open(&input).unwrap().into_rgba8();
        smush_lut::write_nutexb(&img, "color_grading_lut.nutexb").unwrap();
    } else {
        match input.extension().unwrap().to_str().unwrap() {
            "nutexb" => {
                let mut output = input.clone();
                output.set_extension("png");

                let img = smush_lut::read_image_lut_from_nutexb(&input).unwrap();
                img.save(output).unwrap();
            }
            "cube" => {
                let contents = fs::read_to_string(&input).unwrap();

                // TODO: This can be try_from.
                let cube = smush_lut::CubeLut3d::from_text(&contents);

                // TODO: Convert to nutexb by default.
            }
            _ => {
                let mut img = image::open(&input).unwrap().into_rgba8();
                smush_lut::write_neutral_lut_to_img(&mut img);
                let mut output = input.clone();
                output.set_extension("lut.png");
                img.save(output).unwrap();
            }
        }
    }
}
