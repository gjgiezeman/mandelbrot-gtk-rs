use crate::mandel_image::{make_mandel_image, Mapping};
use gtk::{cairo::ImageSurface, glib::WeakRef, prelude::*, DrawingArea};

use super::WIN_SZ0;

pub struct State {
    mapping: Mapping,
    img: Option<ImageSurface>,
    canvas: WeakRef<DrawingArea>,
}

impl State {
    pub fn new() -> State {
        State {
            mapping: Mapping::new_for_size(WIN_SZ0),
            img: None,
            canvas: WeakRef::new(),
        }
    }
    pub fn img(&self) -> &Option<ImageSurface> {
        &self.img
    }
    pub fn set_img(&mut self, img: ImageSurface) {
        self.img = Some(img);
        if let Some(canvas) = self.canvas.upgrade() {
            canvas.queue_draw();
        }
    }
    pub fn set_canvas(&mut self, canvas: WeakRef<DrawingArea>) {
        self.canvas = canvas;
    }
    pub fn on_resize(&mut self, w: i32, h: i32) {
        self.mapping.win_width = w as usize;
        self.mapping.win_height = h as usize;
        self.recompute_image();
    }
    pub fn cx(&self) -> f64 {
        self.mapping.cx
    }
    pub fn set_cx(&mut self, v_opt: Option<f64>) {
        if let Some(value) = v_opt {
            self.mapping.cx = value;
            self.recompute_image();
        }
    }
    pub fn cy(&self) -> f64 {
        self.mapping.cy
    }
    pub fn set_cy(&mut self, v_opt: Option<f64>) {
        if let Some(value) = v_opt {
            self.mapping.cy = value;
            self.recompute_image();
        }
    }
    pub fn set_zoom(&mut self, zoom: f64) {
        // The value is chosen such that floating point approximation becomes clear near zoom == 1000
        let scale = 1.035_f64.powf(-zoom);
        self.mapping.scale = 4.0 * scale / WIN_SZ0 as f64;
        self.recompute_image();
    }
    pub fn set_iter_depth(&mut self, value: f64) {
        let iter_depth = value as u32;
        self.mapping.iteration_depth = iter_depth;
        self.recompute_image();
    }
    pub fn iter_depth(&self) -> f64 {
        self.mapping.iteration_depth as f64
    }
    fn recompute_image(&mut self) {
        if let Some(img) = make_mandel_image(&self.mapping) {
            self.set_img(img);
        }
    }
}
