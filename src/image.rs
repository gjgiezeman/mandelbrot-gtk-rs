use gtk::cairo::{Format, ImageSurface};

pub struct Image {
    _data: Vec<u8>,
    surface: ImageSurface,
}

impl Image {
    pub fn new(mut data: Vec<u8>, format: Format, width: i32, height: i32, stride: i32) -> Image {
        let surface;
        unsafe {
            surface = ImageSurface::create_for_data_unsafe(
                data.as_mut_ptr(),
                format,
                width,
                height,
                stride,
            )
            .unwrap();
        }
        Image {
            _data: data,
            surface,
        }
    }

    pub fn surface(&self) -> &ImageSurface {
        &self.surface
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        // Calling finish is important.
        // Otherwise cairo will try to access the removed data
        self.surface.finish();
    }
}
