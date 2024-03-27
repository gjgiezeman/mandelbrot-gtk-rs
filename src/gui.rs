use crate::mandel_image::{make_mandel_image, Mapping};
use gtk::cairo::ImageSurface;
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

fn set_image(state: &mut State) {
    let img = make_mandel_image(&state.mparams);
    state.img = img;
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

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Mandelbrot")
        .child(&canvas)
        .build();
    set_image(&mut state.borrow_mut());
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
