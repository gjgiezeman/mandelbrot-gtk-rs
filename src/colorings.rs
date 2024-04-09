pub trait Coloring {
    /// Get a color in GTK RGB-format, given the mandelbrot value
    /// and the maximum mandelbrot value
    fn get_color(&self, v: u32, max: u32) -> u32;
    /// Get a name for the coloring scheme, suitable for use in the UI
    fn name(&self) -> &str;
}

#[derive(Clone)]
struct Rgb18 {}

impl Coloring for Rgb18 {
    fn get_color(&self, v: u32, max: u32) -> u32 {
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

    fn name(&self) -> &'static str {
        "rgb18"
    }
}

#[derive(Clone)]
struct RedBlue {}

impl Coloring for RedBlue {
    fn get_color(&self, v: u32, max: u32) -> u32 {
        if max <= v {
            return 0x404040;
        }
        match v % 16 {
            0 => 0x000000,
            1 => 0x400000,
            2 => 0x800000,
            3 => 0xc00000,
            4 => 0xff0000,
            5 => 0xff0040,
            6 => 0xff0080,
            7 => 0xff00c0,
            8 => 0xff00ff,
            9 => 0xc000ff,
            10 => 0x8000ff,
            11 => 0x4000ff,
            12 => 0x0000ff,
            13 => 0x0000c0,
            14 => 0x000080,
            15 => 0x000040,
            _ => panic!("invalid remainder"),
        }
    }

    fn name(&self) -> &str {
        "red-blue16"
    }
}

#[derive(Clone)]
struct RgbAlternating {}

impl Coloring for RgbAlternating {
    fn get_color(&self, v: u32, max: u32) -> u32 {
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

    fn name(&self) -> &str {
        "rgb3"
    }
}
#[derive(Clone)]
struct BlackWhite {}

impl Coloring for BlackWhite {
    fn get_color(&self, v: u32, max: u32) -> u32 {
        if v == max {
            0x808080
        } else {
            if v % 2 == 1 {
                0xffffff
            } else {
                0x0
            }
        }
    }

    fn name(&self) -> &str {
        "black-white"
    }
}

#[derive(Clone)]
struct OldBlackWhite {}

impl Coloring for OldBlackWhite {
    fn get_color(&self, v: u32, _max: u32) -> u32 {
        if v % 2 == 1 {
            0xffffff
        } else {
            0x0
        }
    }

    fn name(&self) -> &str {
        "old-bw"
    }
}

fn all_colorings() -> Vec<Box<dyn Coloring>> {
    vec![
        Box::new(Rgb18 {}),
        Box::new(RgbAlternating {}),
        Box::new(RedBlue {}),
        Box::new(BlackWhite {}),
        Box::new(OldBlackWhite {}),
    ]
}

pub struct ColorInfo {
    colorings: Vec<Box<dyn Coloring>>,
}

pub struct NameIter<'a> {
    iter: std::slice::Iter<'a, Box<dyn Coloring>>,
}

impl<'a> Iterator for NameIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().and_then(|clr| Some(clr.name()))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl ColorInfo {
    pub fn new() -> ColorInfo {
        ColorInfo {
            colorings: all_colorings(),
        }
    }

    pub fn len(&self) -> usize {
        self.colorings.len()
    }
    pub fn scheme(&self, i: usize) -> &Box<dyn Coloring> {
        &self.colorings[i]
    }
    pub fn names_iter(&self) -> NameIter {
        NameIter {
            iter: self.colorings.iter(),
        }
    }
}
