pub const WIDTH: usize = 64;
pub const HEIGHT: usize = 32;

pub struct Display {
    pub pixels: [bool; HEIGHT * WIDTH],
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}

impl Display {
    pub fn new() -> Self {
        Self {
            pixels: [false; HEIGHT * WIDTH],
        }
    }

    pub fn clear(&mut self) {
        self.pixels.fill(false);
    }
}
