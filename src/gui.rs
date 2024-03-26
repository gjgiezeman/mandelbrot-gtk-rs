use crate::colorings::ColorInfo;
use crate::mandel_image::{make_mandel_image, Mapping, WinToMandel};
use crate::presets::Presets;
use crate::MandelReq;
use gtk::cairo::ImageSurface;
use gtk::ffi::GTK_INVALID_LIST_POSITION;
use gtk::glib::{clone, WeakRef};
use gtk::{
    glib, prelude::*, Adjustment, Align, Application, ApplicationWindow, Button, DrawingArea,
    DropDown, Label, Orientation, Scale, SpinButton, Window,
};
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "nl.uu.gjgiezeman.mandelbrot";
const WIN_SZ0: i32 = 600;

struct State {
    mparams: Mapping,
    img: Option<ImageSurface>,
    col_idx: usize,
    preset: Option<u8>,
    color_info: ColorInfo,
    canvas: WeakRef<DrawingArea>,
}

impl State {
    fn new() -> State {
        State {
            mparams: Mapping::new_for_size(WIN_SZ0 as usize),
            img: None,
            col_idx: 0,
            preset: None,
            color_info: ColorInfo::new(),
            canvas: WeakRef::new(),
        }
    }

    fn win_to_mandel(&self, wx: f64, wy: f64) -> (f64, f64) {
        WinToMandel::from_mapping(&self.mparams).cvt(wx as usize, wy as usize)
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

fn expect_float_value(e: &gtk::Entry) -> Option<f64> {
    let t = e.text();
    if let Ok(value) = t.parse::<f64>() {
        Some(value)
    } else {
        None
    }
}

fn handle_new_image(reply: Option<ImageSurface>, state: &mut State) {
    state.img = reply;
    if let Some(canvas) = state.canvas.upgrade() {
        canvas.queue_draw();
    }
}

fn recompute_image(state: &mut State) {
    let coloring = state.color_info.producer(state.col_idx);
    let request = MandelReq {
        params: state.mparams.clone(),
        coloring,
    };
    let reply = make_mandel_image(&request);
    handle_new_image(reply, state);
}

fn on_resize(state: &Rc<RefCell<State>>, _da: &DrawingArea, w: i32, h: i32) {
    {
        let mut s = state.borrow_mut();
        s.mparams.win_width = w as usize;
        s.mparams.win_height = h as usize;
        recompute_image(&mut s);
    }
}

fn on_click(state: &Rc<RefCell<State>>, wx: f64, wy: f64) -> (f64, f64) {
    let mut state = state.borrow_mut();
    let (cx, cy) = state.win_to_mandel(wx, wy);
    state.mparams.cx = cx;
    state.mparams.cy = cy;
    (cx, cy)
}

fn zoom_changed(state: &mut State, adj: &Adjustment) {
    let zoom = adj.value();
    // The value is chosen such that floating point approximation becomes clear near zoom == 1000
    let scale = 1.035_f64.powf(-zoom);
    state.mparams.scale = 4.0 * scale / WIN_SZ0 as f64;
    recompute_image(state);
}

fn iter_depth_changed(state: &mut State, adj: &Adjustment) {
    let iter_depth = adj.value() as u32;
    state.mparams.iteration_depth = iter_depth;
    recompute_image(state);
}

fn col_changed(state: &mut State, dd: &DropDown) {
    let sel = dd.selected();
    if sel != GTK_INVALID_LIST_POSITION {
        state.col_idx = sel as usize;
        recompute_image(state);
    }
}

fn preset_ready(
    state: &Rc<RefCell<State>>,
    cx_value: &gtk::Entry,
    cy_value: &gtk::Entry,
    zoom_adj: &Adjustment,
    iter_adj: &Adjustment,
    presets: &Presets,
) {
    let preset;
    {
        preset = state.borrow_mut().preset.take();
    }

    if let Some(preset) = preset {
        let preset = presets.get(preset as usize);
        let cx = preset.cx();
        let cy = preset.cy();
        cx_value.buffer().set_text(cx.to_string());
        cy_value.buffer().set_text(cy.to_string());
        zoom_adj.set_value(preset.zoom());
        iter_adj.set_value(preset.iter_depth());
    }
}

fn build_preset_window(state: &Rc<RefCell<State>>, presets: &Presets) -> Window {
    let preset_dropdown = DropDown::from_strings(presets.names());
    preset_dropdown.set_margin_top(10);
    preset_dropdown.set_margin_start(10);
    preset_dropdown.set_margin_end(10);
    let cancel_btn = Button::builder().label("Cancel").build();
    let ok_btn = Button::builder().label("Apply").margin_start(10).build();
    let ready_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .margin_top(10)
        .margin_start(10)
        .margin_bottom(10)
        .margin_end(10)
        .build();
    ready_box.append(&cancel_btn);
    ready_box.append(&ok_btn);
    let content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    content_box.append(&preset_dropdown);
    content_box.append(&ready_box);
    let win = Window::builder()
        .modal(true)
        .decorated(false)
        .child(&content_box)
        .build();
    cancel_btn.connect_clicked(clone!(@weak win, @strong state => move |_| {
        state.borrow_mut().preset=None;
        win.set_visible(false);
    }));
    ok_btn.connect_clicked(clone!(@weak win, @strong state => move |_| {
        let sel = preset_dropdown.selected();
        state.borrow_mut().preset=Some(sel as u8);
        win.set_visible(false);
    }));
    win
}

fn make_row_box() -> gtk::Box {
    gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .margin_start(10)
        .margin_top(10)
        .spacing(5)
        .build()
}

fn build_ui(app: &Application) {
    let state = Rc::new(RefCell::new(State::new()));
    let colors = DropDown::from_strings(state.borrow().color_info.color_names());
    colors.set_hexpand(false);
    colors.set_halign(Align::Start);
    colors.set_margin_end(15);
    let preset_btn = Button::builder()
        .label("Choose Preset")
        .halign(Align::End)
        .build();
    let iter_adj = Adjustment::new(100.0, 10.0, 1000.0, 1.0, 0.0, 0.0);
    let iteration_button = SpinButton::builder()
        .adjustment(&iter_adj)
        .halign(Align::Start)
        .margin_end(15)
        .build();
    let first_row = make_row_box();
    first_row.append(&Label::new(Some("coloring:")));
    first_row.append(&colors);
    first_row.append(&Label::new(Some("max iterations:")));
    first_row.append(&iteration_button);
    first_row.append(&preset_btn);
    let cx_value = gtk::Entry::builder()
        .text(&state.borrow().mparams.cx.to_string())
        .width_chars(10)
        .margin_end(15)
        .build();
    let cy_value = gtk::Entry::builder()
        .text(&state.borrow().mparams.cy.to_string())
        .width_chars(10)
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
        .content_height(WIN_SZ0)
        .content_width(WIN_SZ0)
        .vexpand(true)
        .can_target(true)
        .build();
    {
        let state = state.clone();
        canvas.set_draw_func(move |d, c, w, h| mandel_draw(&state, d, c, w, h));
    }
    state.borrow_mut().canvas = canvas.downgrade();

    let content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(5)
        .build();
    content_box.append(&first_row);
    content_box.append(&second_row);
    content_box.append(&third_row);
    content_box.append(
        &gtk::Frame::builder()
            .child(&canvas)
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .build(),
    );

    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Mandelbrot")
        .child(&content_box)
        .build();

    let presets = Presets::new();
    let preset_window = build_preset_window(&state, &presets);
    preset_window.set_transient_for(Some(&window));
    preset_window.connect_hide(
        clone!(@strong state, @weak zoom_adj, @weak iter_adj, @weak cx_value, @weak cy_value =>
            move|_w| preset_ready(&state, &cx_value, &cy_value, &zoom_adj, &iter_adj, &presets)),
    );

    // Add the actions to the widgets
    preset_btn
        .connect_clicked(clone!(@strong preset_window => move |_btn| preset_window.present();));

    // Color_producer
    colors.connect_selected_notify(clone!(@strong state => move |dd| {
        col_changed(&mut state.borrow_mut(), dd);
    }));
    // Zoom
    zoom_adj.connect_value_changed(clone!(@strong state => move |adj| {
        zoom_changed(&mut state.borrow_mut(), adj);
    }));
    // Iteration depth
    iter_adj.connect_value_changed(clone!(@strong state => move |a| {
        iter_depth_changed(&mut state.borrow_mut(), a);
    }));
    // New horizontal center
    cx_value.connect_changed(clone!(@strong state => move |e| {
        if let Some(value) = expect_float_value(e) {
            let mut s = state.borrow_mut();
            s.mparams.cx=value;
            recompute_image(&mut s);
        }
    }));
    // New vertical center
    cy_value.connect_changed(clone!(@strong state => move |e| {
        if let Some(value) = expect_float_value(e) {
            let mut s = state.borrow_mut();
            s.mparams.cy=value;
            recompute_image(&mut s);
        }
    }));
    // Click for new center
    let gesture = gtk::GestureClick::new();
    gesture.set_button(gtk::gdk::ffi::GDK_BUTTON_PRIMARY as u32);
    {
        let state = state.clone();
        gesture.connect_pressed(move |gesture, _, x, y| {
            gesture.set_state(gtk::EventSequenceState::Claimed);
            let (new_cx, new_cy) = on_click(&state, x, y);
            cx_value.set_text(&new_cx.to_string());
            cy_value.set_text(&new_cy.to_string());
        });
    }
    canvas.add_controller(gesture);

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
