pub mod colorings;
pub mod gui;
pub mod mandel_image;
pub mod presets;

use colorings::ColorFromMandel;
use mandel_image::Mapping;

const IMG_FMT: gtk::cairo::Format = gtk::cairo::Format::Rgb24;

pub struct MandelReq {
    params: Mapping,
    coloring: Box<dyn ColorFromMandel>,
}
