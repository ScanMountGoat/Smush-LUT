use std::io::{BufWriter, Write};

use crate::Lut3dLinear;
use itertools::Itertools;

use self::parser::KeyWord;

#[derive(Debug, PartialEq)]
pub struct CubeLut3d {
    title: String,
    size: u32,
    domain_min: (f32, f32, f32),
    domain_max: (f32, f32, f32),
    data: Vec<(f32, f32, f32)>,
}

impl From<Lut3dLinear> for CubeLut3d {
    fn from(value: Lut3dLinear) -> Self {
        (&value).into()
    }
}

impl From<&Lut3dLinear> for CubeLut3d {
    fn from(value: &Lut3dLinear) -> Self {
        // Map RGBA u8 values to (r,g,b) f32 values in the range 0.0 to 1.0.
        let data = value
            .as_ref()
            .iter()
            .map(|u| *u as f32 / 255f32)
            .tuples::<(f32, f32, f32, f32)>()
            .map(|t| (t.0, t.1, t.2))
            .collect();

        CubeLut3d::new(
            "".into(),
            value.size(),
            (0f32, 0f32, 0f32),
            (1f32, 1f32, 1f32),
            data,
        )
    }
}

impl CubeLut3d {
    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn domain_min(&self) -> (f32, f32, f32) {
        self.domain_min
    }

    pub fn domain_max(&self) -> (f32, f32, f32) {
        self.domain_max
    }

    pub fn data(&self) -> &[(f32, f32, f32)] {
        &self.data
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
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
    pub fn new(
        title: String,
        size: u32,
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
        // TODO: Convert the parsing errors?
        let result = parser::keywords_data(text);
        println!("{:?}", result);
        let (_, (keywords, data)) = result.ok().ok_or("Failed to parse CUBE.")?;

        let mut title: String = "".into();
        let mut size: Option<u32> = Option::None;

        // Use the default values if not specified.
        let mut domain_min = (0f32, 0f32, 0f32);
        let mut domain_max = (1f32, 1f32, 1f32);

        for keyword in keywords {
            match keyword {
                KeyWord::Title(value) => title = value,
                KeyWord::DomainMin(r, g, b) => domain_min = (r, g, b),
                KeyWord::DomainMax(r, g, b) => domain_max = (r, g, b),
                KeyWord::Lut3dSize(value) => size = Some(value),
            }
        }

        let size = size.ok_or("Failed to parse LUT_3D_SIZE.")?;

        if data.len() != (size as usize).pow(3) {
            return Err("Data point count does not agree with LUT_3D_SIZE.");
        }

        // TODO: Make sure the size and the actual data length match.
        // TODO: Size must be greater than 2.
        let cube = CubeLut3d::new(title, size, domain_min, domain_max, data);
        Ok(cube)
    }
}

mod parser {
    use std::str::FromStr;

    // .cube is a text based format for 1D or 3D LUTs.
    // The format specification can be found at the link below.
    // https://wwwimages2.adobe.com/content/dam/acom/en/products/speedgrade/cc/pdfs/cube-lut-specification-1.0.pdf
    use nom::{
        branch::alt,
        bytes::complete::{tag, take_until, take_while},
        character::complete::{char, digit1, space1},
        combinator::map_res,
        multi::{many0, separated_list1},
        number::complete::float,
        sequence::{delimited, preceded, tuple},
        IResult,
    };

    #[derive(Debug, PartialEq)]
    pub enum KeyWord {
        DomainMin(f32, f32, f32),
        DomainMax(f32, f32, f32),
        Title(String),
        Lut3dSize(u32),
    }

    pub fn keywords_data(input: &str) -> IResult<&str, (Vec<KeyWord>, Vec<(f32, f32, f32)>)> {
        let (input, (keywords, data)) = tuple((keywords, data_domain))(input)?;
        Ok((input, (keywords, data)))
    }

    // TODO: How to make this return a function?
    fn line_separated<T, F: Fn(&str) -> IResult<&str, T>>(
        input: &str,
        parse: F,
    ) -> IResult<&str, Vec<T>> {
        // Handle leading and trailing newlines for each line.
        // \nline1\n\nline2\nline3\n
        let (input, (_, _, result)) = tuple((
            end_of_line,
            many0(comment_line),
            separated_list1(end_of_line, parse),
        ))(input)?;
        Ok((input, result))
    }

    fn data_domain(input: &str) -> IResult<&str, Vec<(f32, f32, f32)>> {
        // 1 1 1
        // 0 0 0
        // 0 1 0
        line_separated(input, rgb_color)
    }

    fn keywords(input: &str) -> IResult<&str, Vec<KeyWord>> {
        // Keywords appear in any order, so collect them all.
        line_separated(input, keyword)
    }

    fn keyword(input: &str) -> IResult<&str, KeyWord> {
        // Check for any supported keyword.
        alt((domain_min, domain_max, title, lut_3d_size))(input)
    }

    fn end_of_line(input: &str) -> IResult<&str, &str> {
        // \r\n isn't part of the spec but should be supported just in case.
        take_while(|c| c == '\n' || c == '\r')(input)
    }

