use crate::{colorings::Coloring, MandelReply, MandelReq, IMG_FMT};

#[derive(Clone)]
/// Parameters for mapping from mandelbrot space to a window
pub struct Mapping {
    /// The x coordinate that is in the horizontal center of the window
    pub cx: f64,
    /// The y coordinate that is in the vertical center of the window
    pub cy: f64,
    /// The length in mandelbrot space that corresponds with the width or
    /// height of one pixel in the window
    pub scale: f64,
    /// The maximum number of iterations used in the computation of the
    /// mandelbrot value. So, also the maximum mandelbrot value.
    pub iteration_depth: u32,
    /// The width of the window
    pub win_width: usize,
    /// The height of the window
    pub win_height: usize,
}

impl Mapping {
    pub fn new_for_size(win_sz: usize) -> Mapping {
        Mapping {
            cx: 0.0,
            cy: 0.0,
            scale: 4.0 / win_sz as f64,
            iteration_depth: 100,
            win_width: win_sz,
            win_height: win_sz,
        }
    }
    pub fn is_valid(&self) -> bool {
        let max = i32::MAX as usize;
        0 < self.win_width
            && self.win_width <= max
            && 0 < self.win_height
            && self.win_height <= max
            && self.scale > 0.0
            && self.iteration_depth > 0
    }
}

/*
The transformation from window coordinates (x_w, y_w) to mandelbrot coordinates (x_m, y_m) can be done
with three parameters: x0, y0, f, such that
x_m(x_w) = x0 + f * x_w
y_m(y_w) = y0 - f * y_w (notice the minus sign, because in mathematics the y axis points upwards, in windows downwards)

With a scale s and mandelcoordinates (x_c, y_c) in the center of the window, we want,
with w the current width of the window, h the current height of the window
f = s
x_m(w/2) = x_c (x_c should be in the horizontal center)
y_m(h/2) = y_c (y_c should be in the vertical center)

The solution is:
f = s
x0 = x_c - (f*w)/2
y0 = y_c - (f*h)/2
 */
pub struct WinToMandel {
    x0: f64,
    y0: f64,
    f: f64,
}

impl WinToMandel {
    pub fn from_mapping(mapping: &Mapping) -> WinToMandel {
        let f = mapping.scale;
        let x0: f64 = mapping.cx - (f * mapping.win_width as f64) / 2.0;
        let y0 = mapping.cy + (f * mapping.win_height as f64) / 2.0;
        WinToMandel { x0, y0, f }
    }
    pub fn cvt(&self, wx: usize, wy: usize) -> (f64, f64) {
        (self.x0 + wx as f64 * self.f, self.y0 - wy as f64 * self.f)
    }
    pub fn cvt_x(&self, wx: usize) -> f64 {
        self.x0 + wx as f64 * self.f
    }
    pub fn cvt_y(&self, wy: usize) -> f64 {
        self.y0 - wy as f64 * self.f
    }
}

// Return the number of iterations before we encounter the stop criterion
fn mandel_value(x: f64, y: f64, max_iter: u32) -> u32 {
    // The number of iterations
    let mut iter = 0;
    // The initial values of r and i.
    let (mut r, mut i) = (0.0, 0.0);
    while iter < max_iter {
        // Compute the new values for r and i
        (r, i) = (r * r - i * i + x, 2.0 * r * i + y);
        // The stop criterion
        if i * i + r * r >= 4.0 {
            break;
        }
        iter += 1;
    }
    iter
}

// Fill the bytes of an image with the mandelbrot image according to the parameters.
// Each row of the image contains ustride bytes.
fn fill_mandel_image(
    data: &mut [u8],
    ustride: usize,
    mapping: &Mapping,
    col_producer: &Box<dyn Coloring>,
) -> bool {
    {
        let converter = WinToMandel::from_mapping(mapping);
        let w = mapping.win_width;
        let h = mapping.win_height;
        let max = mapping.iteration_depth;
        for dy in 0..h {
            let y = converter.cvt_y(dy);
            let line = &mut data[dy * ustride..(dy + 1) * ustride];
            let mut iter = line.iter_mut();
            for wx in 0..w {
                let x = converter.cvt_x(wx);
                let mv = mandel_value(x, y, max);
                let color = col_producer.get_color(mv, max);
                let bytes = color.to_ne_bytes();
                for i in 0..bytes.len() {
                    if let Some(v) = iter.next() {
                        *v = bytes[i];
                    } else {
                        return false;
                    }
                }
            }
        }
        true
    }
}

// Make an Vec<u8> and fill it with a mandelbrot image, according to the parameters.
pub fn make_mandel_image(
    mapping: &Mapping,
    col_producer: &Box<dyn Coloring>,
) -> Option<(Vec<u8>, i32)> {
    if !mapping.is_valid() {
        return None;
    }
    match IMG_FMT.stride_for_width(mapping.win_width as u32) {
        Err(_) => None,
        Ok(stride) => {
            let h = mapping.win_height as usize;
            let ustride = stride as usize;
            let mut surface: Vec<u8> = vec![0; h * ustride];
            if fill_mandel_image(surface.as_mut(), ustride, mapping, col_producer) {
                Some((surface, stride))
            } else {
                None
            }
        }
    }
}

fn last_request(
    mut request: MandelReq,
    req_receiver: &async_channel::Receiver<MandelReq>,
) -> MandelReq {
    loop {
        // If there are multiple requests, throw away all but the last
        match req_receiver.try_recv() {
            Ok(new_request) => {
                request = new_request;
            }
            Err(_) => return request,
        }
    }
}

pub fn mandel_producer(
    req_receiver: async_channel::Receiver<MandelReq>,
    reply_sender: async_channel::Sender<MandelReply>,
) {
    loop {
        let mut request;
        match req_receiver.recv_blocking() {
            Err(_) => break,
            Ok(new_request) => {
                request = new_request;
            }
        }
        request = last_request(request, &req_receiver);
        if let Some((data, stride)) = make_mandel_image(&request.mapping, &request.coloring) {
            let _ = reply_sender.send_blocking(MandelReply {
                data,
                width: request.mapping.win_width as i32,
                height: request.mapping.win_height as i32,
                stride,
            });
        }
    }
}
