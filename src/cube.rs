use std::io::{BufWriter, Write};

use crate::Lut3dLinear;
#[cfg(test)]
use indoc::indoc;

#[derive(Debug, PartialEq)]
pub struct CubeLut3d {
    pub title: String,
    pub size: u8,
    pub domain_min: (f32, f32, f32),
    pub domain_max: (f32, f32, f32),
    pub data: Vec<(f32, f32, f32)>,
}

impl From<Lut3dLinear> for CubeLut3d {
    fn from(value: Lut3dLinear) -> Self {
        (&value).into()
    }
}

impl From<&Lut3dLinear> for CubeLut3d {
    fn from(lut: &Lut3dLinear) -> Self {
        let data = lut.data.chunks(4).map(|c| (c[0], c[1], c[2])).collect();

        CubeLut3d::new(
            "".into(),
            lut.size as u8,
            (0f32, 0f32, 0f32),
            (1f32, 1f32, 1f32),
            data,
        )
    }
}

impl CubeLut3d {
    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let mut file = BufWriter::new(writer);
        file.write_all(b"#Created by: smush_lut.exe\n")?;
        writeln!(&mut file, "TITLE \"{}\"", self.title)?;
        file.write_all(b"\n")?;

        file.write_all(b"#LUT Size\n")?;
        writeln!(&mut file, "LUT_3D_SIZE {}", self.size)?;
        file.write_all(b"\n")?;

        file.write_all(b"#data domain\n")?;
        file.write_all(b"DOMAIN_MIN 0.0 0.0 0.0\n")?;
        file.write_all(b"DOMAIN_MAX 1.0 1.0 1.0\n")?;
        file.write_all(b"\n")?;

        file.write_all(b"#LUT data points\n")?;
        for (r, g, b) in &self.data {
            writeln!(&mut file, "{r} {g} {b}")?
        }

