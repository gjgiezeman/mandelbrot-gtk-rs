mod state;

use self::state::State;
use gtk::gdk::ffi::GDK_BUTTON_PRIMARY;
use gtk::glib::clone;
use gtk::{
    glib, prelude::*, Adjustment, Application, ApplicationWindow, DrawingArea, GestureClick, Label,
    Orientation, Scale, SpinButton,
};
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "nl.uu.gjgiezeman.mandelbrot";
const WIN_SZ0: usize = 600;

fn mandel_draw(state: &Rc<RefCell<State>>, ctxt: &gtk::cairo::Context) {
    if let Some(img) = &state.borrow().img() {
        ctxt.set_source_surface(img, 0.0, 0.0)
            .expect("Expected to be able to set source surface");
        ctxt.paint().unwrap();
    }
}

fn expect_float_value(e: &gtk::Entry) -> Option<f64> {
    let t = e.text();
    if let Ok(value) = t.parse::<f64>() {
        Some(value)
    } else {
        None
    }
}

fn on_clicked(
    state: &Rc<RefCell<State>>,
    gesture: &GestureClick,
    wx: f64,
    wy: f64,
    cx_value: &gtk::Entry,
    cy_value: &gtk::Entry,
) {
    gesture.set_state(gtk::EventSequenceState::Claimed);
    let (new_cx, new_cy) = state.borrow().win_to_mandel(wx, wy);
    cx_value.set_text(&new_cx.to_string());
    cy_value.set_text(&new_cy.to_string());
}

fn make_row_box() -> gtk::Box {
    gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(5)
        .build()
}

fn build_ui(app: &Application) {
    let state = Rc::new(RefCell::new(State::new()));
    let iter_val = state.borrow().iter_depth();
    let iter_adj = Adjustment::new(iter_val, 10.0, 1000.0, 1.0, 0.0, 0.0);
    let iteration_button = SpinButton::builder().adjustment(&iter_adj).build();
    let first_row = make_row_box();
    first_row.append(&Label::new(Some("max iterations:")));
    first_row.append(&iteration_button);
    let cx_value = gtk::Entry::builder()
        .text(&state.borrow().cx().to_string())
        .width_chars(15)
        .margin_end(10)
        .build();
    let cy_value = gtk::Entry::builder()
        .text(&state.borrow().cy().to_string())
        .width_chars(15)
        .build();
    let second_row = make_row_box();
    second_row.append(&Label::new(Some("center x:")));
    second_row.append(&cx_value);
    second_row.append(&Label::new(Some("center y:")));
    second_row.append(&cy_value);
    let zoom_adj = Adjustment::new(0.0, 0.0, 1000.0, 1.0, 0.0, 0.0);
    let zoom_bar = Scale::new(Orientation::Horizontal, Some(&zoom_adj));
    zoom_bar.set_hexpand(true);
    let third_row = make_row_box();
    third_row.append(&Label::new(Some("zoom:")));
    third_row.append(&zoom_bar);
    let canvas = DrawingArea::builder()
        .content_height(WIN_SZ0 as i32)
        .content_width(WIN_SZ0 as i32)
        .vexpand(true)
        .build();
    state.borrow_mut().set_canvas(canvas.downgrade());
    let content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(5)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .build();
    content_box.append(&first_row);
    content_box.append(&second_row);
    content_box.append(&third_row);
    content_box.append(&canvas);
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Mandelbrot")
        .child(&content_box)
        .build();

    // Set actions
    canvas.set_draw_func(clone!(@strong state =>move |_d, ctxt, _w, _h| mandel_draw(&state, ctxt)));
    iter_adj.connect_value_changed(clone!(@strong state => move |a| {
        state.borrow_mut().set_iter_depth(a.value());
    }));
    cx_value.connect_changed(
        clone!(@strong state => move |e| { state.borrow_mut().set_cx(expect_float_value(e));}),
    );
    cy_value.connect_changed(
        clone!(@strong state => move |e| { state.borrow_mut().set_cy(expect_float_value(e));}),
    );
    let gesture = gtk::GestureClick::new();
    gesture.set_button(GDK_BUTTON_PRIMARY as u32);
    gesture.connect_pressed(clone!(@strong state => move |gesture, _, wx, wy| on_clicked(&state, gesture, wx, wy, &cx_value, &cy_value)));
    canvas.add_controller(gesture);
    zoom_adj.connect_value_changed(clone!(@strong state => move |adj| {
        state.borrow_mut().set_zoom(adj.value());
    }));
    canvas.connect_resize(
        clone!(@strong state => move |_da, w, h| state.borrow_mut().on_resize(w, h)),
    );
    window.present();
}

pub fn run() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}
