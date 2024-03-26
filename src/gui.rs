use crate::mandel_image::{make_mandel_image, Mapping};
use gtk::cairo::ImageSurface;
use gtk::glib::clone;
use gtk::{glib, prelude::*, Application, ApplicationWindow, DrawingArea};
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "nl.uu.gjgiezeman.mandelbrot";
const WIN_SZ0: usize = 600;

struct State {
    mparams: Mapping,
    img: Option<ImageSurface>,
}

impl State {
    fn new() -> State {
        State {
            mparams: Mapping::new_for_size(WIN_SZ0),
            img: None,
        }
    }
}

fn mandel_draw(
    state: &Rc<RefCell<State>>,
    _da: &DrawingArea,
    ctxt: &gtk::cairo::Context,
    _w: i32,
    _h: i32,
) {
    if let Some(img) = &state.borrow().img {
        ctxt.set_source_surface(img, 0.0, 0.0)
            .expect("Expected to be able to set source surface");
        ctxt.paint().unwrap();
    }
}

fn recompute_image(state: &mut State) {
    let img = make_mandel_image(&state.mparams);
    state.img = img;
}

fn on_resize(state: &Rc<RefCell<State>>, _da: &DrawingArea, w: i32, h: i32) {
    {
        let mut s = state.borrow_mut();
        s.mparams.win_width = w as usize;
        s.mparams.win_height = h as usize;
        recompute_image(&mut s);
    }
}

fn build_ui(app: &Application) {
    let state = Rc::new(RefCell::new(State::new()));

    let canvas = DrawingArea::builder()
        .content_height(WIN_SZ0 as i32)
        .content_width(WIN_SZ0 as i32)
        .vexpand(true)
        .can_target(true)
        .build();
    {
        let state = state.clone();
        canvas.set_draw_func(move |d, c, w, h| mandel_draw(&state, d, c, w, h));
    }

    let content_box = gtk::Frame::builder()
        .child(&canvas)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .build();

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Mandelbrot")
        .child(&content_box)
        .build();

    // Add the actions to the widgets

    canvas.connect_resize(clone!(@strong state => move |d, w, h| on_resize(&state, d, w, h)));

    window.present();
}

pub fn run() -> glib::ExitCode {
    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();
    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);
    // Run the application
    app.run()
}
