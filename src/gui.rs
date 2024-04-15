use gtk::{cairo::ImageSurface, glib, prelude::*, Application, ApplicationWindow, DrawingArea};

use crate::mandel_image::{make_mandel_image, Mapping};

const APP_ID: &str = "nl.uu.gjgiezeman.mandelbrot";
const WIN_SZ0: usize = 600;

fn mandel_draw(img: &Option<ImageSurface>, ctxt: &gtk::cairo::Context) {
    if let Some(img) = img {
        ctxt.set_source_surface(img, 0.0, 0.0)
            .expect("Expected to be able to set source surface");
        ctxt.paint().unwrap();
    }
}

fn build_ui(app: &Application) {
    let canvas = DrawingArea::builder()
        .content_height(WIN_SZ0 as i32)
        .content_width(WIN_SZ0 as i32)
        .margin_start(10)
        .margin_end(10)
        .margin_top(10)
        .margin_bottom(10)
        .build();
    let image = make_mandel_image(&Mapping::new_for_size(WIN_SZ0));
    canvas.set_draw_func(move |_d, ctxt, _w, _h| mandel_draw(&image, ctxt));

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Mandelbrot")
        .child(&canvas)
        .build();

    window.present();
}

pub fn run() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}
