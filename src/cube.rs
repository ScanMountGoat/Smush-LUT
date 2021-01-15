use std::io::{BufWriter, Write};
use indoc::indoc;

#[derive(Debug, PartialEq)]
struct CubeLut3d {
    title: String,
    size: u8,
    domain_min: (f32, f32, f32),
    domain_max: (f32, f32, f32),
    data: Vec<(f32, f32, f32)>,
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
        for (r, g, b) in &self.data {
            write!(&mut file, "{} {} {}\n", r, g, b)?
        }

        file.flush()?;
        Ok(())
    }

    /// Creates a new cube lut with the specified parameters.
    fn new(
        title: String,
        size: u8,
        domain_min: (f32, f32, f32),
        domain_max: (f32, f32, f32),
        data: Vec<(f32, f32, f32)>,
    ) -> CubeLut3d {
        CubeLut3d {
            title,
            size,
            domain_min,
            domain_max,
            data,
        }
    }

    fn from_text(text: &str) -> CubeLut3d {
        // Skip lines with "#" to ignore comments.
        // Trim each line because the spec allows for leading/trailing whitespace.
        let lines: Vec<&str> = text
            .lines()
            .filter(|s| !s.starts_with("#") && !s.is_empty())
            .map(|s| s.trim())
            .collect();

        let mut size: Option<u8> = Option::None;

        // Use the default values if not specified.
        let mut title: String = "".into();
        let mut domain_min = (0f32, 0f32, 0f32);
        let mut domain_max = (1f32, 1f32, 1f32);

        let mut data_starting_line: Option<usize> = Option::None;

        // Keywords can appear in any order.
        for (i, line) in lines.iter().enumerate() {
            match line.split_whitespace().collect::<Vec<&str>>()[..] {
                ["TITLE", value] => title = value.trim_matches('"').to_string(),
                ["LUT_3D_SIZE", value] => size = Some(value.parse::<u8>().unwrap()),
                ["DOMAIN_MIN", r, g, b] => {
                    domain_min = (r.parse().unwrap(), g.parse().unwrap(), b.parse().unwrap())
                }
                ["DOMAIN_MAX", r, g, b] => {
                    domain_max = (r.parse().unwrap(), g.parse().unwrap(), b.parse().unwrap())
                }
                [_r, _g, _b] => {
                    // The data is listed after all keyword lines.
                    data_starting_line = Some(i);
                    break;
                }
                _ => (),
            }
        }

        let parse_rgb = |s: &str| {
            let mut parts = s.split_whitespace();
            let r: f32 = parts.next()?.parse().ok()?;
            let g: f32 = parts.next()?.parse().ok()?;
            let b: f32 = parts.next()?.parse().ok()?;
            Some((r, g, b))
        };

        // Parse "0 0 1\n1 0 0..." into a single vector.
        let data_starting_line = data_starting_line.unwrap();
        let data: Vec<(f32, f32, f32)> = lines[data_starting_line..]
            .iter()
            .filter_map(|s| parse_rgb(s))
            .collect();

        // TODO: Make sure the size and the actual data length match.
        // TODO: Size must be greater than 2.
        CubeLut3d::new(title, size.unwrap(), domain_min, domain_max, data)
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
        assert_eq!(cube.domain_min, (0f32, 0f32, 0f32));
        assert_eq!(cube.domain_max, (1f32, 1f32, 1f32));
        assert_eq!(
            cube.data,
            vec![
                (0f32, 0f32, 0f32),
                (1f32, 0f32, 0f32),
                (0f32, 0.75f32, 0f32),
                (1f32, 0.75f32, 0f32),
                (0f32, 0.25f32, 1f32),
                (1f32, 0.25f32, 1f32),
                (0f32, 1f32, 1f32),
                (1f32, 1f32, 1f32)
            ]
        );
    }

    #[test]
    fn create_from_name_size_data() {
        let cube = CubeLut3d::new(
            "cube".into(),
            2,
            (0f32, 0f32, 0f32),
            (1f32, 1f32, 1f32),
            vec![(1f32, 1f32, 1f32); 8],
        );
        assert_eq!(cube.title, "cube");
        assert_eq!(cube.size, 2);
        assert_eq!(cube.domain_min, (0f32, 0f32, 0f32));
        assert_eq!(cube.domain_max, (1f32, 1f32, 1f32));
        assert_eq!(cube.data, vec![(1f32, 1f32, 1f32); 8]);
    }

    #[test]
    fn read_write() {
        // Make sure the parser and writer are compatible.
        let cube = CubeLut3d::new(
            "cube".into(),
            2,
            (0f32, 0f32, 0f32),
            (1f32, 1f32, 1f32),
            vec![(0.5f32, 0.5f32, 0.5f32); 8],
        );

        let mut c = Cursor::new(Vec::new());
        cube.write(&mut c).unwrap();

        let text = get_string(&mut c).unwrap();
        let new_cube = CubeLut3d::from_text(&text);

        assert_eq!(cube, new_cube);
    }

    #[test]
    fn write_new() {
        let cube = CubeLut3d::new(
            "cube".into(),
            2,
            (0f32, 0f32, 0f32),
            (1f32, 1f32, 1f32),
            vec![(1f32, 1f32, 1f32); 8],
        );

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
            1 1 1
            1 1 1
            1 1 1
            1 1 1
            1 1 1
            1 1 1
            1 1 1
        "#};

        assert_eq!(expected, actual);
    }
}
