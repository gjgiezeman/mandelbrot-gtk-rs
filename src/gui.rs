mod state;

use self::state::State;
use crate::presets::Presets;
use gtk::ffi::GTK_INVALID_LIST_POSITION;
use gtk::gdk::ffi::GDK_BUTTON_PRIMARY;
use gtk::glib::clone;
use gtk::glib::object::Cast;
use gtk::{
    glib, prelude::*, Adjustment, Application, ApplicationWindow, Button, DrawingArea, DropDown,
    GestureClick, Label, ListItem, ListView, Orientation, Scale, SignalListItemFactory,
    SingleSelection, SpinButton, StringList, StringObject, Window,
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

fn color_changed(state: &mut State, dd: &DropDown) {
    let sel = dd.selected();
    if sel != GTK_INVALID_LIST_POSITION {
        state.set_col_idx(sel as usize);
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

fn preset_ready(
    state: &Rc<RefCell<State>>,
    cx_value: &gtk::Entry,
    cy_value: &gtk::Entry,
    zoom_adj: &Adjustment,
    iter_adj: &Adjustment,
    presets: &Presets,
) {
    let preset = state.borrow_mut().take_preset();
    if let Some(preset) = preset {
        let preset = presets.get(preset as usize);
        cx_value.set_text(&preset.cx().to_string());
        cy_value.set_text(&preset.cy().to_string());
        zoom_adj.set_value(preset.zoom());
        iter_adj.set_value(preset.iter_depth());
    }
}

fn preset_setup(_fac: &SignalListItemFactory, item: &ListItem) {
    item.set_child(Some(&Label::new(None)));
}

fn preset_bind(_fac: &SignalListItemFactory, item: &ListItem) {
    if let Some(widget) = item.child() {
        if let Some(obj) = item.item() {
            if let Ok(str) = obj.downcast::<StringObject>() {
                if let Ok(label) = widget.downcast::<Label>() {
                    label.set_text(&str.string());
                }
            }
        }
    }
}

fn build_preset_window(state: &Rc<RefCell<State>>, presets: &Presets) -> Window {
    let preset_list = SingleSelection::new(Some(StringList::new(presets.names())));
    let factory = SignalListItemFactory::new();
    factory.connect_setup(preset_setup);
    factory.connect_bind(preset_bind);
    let preset_view = ListView::builder()
        .model(&preset_list.clone())
        .factory(&factory)
        .margin_top(20)
        .margin_start(20)
        .margin_end(20)
        .build();
    let cancel_btn = Button::builder().label("Cancel").build();
    let ok_btn = Button::builder().label("Apply").margin_start(10).build();
    let ready_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .margin_top(30)
        .margin_start(20)
        .margin_bottom(20)
        .margin_end(20)
        .build();
    ready_box.append(&cancel_btn);
    ready_box.append(&ok_btn);
    let content_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();
    content_box.append(&preset_view);
    content_box.append(&ready_box);
    let win = Window::builder()
        .title("Presets")
        .modal(true)
        .resizable(false)
        .deletable(false)
        .hide_on_close(true)
        .child(&content_box)
        .build();
    cancel_btn.connect_clicked(clone!(@weak win, @strong state => move |_| {
        state.borrow_mut().set_preset(None);
        win.set_visible(false);
    }));
    ok_btn.connect_clicked(clone!(@weak win, @strong state => move |_| {
        let sel = preset_list.selected();
        state.borrow_mut().set_preset(Some(sel as u8));
        win.set_visible(false);
    }));
    win
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
    colorings = DropDown::from_strings(&state.borrow().coloring_names());
    colorings.set_width_request(120);
    colorings.set_margin_end(15);
    let iter_val = state.borrow().iter_depth();
    let iter_adj = Adjustment::new(iter_val, 10.0, 1000.0, 1.0, 0.0, 0.0);
    let iteration_button = SpinButton::builder().adjustment(&iter_adj).build();
    let preset_btn = Button::builder()
        .label("Choose Preset")
        .margin_start(15)
        .build();
    let first_row = make_row_box();
    first_row.append(&Label::new(Some("coloring:")));
    first_row.append(&colorings);
    first_row.append(&Label::new(Some("max iterations:")));
    first_row.append(&iteration_button);
    first_row.append(&preset_btn);
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

    let presets = Presets::new();
    let preset_window = build_preset_window(&state, &presets);
    preset_window.set_transient_for(Some(&window));
    preset_window.connect_hide(
        clone!(@strong state, @weak zoom_adj, @weak iter_adj, @weak cx_value, @weak cy_value =>
            move|_w| preset_ready(&state, &cx_value, &cy_value, &zoom_adj, &iter_adj, &presets)),
    );

    // Set actions
    canvas.set_draw_func(clone!(@strong state =>move |_d, ctxt, _w, _h| mandel_draw(&state, ctxt)));
    iter_adj.connect_value_changed(clone!(@strong state => move |a| {
        state.borrow_mut().set_iter_depth(a.value());
    }));
    preset_btn
        .connect_clicked(clone!(@strong preset_window => move |_btn| preset_window.present();));
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
    colorings.connect_selected_notify(clone!(@strong state => move |dd| {
        color_changed(&mut state.borrow_mut(), dd);
    }));
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
