use std::thread;

use crate::{colorings::ColorFromMandel, MandelReply, MandelReq, IMG_FMT};
//use gtk::glib::ThreadPool;
use scoped_threadpool::Pool;

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
    pub fn from_mapping(mapping_params: &Mapping) -> WinToMandel {
        let f = mapping_params.scale;
        let x0: f64 = mapping_params.cx - (f * mapping_params.win_width as f64) / 2.0;
        let y0 = mapping_params.cy + (f * mapping_params.win_height as f64) / 2.0;
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

pub fn mandel_value(x: f64, y: f64, max: u32) -> u32 {
    let mut iter = 0;
    let mut r = 0.0;
    let mut i = 0.0;
    while iter < max {
        let rnext = r * r - i * i + x;
        i = 2.0 * r * i + y;
        r = rnext;
        if i * i + r * r >= 4.0 {
            break;
        }
        iter += 1;
    }
    iter
}

fn fill_mandel_image_partial(
    data: &mut [u8],
    col_producer: &Box<dyn ColorFromMandel>,
    converter: &WinToMandel,
    w: usize,
    h_start: usize,
    h_end: usize,
    max: u32,
    ustride: usize,
) -> bool {
    {
        for dy in 0..(h_end - h_start) {
            let y = converter.cvt_y(h_start + dy);
            let line = &mut data[dy * ustride..(dy + 1) * ustride];
            let mut iter = line.iter_mut();
            for wx in 0..w {
                let x = converter.cvt_x(wx);
                let mv = mandel_value(x, y, max);
                let bytes = col_producer.get(mv, max).to_ne_bytes();
                for i in 0..bytes.len() {
                    if let Some(v) = iter.next() {
                        *v = bytes[i];
                    }
                }
            }
        }
        return true;
    }
}

fn fill_mandel_image(
    data: &mut [u8],
    col_producer: &Box<dyn ColorFromMandel>,
    w: usize,
    h: usize,
    max: u32,
    ustride: usize,
    mparams: &Mapping,
) -> bool {
    let converter = WinToMandel::from_mapping(mparams);
    fill_mandel_image_partial(data, col_producer, &converter, w, 0, h, max, ustride)
}

fn fill_mandel_image_parallel(
    pool: &mut Pool,
    data: &mut [u8],
    col_producer: &Box<dyn ColorFromMandel>,
    w: usize,
    h: usize,
    max: u32,
    ustride: usize,
    mparams: &Mapping,
) -> bool {
    let converter = WinToMandel::from_mapping(mparams);
    let par_count = pool.thread_count() as usize;

    let h_step = h / par_count;
    let mut h_extra = h % par_count;
    let mut splits = Vec::with_capacity(par_count - 1);
    let mut split = 0;
    for _i in 0..par_count {
        splits.push(split);
        if h_extra > 0 {
            split += h_step + 1;
            h_extra -= 1;
        } else {
            split += h_step;
        }
    }
    let mut end = h;
    let mut statuses = vec![true; 8];
    pool.scoped(|scope| {
        let mut data = data;
        let mut statuses = &mut statuses[..];
        while let Some(s) = splits.pop() {
            let (first_status, last_part);
            (data, last_part) = data.split_at_mut(ustride * s);
            (first_status, statuses) = statuses.split_at_mut(1);
            let converter_ref = &converter;
            scope.execute(move || {
                first_status[0] = fill_mandel_image_partial(
                    last_part,
                    col_producer,
                    converter_ref,
                    w,
                    s,
                    end,
                    max,
                    ustride,
                );
            });
            end = s;
        }
    });
    for status in statuses {
        if !status {
            return false;
        }
    }
    true
}

fn make_mandel_image(request: &mut MandelReq, pool: &mut Pool) -> (Option<Vec<u8>>, i32) {
    if !request.params.is_valid() {
        return (None, 0);
    }

    match IMG_FMT.stride_for_width(request.params.win_width as u32) {
        Err(_) => {
            return (None, 0);
        }
        Ok(stride) => {
            let w = request.params.win_width as usize;
            let h = request.params.win_height as usize;
            let ustride = stride as usize;
            let max = request.params.iteration_depth;
            let mut surface: Vec<u8> = vec![0; h * ustride];
            let success = if h >= pool.thread_count() as usize {
                fill_mandel_image_parallel(
                    pool,
                    surface.as_mut(),
                    &request.coloring,
                    w,
                    h,
                    max,
                    ustride,
                    &request.params,
                )
            } else {
                fill_mandel_image(
                    surface.as_mut(),
                    &request.coloring,
                    w,
                    h,
                    max,
                    ustride,
                    &request.params,
                )
            };
            if success {
                return (Some(surface), stride);
            } else {
                return (None, 0);
            }
        }
    }
}

fn replace_by_last_request(
    available: &mut Option<MandelReq>,
    req_receiver: &async_channel::Receiver<MandelReq>,
) {
    loop {
        // If there are multiple requests, throw away all but the last
        match req_receiver.try_recv() {
            Ok(new_req) => {
                available.replace(new_req);
            }
            Err(_) => break,
        }
    }
}

pub fn producer(
    req_receiver: async_channel::Receiver<MandelReq>,
    reply_sender: async_channel::Sender<MandelReply>,
) {
    let par_count: usize;
    match thread::available_parallelism() {
        Ok(pc) => par_count = pc.into(),
        Err(_) => par_count = 8,
    }
    eprintln!("Parallelism is {}", par_count);
    let mut pool = Pool::new(par_count as u32);
    let mut available = None;
    loop {
        if available.is_none() {
            match req_receiver.recv_blocking() {
                Err(_) => break,
                Ok(request) => {
                    available = Some(request);
                }
            }
        }
        replace_by_last_request(&mut available, &req_receiver);
        // available is guaranteed to be is_some, hence unwrap is safe.
        let mut request = available.take().unwrap();
        let (data, stride) = make_mandel_image(&mut request, &mut pool);
        let _ = reply_sender.send_blocking(MandelReply {
            result: data,
            width: request.params.win_width as i32,
            height: request.params.win_height as i32,
            stride: stride,
        });
    }
}
