pub mod colorings;
pub mod gui;
pub mod image;
pub mod mandel_image;
pub mod presets;

use colorings::ColorFromMandel;
use mandel_image::Mapping;

const IMG_FMT: gtk::cairo::Format = gtk::cairo::Format::Rgb24;

pub struct MandelReq {
    params: Mapping,
    coloring: Box<dyn ColorFromMandel>,
}

pub struct MandelReply {
    result: Option<Vec<u8>>,
    width: i32,
    height: i32,
    stride: i32,
}
