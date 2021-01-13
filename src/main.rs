fn main() {
    let args: Vec<String> = std::env::args().collect();
    let input = std::path::PathBuf::from(&args[1]);
    let mut img = image::open(&input).unwrap().into_rgba8();

    // Infer whether to read or write the LUT based on the file name.
    // This allows using a single argument to enable drag + drop support.
    if input
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap()
        .ends_with(".lut.png")
    {
        smush_lut::write_nutexb(&img, "color_grading_lut.nutexb").unwrap();
    } else {
        smush_lut::write_lut_to_img(&mut img);
        let mut output = input.clone();
        output.set_extension("lut.png");
        img.save(output).unwrap();
    }
}
