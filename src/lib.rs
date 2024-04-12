use colorings::Coloring;
use mandel_image::Mapping;

pub mod colorings;
pub mod gui;
pub mod image;
pub mod mandel_image;
pub mod presets;

const IMG_FMT: gtk::cairo::Format = gtk::cairo::Format::Rgb24;

pub struct MandelReq {
    mapping: Mapping,
    coloring: Box<dyn Coloring>,
}

pub struct MandelReply {
    data: Vec<u8>,
    width: i32,
    height: i32,
    stride: i32,
}
