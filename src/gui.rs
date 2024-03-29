use std::cell::RefCell;
use std::rc::Rc;

use crate::mandel_image::{make_mandel_image, Mapping};
use gtk::cairo::ImageSurface;
use gtk::{glib, prelude::*, Application, ApplicationWindow, DrawingArea};

const APP_ID: &str = "nl.uu.gjgiezeman.mandelbrot";
const WIN_SZ0: usize = 600;

struct State {
    mapping: Mapping,
    img: Option<ImageSurface>,
}

impl State {
    fn new() -> State {
        State {
            mapping: Mapping::new_for_size(WIN_SZ0),
            img: None,
        }
    }
}

fn mandel_draw(state: &Rc<RefCell<State>>, ctxt: &gtk::cairo::Context) {
    if let Some(img) = &state.borrow().img {
        ctxt.set_source_surface(img, 0.0, 0.0)
            .expect("Expected to be able to set source surface");
        ctxt.paint().unwrap();
    }
}

fn on_resize(state: &Rc<RefCell<State>>, w: i32, h: i32) {
    let mut s = state.borrow_mut();
    s.mapping.win_width = w as usize;
    s.mapping.win_height = h as usize;
    s.img = make_mandel_image(&s.mapping);
}

fn build_ui(app: &Application) {
    let state = Rc::new(RefCell::new(State::new()));
    let canvas = DrawingArea::builder()
        .content_height(WIN_SZ0 as i32)
        .content_width(WIN_SZ0 as i32)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .build();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Mandelbrot")
        .child(&canvas)
        .build();

    let state2 = state.clone();
    canvas.set_draw_func(move |_d, ctxt, _w, _h| mandel_draw(&state2, ctxt));
    canvas.connect_resize(move |_da, w, h| on_resize(&state, w, h));

    window.present();
}

pub fn run() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}
