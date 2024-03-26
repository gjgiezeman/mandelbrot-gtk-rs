use dyn_clone::DynClone;
pub trait ColorFromMandel: DynClone + Sync + Send {
    /// Get a color in GTK RGB-format, given the mandelbrot value
    /// and the maximum mandelbrot value
    fn get(&self, v: u32, max: u32) -> u32;
}

dyn_clone::clone_trait_object!(ColorFromMandel);

#[derive(Clone)]
struct Rgb18 {}

impl ColorFromMandel for Rgb18 {
    fn get(&self, v: u32, max: u32) -> u32 {
        if max <= v {
            return 0x000000;
        }
        match v % 18 {
            0 => 0xff3f3f,
            1 => 0xff7f3f,
            2 => 0xffbf3f,
            3 => 0xffff3f,
            4 => 0xbfff3f,
            5 => 0x7fff3f,
            6 => 0x3fff3f,
            7 => 0x3fff7f,
            8 => 0x3fffbf,
            9 => 0x3fffff,
            10 => 0x3fbfff,
            11 => 0x3f7fff,
            12 => 0x3f3fff,
            13 => 0x7f3fff,
            14 => 0xbf3fff,
            15 => 0xff3fff,
            16 => 0xff3fbf,
            17 => 0xff3f7f,
            _ => panic!("invalid remainder"),
        }
    }
}

#[derive(Clone)]
struct RgbAlternating {}

impl ColorFromMandel for RgbAlternating {
    fn get(&self, v: u32, max: u32) -> u32 {
        if max <= v {
            return 0x000000;
        }
        match v % 3 {
            0 => 0xff0000,
            1 => 0x00ff00,
            2 => 0x0000ff,
            _ => panic!("invalid remainder"),
        }
    }
}

#[derive(Clone)]
struct RedBlack {}

impl ColorFromMandel for RedBlack {
    fn get(&self, v: u32, _max: u32) -> u32 {
        if v % 2 == 1 {
            0xff0000
        } else {
            0x0
        }
    }
}

fn all_color_from_mandels() -> Vec<Box<dyn ColorFromMandel>> {
    vec![
        Box::new(Rgb18 {}),
        Box::new(RgbAlternating {}),
        Box::new(RedBlack {}),
    ]
}

pub struct ColorInfo {
    names: [&'static str; 3],
    producers: Vec<Box<dyn ColorFromMandel>>,
}

impl ColorInfo {
    pub fn new() -> ColorInfo {
        let names = ["rgb18", "rgb", "red-black"];
        let producers = all_color_from_mandels();
        assert_eq!(names.len(), producers.len());
        ColorInfo { names, producers }
    }
    pub fn color_names(&self) -> &[&str] {
        self.names.as_slice()
    }
    pub fn len(&self) -> usize {
        self.names.len()
    }
    pub fn producer(&self, i: usize) -> Box<dyn ColorFromMandel> {
        assert!(i < self.len());
        self.producers[i].clone()
    }
}
