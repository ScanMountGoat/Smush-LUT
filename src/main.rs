use clap::{App, Arg};
use std::{
    convert::TryFrom,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use smush_lut::{correct_lut, Lut3dLinear};

fn main() {
    let matches = App::new("smush_lut")
        .version("0.2")
        .author("SMG")
        .about("Convert 3D color grading LUTs for Smash Ultimate")
        .arg(
            Arg::with_name("input")
                .index(1)
                .short("i")
                .long("input")
                .help("the input image, .cube, or .nutexb file")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("output")
                .index(2)
                .short("o")
                .long("output")
                .help("the output image, .cube, .nutexb, or .bin file")
                .required(false)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("raw")
                .short("r")
                .long("raw")
                .help("Exports the raw LUT values without any stage LUT compensation")
                .required(false)
                .takes_value(false),
        )
        .get_matches();

    let input: PathBuf = matches.value_of("input").unwrap().into();

    let input_extension = input
        .extension()
        .unwrap()
        .to_str()
        .expect("The input file must have an extension.");

    // Use the default conversion if no output is specified.
    let output: PathBuf = match matches.value_of("output") {
        Some(path) => path.into(),
        None => match input_extension {
            "nutexb" => input.with_extension("png").to_str().unwrap().into(),
            _ => input.with_extension("nutexb").to_str().unwrap().into(),
        },
    };

    let lut_linear = parse_input(&input).unwrap();

    // Check if the user wants to disable stage LUT compensation.
    let lut_final = if matches.is_present("raw") {
        lut_linear
    } else {
        // TODO: Make the stage lut an optional parameter?
        let lut_stage = Lut3dLinear::default_stage();

        correct_lut(&lut_linear, &lut_stage)
    };

    save_output(&lut_final, &output);
}

fn parse_input(input: &Path) -> Option<Lut3dLinear> {
    let parse = std::time::Instant::now();
    let lut_linear: Option<Lut3dLinear> = match input.extension().unwrap().to_str().unwrap() {
        "nutexb" => smush_lut::read_nutexb_lut(&input).ok(),
        "cube" => {
            let contents = fs::read_to_string(&input).unwrap();
            let cube = smush_lut::CubeLut3d::from_text(&contents).unwrap();

            Some(cube.into())
        }
        _ => {
            // Assume anything else is some form of supported image format.
            let img = image::open(&input).unwrap().into_rgba8();
            Lut3dLinear::try_from(&img).ok()
        }
    };

    eprintln!("Parse Time: {:?}", parse.elapsed());

    lut_linear
}

fn save_output(lut_linear: &Lut3dLinear, output: &Path) {
    let export = std::time::Instant::now();
    match output.extension().unwrap().to_str().unwrap() {
        "nutexb" => {
            smush_lut::write_lut_to_nutexb(lut_linear, output).unwrap();
        }
        "cube" => {
            smush_lut::linear_lut_to_cube(lut_linear, output).unwrap();
        }
        "bin" => {
            // Dump the unswizzled binary.
            let mut file = File::create(output).unwrap();
            file.write_all(&lut_linear.to_rgba()).unwrap();
        }
        _ => {
            // Assume anything else is some form of supported image format.
            let img = image::RgbaImage::try_from(lut_linear).unwrap();
            img.save(output).unwrap();
        }
    }
    eprintln!("Export Time: {:?}", export.elapsed());
}
