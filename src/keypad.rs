pub struct Keypad {
    pub keys: [bool; 16],
}

impl Default for Keypad {
    fn default() -> Self {
        Self::new()
    }
}

impl Keypad {
    pub fn new() -> Self {
        Self { keys: [false; 16] }
    }

    pub fn set(&mut self, key: usize, pressed: bool) {
        self.keys[key] = pressed;
    }

    pub fn is_pressed(&self, key: usize) -> bool {
        self.keys[key]
    }

    pub fn first_pressed(&self) -> Option<u8> {
        self.keys
            .iter()
            .position(|&pressed| pressed)
            .map(|i| i as u8)
    }
}
