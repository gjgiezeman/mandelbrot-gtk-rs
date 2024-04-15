use crate::mandel_image::{make_mandel_image, Mapping};
use gtk::cairo::ImageSurface;

use super::WIN_SZ0;

pub struct State {
    mapping: Mapping,
    img: Option<ImageSurface>,
}

impl State {
    pub fn new() -> State {
        State {
            mapping: Mapping::new_for_size(WIN_SZ0),
            img: None,
        }
    }
    pub fn img(&self) -> &Option<ImageSurface> {
        &self.img
    }
    pub fn on_resize(&mut self, w: i32, h: i32) {
        self.mapping.win_width = w as usize;
        self.mapping.win_height = h as usize;
        self.img = make_mandel_image(&self.mapping);
    }
}