    fn not_end_of_line(input: &str) -> IResult<&str, &str> {
        // \r\n isn't part of the spec but should be supported just in case.
        take_while(|c| c != '\n' && c != '\r')(input)
    }

    fn sp(input: &str) -> IResult<&str, &str> {
        space1(input)
    }

    fn title_text(input: &str) -> IResult<&str, &str> {
        // TITLE "title"
        delimited(char('"'), take_until("\""), char('"'))(input)
    }

    fn rgb_color(input: &str) -> IResult<&str, (f32, f32, f32)> {
        // 0.85 1.0 1.0
        let (input, (r, _, g, _, b)) = tuple((float, sp, float, sp, float))(input)?;
        Ok((input, (r, g, b)))
    }

    fn domain_min(input: &str) -> IResult<&str, KeyWord> {
        // DOMAIN_MIN 0 0 0
        let (input, (_, _, (r, g, b))) = tuple((tag("DOMAIN_MIN"), sp, rgb_color))(input)?;
        Ok((input, KeyWord::DomainMin(r, g, b)))
    }

    fn domain_max(input: &str) -> IResult<&str, KeyWord> {
        // DOMAIN_MAX 1 1 1
        let (input, (_, _, (r, g, b))) = tuple((tag("DOMAIN_MAX"), sp, rgb_color))(input)?;
        Ok((input, KeyWord::DomainMax(r, g, b)))
    }

    fn comment_line(input: &str) -> IResult<&str, &str> {
        // # this is a comment\r\n
        let (input, (comment, _)) = tuple((comment, end_of_line))(input)?;
        Ok((input, comment))
    }

    fn comment(input: &str) -> IResult<&str, &str> {
        // # this is a comment
        delimited(tag("#"), not_end_of_line, end_of_line)(input)
    }

    fn uint32(input: &str) -> IResult<&str, u32> {
        map_res(digit1, FromStr::from_str)(input)
    }

    fn lut_3d_size(input: &str) -> IResult<&str, KeyWord> {
        // LUT_3D_SIZE 8
        let (input, (_, _, size)) = tuple((tag("LUT_3D_SIZE"), sp, uint32))(input)?;
        Ok((input, KeyWord::Lut3dSize(size)))
    }

