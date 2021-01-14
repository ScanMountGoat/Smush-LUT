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
        let mut data = Vec::new();

        // TODO: Account for whitespace.
        let size_header = text.lines().next().unwrap();
        let size: u32 = size_header.split_whitespace().skip(1).next().unwrap().parse().unwrap();

        for line in text.lines().skip(1) {
            // TODO: Handle errors and return a result;
            for symbol in line.split_whitespace() {
                println!("{}", symbol);
                let value: f32 = symbol.parse().unwrap();
                data.push(value);
            }
        }

        // TODO: Make sure the size and the actual data length match.
        CubeLut3d::new("".into(), size, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_from_text_size2() {
        let text = r"LUT_3D_SIZE 2
0 0 0
1 0 0
0 .75 0
1 .75 0
0 .25 1
1 .25 1
0 1 1
1 1 1
";
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
}
