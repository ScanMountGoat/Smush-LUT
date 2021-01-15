use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use indoc::indoc;
use itertools::Itertools;

struct CubeLut3d {
    title: String,
    size: u32,
    domain_min_r: f32,
    domain_min_g: f32,
    domain_min_b: f32,
    domain_max_r: f32,
    domain_max_g: f32,
    domain_max_b: f32,
    data: Vec<f32>,
}

impl CubeLut3d {
    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let mut file = BufWriter::new(writer);
        file.write(b"#Created by: smush_lut.exe\n")?;
        write!(&mut file, "TITLE \"{}\"\n", self.title)?;
        file.write(b"\n")?;

        file.write(b"#LUT Size\n")?;
        write!(&mut file, "LUT_3D_SIZE {}\n", self.size)?;
        file.write(b"\n")?;

        file.write(b"#data domain\n")?;
        file.write(b"DOMAIN_MIN 0.0 0.0 0.0\n")?;
        file.write(b"DOMAIN_MAX 1.0 1.0 1.0\n")?;
        file.write(b"\n")?;

        file.write(b"#LUT data points\n")?;
        for values in self.data.chunks(3) {
            if let [r, g, b] = values {
                write!(&mut file, "{} {} {}\n", r, g, b)?;
            }
        }

        file.flush()?;
        Ok(())
    }

    /// Creates a new cube lut with the specified title, data, and size.
    /// The domain min and max are set to (0.0,0.0,0.0) and (1.0,1.0,1.0) respectively.
    fn new(title: String, size: u32, data: Vec<f32>) -> CubeLut3d {
        CubeLut3d {
            title,
            size,
            domain_min_r: 0.0f32,
            domain_min_g: 0.0f32,
            domain_min_b: 0.0f32,
            domain_max_r: 1.0f32,
            domain_max_g: 1.0f32,
            domain_max_b: 1.0f32,
            data,
        }
    }

    fn from_text(text: &str) -> CubeLut3d {
        // TODO: read title if present
        // TODO: read domain if present

        // TODO: Account for whitespace.
        // Skip lines with "#" to ignore comments.
        let mut lines = text.lines().filter(|s| !s.starts_with("#"));
        
        let size_header = (&mut lines)
            .skip_while(|s| !s.starts_with("LUT_3D_SIZE"))
            .next()
            .unwrap();

        let size: u32 = size_header
            .split_whitespace()
            .skip(1)
            .next()
            .unwrap()
            .parse()
            .unwrap();

        // Parse "0 0 1\n1 0 0..." into a single vector.
        let data: Vec<f32> = lines
            .map(|l| l.split_whitespace())
            .flatten()
            .filter_map(|s| s.parse().ok())
            .collect();

        // TODO: Make sure the size and the actual data length match.
        CubeLut3d::new("".into(), size, data)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read, Seek, SeekFrom};

    use super::*;

    fn get_string(c: &mut Cursor<Vec<u8>>) -> Option<String> {
        // Get the string contents.
        let mut out = Vec::new();
        c.seek(SeekFrom::Start(0)).ok()?;
        c.read_to_end(&mut out).ok()?;
        let out = String::from_utf8(out).ok()?;
        Some(out)
    }

    #[test]
    fn create_from_text_size2() {
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
        let cube = CubeLut3d::from_text(text);
        assert_eq!(cube.title, "");
        assert_eq!(cube.size, 2);
        assert_eq!(cube.domain_min_r, 0.0f32);
        assert_eq!(cube.domain_min_g, 0.0f32);
        assert_eq!(cube.domain_min_b, 0.0f32);
        assert_eq!(cube.domain_max_r, 1.0f32);
        assert_eq!(cube.domain_max_g, 1.0f32);
        assert_eq!(cube.domain_max_b, 1.0f32);
        assert_eq!(
            cube.data,
            vec![
                0f32, 0f32, 0f32, 1f32, 0f32, 0f32, 0f32, 0.75f32, 0f32, 1f32, 0.75f32, 0f32, 0f32,
                0.25f32, 1f32, 1f32, 0.25f32, 1f32, 0f32, 1f32, 1f32, 1f32, 1f32, 1f32
            ]
        );
    }

    #[test]
    fn create_from_name_size_data() {
        let cube = CubeLut3d::new("cube".into(), 2, vec![1f32; 8]);
        assert_eq!(cube.title, "cube");
        assert_eq!(cube.size, 2);
        assert_eq!(cube.domain_min_r, 0.0f32);
        assert_eq!(cube.domain_min_g, 0.0f32);
        assert_eq!(cube.domain_min_b, 0.0f32);
        assert_eq!(cube.domain_max_r, 1.0f32);
        assert_eq!(cube.domain_max_g, 1.0f32);
        assert_eq!(cube.domain_max_b, 1.0f32);
        assert_eq!(cube.data, vec![1f32; 8]);
    }

    #[test]
    fn write_new() {
        let cube = CubeLut3d::new("cube".into(), 2, vec![1f32; 3]);

        let mut c = Cursor::new(Vec::new());
        cube.write(&mut c).unwrap();

        let actual = get_string(&mut c).unwrap();

        let expected = indoc! {r#"
            #Created by: smush_lut.exe
            TITLE "cube"
            
            #LUT Size
            LUT_3D_SIZE 2
            
            #data domain
            DOMAIN_MIN 0.0 0.0 0.0
            DOMAIN_MAX 1.0 1.0 1.0
            
            #LUT data points
            1 1 1
        "#};

        assert_eq!(expected, actual);
    }
}