    fn title(input: &str) -> IResult<&str, KeyWord> {
        // "title"
        let (input, (_, _, title)) = tuple((tag("TITLE"), sp, title_text))(input)?;
        Ok((input, KeyWord::Title(title.into())))
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use indoc::indoc;

        #[test]
        fn parse_data_domain() {
            let text = indoc! {r#"
            #data domain
            0 0 0
            1 0 0
            0 .75 0
            1 .75 0


            0 .25 1
            1 .25 1
            0 1 1

            1 1 0.5
        "#};
            let result = data_domain(&text).unwrap().1;
            assert_eq!(
                vec![
                    (0f32, 0f32, 0f32),
                    (1f32, 0f32, 0f32),
                    (0f32, 0.75f32, 0f32),
                    (1f32, 0.75f32, 0f32),
                    (0f32, 0.25f32, 1f32),
                    (1f32, 0.25f32, 1f32),
                    (0f32, 1f32, 1f32),
                    (1f32, 1f32, 0.5f32)
                ],
                result
            );
        }

        #[test]
        fn parse_all_keywords() {
            let text = indoc! {r#"

            DOMAIN_MIN -1 -1 -1
            LUT_3D_SIZE 2

            DOMAIN_MAX 1 2 3


            TITLE "lut1"


            0 0 0
        "#};
            let result = keywords(&text).unwrap().1;
            assert_eq!(4, result.len());
            assert_eq!(KeyWord::DomainMin(-1f32, -1f32, -1f32), result[0]);
            assert_eq!(KeyWord::Lut3dSize(2), result[1]);
            assert_eq!(KeyWord::DomainMax(1f32, 2f32, 3f32), result[2]);
            assert_eq!(KeyWord::Title("lut1".into()), result[3]);
        }

        #[test]
        fn parse_keywords_and_data() {
            let text = "#Created by: smush_lut.exe\nTITLE \"cube\"\n\n#LUT Size\nLUT_3D_SIZE 2\n\n#data domain\nDOMAIN_MIN 0.0 0.0 0.0\nDOMAIN_MAX 1.0 1.0 1.0\n\n#LUT data points\n0.5 0.5 0.5\n0.5 0.5 0.5\n0.5 0.5 0.5\n0.5 0.5 0.5\n0.5 0.5 0.5\n0.5 0.5 0.5\n0.5 0.5 0.5\n0.5 0.5 0.5\n";

            // Make sure the keywords were parsed.
            let result = keywords_data(&text).unwrap().1;
            assert_eq!(4, result.0.len());
            assert_eq!(KeyWord::Lut3dSize(2), result.0[0]);
            assert_eq!(KeyWord::DomainMin(0f32, 0f32, 0f32), result.0[1]);
            assert_eq!(KeyWord::DomainMax(1f32, 1f32, 1f32), result.0[2]);
            assert_eq!(KeyWord::Title("lut1".into()), result.0[3]);

            // The data section comes after all keyword lines.
            assert_eq!(
                vec![
                    (0.5f32, 0.5f32, 0.5f32),
                    (0.5f32, 0.5f32, 0.5f32),
                    (0.5f32, 0.5f32, 0.5f32),
                    (0.5f32, 0.5f32, 0.5f32),
                    (0.5f32, 0.5f32, 0.5f32),
                    (0.5f32, 0.5f32, 0.5f32),
                    (0.5f32, 0.5f32, 0.5f32),
                    (0.5f32, 0.5f32, 0.5f32)
                ],
                result.1
            );
        }

        #[test]
        fn parse_keyword() {
            assert_eq!(
                KeyWord::DomainMin(1f32, 2f32, 3f32),
                keyword("DOMAIN_MIN 1 2 3").unwrap().1
            );
            assert_eq!(
                KeyWord::DomainMax(1f32, 2f32, 3f32),
                keyword("DOMAIN_MAX 1 2 3").unwrap().1
            );
            assert_eq!(
                KeyWord::Title("abc".into()),
                keyword("TITLE \"abc\"").unwrap().1
            );
            assert_eq!(KeyWord::Lut3dSize(8), keyword("LUT_3D_SIZE 8").unwrap().1);
        }

        #[test]
        #[should_panic]
        fn parse_invalid_keyword() {
            keyword("1.0 2.0 3.0").unwrap();
        }

        #[test]
        fn parse_end_of_line() {
            assert_eq!("abc ", end_of_line("\r\nabc ").unwrap().0);
            assert_eq!("a b c", end_of_line("\na b c").unwrap().0);
            assert_eq!("a b c", end_of_line("\n\na b c").unwrap().0);
            assert_eq!("a b c", end_of_line("\n\n\r\r\na b c").unwrap().0);
        }

        #[test]
        fn parse_sp() {
            assert_eq!("  \t\t ", sp("  \t\t \n").unwrap().1);
        }

        #[test]
        fn parse_comment_line() {
            assert_eq!(
                ("rest of line", " this is a comment"),
                comment("# this is a comment\r\nrest of line").unwrap()
            );
        }

        #[test]
        fn parse_comment() {
            assert_eq!(
                " this is a comment",
                comment("# this is a comment").unwrap().1
            );
        }

        #[test]
        fn parse_domain_min() {
            assert_eq!(
                KeyWord::DomainMin(1f32, 2f32, 3f32),
                domain_min("DOMAIN_MIN 1 2 3").unwrap().1
            );
        }

        #[test]
        fn parse_lut_3d_size() {
            assert_eq!(
                KeyWord::Lut3dSize(8),
                lut_3d_size("LUT_3D_SIZE 8").unwrap().1
            );
            assert_eq!(
                KeyWord::Lut3dSize(65536),
                lut_3d_size("LUT_3D_SIZE 65536").unwrap().1
            );
        }

        #[test]
        fn parse_domain_max() {
            assert_eq!(
                KeyWord::DomainMax(1f32, 2f32, 3f32),
                domain_max("DOMAIN_MAX 1 2 3").unwrap().1
            );
        }

        #[test]
        fn parse_title_text() {
            assert_eq!("title", title_text("\"title\"").unwrap().1);
            assert_eq!("ab cd ", title_text("\"ab cd \"").unwrap().1);
        }

        #[test]
        fn parse_rgb_color() {
            assert_eq!((0f32, 0f32, 0f32), rgb_color("0 0 0").unwrap().1);
            assert_eq!(
                (0.5f32, 1.5f32, 2.5f32),
                rgb_color("0.5\t1.5\t2.5").unwrap().1
            );
            assert_eq!(
                (1e37f32, 1e37f32, -1e37f32),
                rgb_color("1e37 \t\t 1E37   -1e37").unwrap().1
            );
        }

        #[test]
        fn parse_title() {
            assert_eq!(
                KeyWord::Title("title".into()),
                title("TITLE \"title\"").unwrap().1
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
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
        let linear = Lut3dLinear::new(
            2u32,
            [0u8, 51u8, 255u8, 255u8]
                .iter()
                .cycle()
                .take(32)
                .map(|u| *u)
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
        let linear = Lut3dLinear::new(
            2u32,
            [0u8, 51u8, 255u8, 255u8]
                .iter()
                .cycle()
                .take(32)
                .map(|u| *u)
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
        assert_eq!(cube, Err("Failed to parse CUBE."));
    }

    #[test]
    fn create_from_text_no_data() {
        let text = indoc! {r#"
            TITLE "no data"
            LUT_3D_SIZE 2
        "#};
        let cube = CubeLut3d::from_text(text);
        assert_eq!(cube, Err("Failed to parse CUBE."));
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
        assert_eq!(cube, Err("Failed to parse CUBE."));
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
        assert_eq!(cube, Err("Failed to parse CUBE."));
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
        assert_eq!(cube, Err("Failed to parse CUBE."));
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
        println!("{:?}", text);
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
