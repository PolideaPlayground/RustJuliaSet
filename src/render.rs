use crate::buffer::Buffer;
use rayon::prelude::*;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum RenderCommand {
    RenderRequest {
        buffer: Buffer,
        params: RenderParameters,
    },
    Quit,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RenderParameters {
    pub iterations: u32,
    pub cx: f32,
    pub cy: f32,
}

#[derive(Debug)]
pub struct RenderResult {
    pub buffer: Buffer,
    pub render_time: Duration,
}

#[allow(clippy::many_single_char_names)]
pub fn render_pixel(
    params: RenderParameters,
    (x, y): (usize, usize),
    (width, heigh): (usize, usize),
) -> u32 {
    let width = width as f32;
    let height = heigh as f32;
    let x = x as f32;
    let y = y as f32;
    let mut zx = 3.0 * (x - 0.5 * width) / (width);
    let mut zy = 2.0 * (y - 0.6 * height) / (height);

    let RenderParameters { cx, cy, iterations } = params;
    let mut i = iterations;
    while zx * zx + zy * zy < 4.0 && i > 1 {
        let tmp = zx * zx - zy * zy + cx;
        zy = 2.0 * zx * zy + cy;
        zx = tmp;
        i -= 1;
    }

    let ratio = i as f32 / iterations as f32;
    let r = (ratio * 255.0) as u8;
    let g = (ratio * ratio * 255.0) as u8;
    let b = (ratio.sqrt() * 255.0) as u8;

    (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b) | 0xFF00_0000
}

pub fn render_loop(receiver: Receiver<RenderCommand>, sender: Sender<RenderResult>) {
    while let RenderCommand::RenderRequest { mut buffer, params } =
        receiver.recv().unwrap_or(RenderCommand::Quit)
    {
        const CHUNKS: usize = 8 * 1024;
        let start_time = Instant::now();
        let w = buffer.width();
        let h = buffer.height();

        buffer
            .as_mut_slice()
            // .chunks_mut(CHUNKS) // Uncomment this line and comment one below to get single CPU perf.
            .par_chunks_mut(CHUNKS)
            .enumerate()
            .for_each(|(offset, mut_slice)| {
                for (i, pixel) in mut_slice.iter_mut().enumerate() {
                    let pos = offset * CHUNKS + i;
                    let x = pos % w;
                    let y = pos / h;
                    *pixel = render_pixel(params, (x, y), (w, h));
                }
            });

        sender
            .send(RenderResult {
                buffer,
                render_time: Instant::now() - start_time,
            })
            .expect("Cannot send render result");
    }
}
