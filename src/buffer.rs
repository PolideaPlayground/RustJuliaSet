use std::fmt::Debug;

pub struct Buffer {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
}

// Implement Debug to not show any pixels.
impl Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Buffer")
    }
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            buffer: vec![0; width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn as_slice(&self) -> &[u32] {
        self.buffer.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [u32] {
        self.buffer.as_mut_slice()
    }
}
