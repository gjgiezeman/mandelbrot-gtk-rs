use std::{cell::RefCell, rc::Rc};

use async_channel::Sender;
use gtk::{glib::WeakRef, prelude::*, DrawingArea};

use crate::{
    colorings::ColorInfo,
    image::Image,
    mandel_image::{Mapping, WinToMandel},
    MandelReq,
};

use super::WIN_SZ0;

pub struct State {
    mapping: Mapping,
    img: Option<Image>,
    col_idx: usize,
    color_info: ColorInfo,
    preset: Option<u8>,
    req_sender: Sender<MandelReq>,
    canvas: WeakRef<DrawingArea>,
    block: bool,
}

impl State {
    pub fn new(req_sender: Sender<MandelReq>) -> State {
        State {
            mapping: Mapping::new_for_size(WIN_SZ0),
            img: None,
            col_idx: 0,
            color_info: ColorInfo::new(),
            preset: None,
            req_sender,
            canvas: WeakRef::new(),
            block: false,
        }
    }
    pub fn coloring_names(&self) -> Vec<&str> {
        self.color_info.names_iter().collect()
    }
    pub fn win_to_mandel(&self, wx: f64, wy: f64) -> (f64, f64) {
        WinToMandel::from_mapping(&self.mapping).cvt(wx as usize, wy as usize)
    }
    pub fn img(&self) -> &Option<Image> {
        &self.img
    }
    pub fn set_img(&mut self, img: Image) {
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
    pub fn set_col_idx(&mut self, col_idx: usize) {
        self.col_idx = col_idx;
        self.recompute_image();
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
    pub fn set_preset(&mut self, preset: Option<u8>) {
        self.preset = preset;
    }
    pub fn take_preset(&mut self) -> Option<u8> {
        self.preset.take()
    }
    fn recompute_image(&mut self) {
        if self.block {
            return;
        }
        let coloring = self.color_info.scheme(self.col_idx).clone();
        let request = MandelReq {
            mapping: self.mapping.clone(),
            coloring,
        };
        let _ = self.req_sender.send_blocking(request);
    }
}

#[must_use = "if unused would redraw without postponing anything"]
pub fn postpone_redraw(state: &Rc<RefCell<State>>) -> PostponedRedraw {
    state.borrow_mut().block = true;
    PostponedRedraw {
        state: state.clone(),
    }
}

#[clippy::has_significant_drop]
pub struct PostponedRedraw {
    state: Rc<RefCell<State>>,
}

impl Drop for PostponedRedraw {
    fn drop(&mut self) {
        let mut state = self.state.borrow_mut();
        state.block = false;
        state.recompute_image();
    }
}