        file.flush()?;
        Ok(())
    }

    /// Creates a new cube lut with the specified parameters.
    pub fn new(
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

    pub fn from_text(text: &str) -> Result<CubeLut3d, &'static str> {
        // Skip lines with "#" to ignore comments.
        // Trim each line because the spec allows for leading/trailing whitespace.
        let lines: Vec<&str> = text
            .lines()
            .map(|s| s.trim())
            .filter(|s| !s.starts_with('#') && !s.is_empty())
            .collect();

        let mut size: Option<u8> = Option::None;

        // Use the default values if not specified.
        let mut title: String = "".into();
        let mut domain_min = (0f32, 0f32, 0f32);
        let mut domain_max = (1f32, 1f32, 1f32);

        let mut data_starting_line: Option<usize> = Option::None;

        // Keywords can appear in any order.
        for (i, line) in lines.iter().enumerate() {
            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("TITLE") => {
                    // The title is within double quotes, so just grab the middle part.
                    title = line
                        .split('"')
                        .nth(1)
                        .ok_or("Missing value for TITLE.")?
                        .into();
                }
                Some("LUT_3D_SIZE") => {
                    if let Some(size_text) = parts.next() {
                        size = size_text.parse().ok()
                    }
                }
                Some("DOMAIN_MIN") => {
                    let values: Vec<f32> = parts
                        .take(3)
                        .filter_map(|f| f.parse::<f32>().ok())
                        .collect();
                    // TODO: This may fail.
                    domain_min = (values[0], values[1], values[2])
                }
                Some("DOMAIN_MAX") => {
                    let values: Vec<f32> = parts
                        .take(3)
                        .filter_map(|f| f.parse::<f32>().ok())
                        .collect();
                    // TODO: This may fail.
                    domain_max = (values[0], values[1], values[2])
                }
                _ => {
                    // The data is listed after all keyword lines.
                    data_starting_line = Some(i);
                    break;
                }
            }
        }

        let size = size.ok_or("Failed to parse LUT_3D_SIZE.")?;

        let parse_rgb = |s: &str| {
            let mut parts = s.split_whitespace();
            let r: f32 = parts.next()?.parse().ok()?;
            let g: f32 = parts.next()?.parse().ok()?;
            let b: f32 = parts.next()?.parse().ok()?;
            Some((r, g, b))
        };

        // Parse "0 0 1\n1 0 0..." into a single vector.
        let data_starting_line = data_starting_line.ok_or("Failed to find data points.")?;
        let data: Vec<(f32, f32, f32)> = lines[data_starting_line..]
            .iter()
            .filter_map(|s| parse_rgb(s))
            .collect();

        if data.len() != (size as usize).pow(3) {
            return Err("Data point count does not agree with LUT_3D_SIZE.");
        }

        // TODO: Make sure the size and the actual data length match.
        // TODO: Size must be greater than 2.
        let cube = CubeLut3d::new(title, size, domain_min, domain_max, data);
        Ok(cube)
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
    fn create_from_linear() {
        // Test u8 to f32 conversion.
        let linear = Lut3dLinear::from_rgba(
            2,
            [0u8, 51u8, 255u8, 255u8]
                .iter()
                .cycle()
                .take(32)
                .copied()
                .collect(),
        );
        let cube: CubeLut3d = linear.into();
        assert_eq!(cube.title, "");
        assert_eq!(cube.size, 2);
        assert_eq!(cube.domain_min, (0f32, 0f32, 0f32));
        assert_eq!(cube.domain_max, (1f32, 1f32, 1f32));
        assert_eq!(
            cube.data,
            vec![
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
            ]
        );
    }

    #[test]
    fn create_from_linear_ref() {
        // Test u8 to f32 conversion.
        let linear = Lut3dLinear::from_rgba(
            2,
            [0u8, 51u8, 255u8, 255u8]
                .iter()
                .cycle()
                .take(32)
                .copied()
                .collect(),
        );
        let cube: CubeLut3d = (&linear).into();
        assert_eq!(cube.title, "");
        assert_eq!(cube.size, 2);
        assert_eq!(cube.domain_min, (0f32, 0f32, 0f32));
        assert_eq!(cube.domain_max, (1f32, 1f32, 1f32));
        assert_eq!(
            cube.data,
            vec![
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
                (0.0f32, 0.2f32, 1.0f32),
            ]
        );
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
        let cube = CubeLut3d::from_text(text).unwrap();
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
    fn create_from_text_missing_size() {
        let text = "bad cube file";
        let cube = CubeLut3d::from_text(text);
        assert_eq!(cube, Err("Failed to parse LUT_3D_SIZE."));
    }

    #[test]
    fn create_from_text_no_data() {
        let text = indoc! {r#"
            TITLE "no data"
            LUT_3D_SIZE 2
        "#};
        let cube = CubeLut3d::from_text(text);
        assert_eq!(cube, Err("Failed to find data points."));
    }

    #[test]
    fn create_from_text_missing_size_value() {
        let text = indoc! {r#"
            # comment
            LUT_3D_SIZE  
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
        assert_eq!(cube, Err("Failed to parse LUT_3D_SIZE."));
    }

    #[test]
    fn create_from_text_invalid_rgb_triple() {
        let text = indoc! {r#"
            # comment
            LUT_3D_SIZE 2 
            0 0 0
            1 0 0
            0 .75 0
            1 .75 0
            0 .25 1
            1 1
            0 1 1
            1
        "#};
        let cube = CubeLut3d::from_text(text);
        assert_eq!(
            cube,
            Err("Data point count does not agree with LUT_3D_SIZE.")
        );
    }

    #[test]
    fn create_from_text_missing_title_value() {
        let text = indoc! {r#"
            # comment
            LUT_3D_SIZE 2
            TITLE  
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
        assert_eq!(cube, Err("Missing value for TITLE."));
    }

    #[test]
    fn create_from_text_domain_title() {
        let text = indoc! {r#"
            # comment
            DOMAIN_MIN -1 -1 -1


            LUT_3D_SIZE 2

            DOMAIN_MAX 1 2 3

            TITLE "lut1"

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
        let cube = CubeLut3d::from_text(text).unwrap();
        assert_eq!(cube.title, "lut1");
        assert_eq!(cube.size, 2);
        assert_eq!(cube.domain_min, (-1f32, -1f32, -1f32));
        assert_eq!(cube.domain_max, (1f32, 2f32, 3f32));
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
    fn create_from_text_title_spaces() {
        let text = indoc! {r#"
            LUT_3D_SIZE 2
            TITLE " a  b    c "
            0 0 0
            1 0 0
            0 .75 0
            1 .75 0
            0 .25 1
            1 .25 1
            0 1 1
            1 1 1
        "#};
        let cube = CubeLut3d::from_text(text).unwrap();
        assert_eq!(cube.title, " a  b    c ");
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
        let new_cube = CubeLut3d::from_text(&text).unwrap();

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
