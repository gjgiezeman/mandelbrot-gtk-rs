use crate::mandel_image::{make_mandel_image, Mapping};
use gtk::cairo::ImageSurface;
use gtk::{glib, prelude::*, Application, ApplicationWindow, DrawingArea};

const APP_ID: &str = "nl.uu.gjgiezeman.mandelbrot";
const WIN_SZ0: usize = 600;

fn mandel_draw(
    img: &Option<ImageSurface>,
    _da: &DrawingArea,
    ctxt: &gtk::cairo::Context,
    _w: i32,
    _h: i32,
) {
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
        .build();
    let image = make_mandel_image(&Mapping::new_for_size(WIN_SZ0));
    canvas.set_draw_func(move |d, c, w, h| mandel_draw(&image, d, c, w, h));

    // Create a window and set the title
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
