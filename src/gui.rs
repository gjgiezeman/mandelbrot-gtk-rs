use std::cell::RefCell;
use std::rc::Rc;

use crate::colorings::ColorInfo;
use crate::mandel_image::{make_mandel_image, Mapping, WinToMandel};
use gtk::cairo::ImageSurface;
use gtk::ffi::GTK_INVALID_LIST_POSITION;
use gtk::gdk::ffi::GDK_BUTTON_PRIMARY;
use gtk::glib::{clone, WeakRef};
use gtk::{
    glib, prelude::*, Adjustment, Application, ApplicationWindow, DrawingArea, DropDown, Label,
    Orientation, Scale, SpinButton,
};

const APP_ID: &str = "nl.uu.gjgiezeman.mandelbrot";
const WIN_SZ0: usize = 600;

struct State {
    mapping: Mapping,
    img: Option<ImageSurface>,
    col_idx: usize,
    color_info: ColorInfo,
    canvas: WeakRef<DrawingArea>,
}

impl State {
    fn new() -> State {
        State {
            mapping: Mapping::new_for_size(WIN_SZ0),
            img: None,
            col_idx: 0,
            color_info: ColorInfo::new(),
            canvas: WeakRef::new(),
        }
    }
    fn win_to_mandel(&self, wx: f64, wy: f64) -> (f64, f64) {
        WinToMandel::from_mapping(&self.mapping).cvt(wx as usize, wy as usize)
    }
}

fn mandel_draw(state: &Rc<RefCell<State>>, ctxt: &gtk::cairo::Context) {
    if let Some(img) = &state.borrow().img {
        ctxt.set_source_surface(img, 0.0, 0.0)
            .expect("Expected to be able to set source surface");
        ctxt.paint().unwrap();
    }
}

fn recompute_image(state: &mut State) {
    let coloring = state.color_info.scheme(state.col_idx);
    state.img = make_mandel_image(&state.mapping, coloring);
    if let Some(canvas) = state.canvas.upgrade() {
        canvas.queue_draw();
    }
}

fn on_resize(state: &Rc<RefCell<State>>, w: i32, h: i32) {
    let mut s = state.borrow_mut();
    s.mapping.win_width = w as usize;
    s.mapping.win_height = h as usize;
    recompute_image(&mut s);
}

fn iter_depth_changed(state: &mut State, adj: &Adjustment) {
    let iter_depth = adj.value() as u32;
    state.mapping.iteration_depth = iter_depth;
    recompute_image(state);
}

fn expect_float_value(e: &gtk::Entry) -> Option<f64> {
    let t = e.text();
    if let Ok(value) = t.parse::<f64>() {
        Some(value)
    } else {
        None
    }
}

fn cx_changed(state: &mut State, e: &gtk::Entry) {
    if let Some(value) = expect_float_value(e) {
        state.mapping.cx = value;
        recompute_image(state);
    }
}

fn cy_changed(state: &mut State, e: &gtk::Entry) {
    if let Some(value) = expect_float_value(e) {
        state.mapping.cy = value;
        recompute_image(state);
    }
}

fn col_changed(state: &mut State, dd: &DropDown) {
    let sel = dd.selected();
    if sel != GTK_INVALID_LIST_POSITION {
        state.col_idx = sel as usize;
        recompute_image(state);
    }
}

fn zoom_changed(state: &mut State, adj: &Adjustment) {
    let zoom = adj.value();
    // The value is chosen such that floating point approximation becomes clear near zoom == 1000
    let scale = 1.035_f64.powf(-zoom);
    state.mapping.scale = 4.0 * scale / WIN_SZ0 as f64;
    recompute_image(state);
}

fn make_row_box() -> gtk::Box {
    gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(5)
        .build()
}

fn build_ui(app: &Application) {
    let state = Rc::new(RefCell::new(State::new()));
    let colorings;
    {
        let state = state.borrow();
        let names: Vec<&str> = state.color_info.names_iter().collect();
        colorings = DropDown::from_strings(&names);
    }
    colorings.set_width_request(120);
    colorings.set_margin_end(15);
    let iter_val = state.borrow().mapping.iteration_depth as f64;
    let iter_adj = Adjustment::new(iter_val, 10.0, 1000.0, 1.0, 0.0, 0.0);
    let iteration_button = SpinButton::builder().adjustment(&iter_adj).build();
    let first_row = make_row_box();
    first_row.append(&Label::new(Some("coloring:")));
    first_row.append(&colorings);
    first_row.append(&Label::new(Some("max iterations:")));
    first_row.append(&iteration_button);
    let cx_value = gtk::Entry::builder()
        .text(&state.borrow().mapping.cx.to_string())
        .width_chars(15)
        .margin_end(10)
        .build();
    let cy_value = gtk::Entry::builder()
        .text(&state.borrow().mapping.cy.to_string())
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
    state.borrow_mut().canvas = canvas.downgrade();
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
        iter_depth_changed(&mut state.borrow_mut(), a);
    }));
    cx_value.connect_changed(
        clone!(@strong state => move |e| { cx_changed(&mut state.borrow_mut(), e);}),
    );
    cy_value.connect_changed(
        clone!(@strong state => move |e| { cy_changed(&mut state.borrow_mut(), e);}),
    );
    let gesture = gtk::GestureClick::new();
    gesture.set_button(GDK_BUTTON_PRIMARY as u32);
    gesture.connect_pressed(clone!(@strong state => move |gesture, _, wx, wy| {
        gesture.set_state(gtk::EventSequenceState::Claimed);
        let (new_cx, new_cy) = state.borrow().win_to_mandel(wx, wy);
        cx_value.set_text(&new_cx.to_string());
        cy_value.set_text(&new_cy.to_string());
    }));
    canvas.add_controller(gesture);
    colorings.connect_selected_notify(clone!(@strong state => move |dd| {
        col_changed(&mut state.borrow_mut(), dd);
    }));
    zoom_adj.connect_value_changed(clone!(@strong state => move |adj| {
        zoom_changed(&mut state.borrow_mut(), adj);
    }));
    canvas.connect_resize(move |_da, w, h| on_resize(&state, w, h));

    window.present();
}

pub fn run() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}
