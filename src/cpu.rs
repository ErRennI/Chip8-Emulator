use crate::display::{Display, HEIGHT, WIDTH};
use crate::font::FONT;
use crate::keypad::Keypad;

const FONT_START: usize = 0x050;
const ROM_START: usize = 0x200;
const STACK_SIZE: usize = 16;
const VF: usize = 0xF;

#[derive(Debug)]
pub enum Chip8Error {
    StackOverflow,
    StackUnderflow,
    InvalidOpcode(u16),
    OutOfBounds { addr: usize },
}

pub struct Chip8 {
    //Memory
    pub memory: [u8; 4096],

    //Registers
    pub v: [u8; 16],
    pub i: u16,

    //Program Counter
    pub pc: u16,
    //Stack
    pub stack: [u16; STACK_SIZE],
    pub sp: usize, //Normally old chip8 hardware had 8 bit sp but in rust usize is better for indexing

    //Timers
    pub delay_timer: u8,
    pub sound_timer: u8,

    pub display: Display,
    pub keypad: Keypad,

    pub waiting_for_vblank: bool,
    pub waiting_key: Option<u8>,
}

impl Default for Chip8 {
    fn default() -> Self {
        Self::new()
    }
}

impl Chip8 {
    pub fn new() -> Self {
        let mut chip8 = Self {
            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: (ROM_START as u16),
            stack: [0; 16],
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            display: Display::new(),
            keypad: Keypad::new(),
            waiting_for_vblank: false,
            waiting_key: None,
        };

        chip8.memory[FONT_START..FONT_START + FONT.len()].copy_from_slice(&FONT);
        chip8
    }

    pub fn load_rom(&mut self, rom: &[u8]) -> Result<(), String> {
        let max: usize = 4096 - ROM_START;
        if rom.len() > max {
            return Err(format!("{} is the maximum size for the ROM", max));
        }
        self.memory[ROM_START..ROM_START + rom.len()].copy_from_slice(rom);
        Ok(())
    }

    fn fetch(&mut self) -> u16 {
        let opcode: u16 = u16::from_be_bytes([
            self.memory[self.pc as usize],
            self.memory[(self.pc + 1) as usize],
        ]);

        self.pc += 2;
        opcode
    }

    fn execute(&mut self, opcode: u16) -> Result<(), Chip8Error> {
        let op: u16 = (opcode & 0xF000) >> 12;
        let x: usize = ((opcode & 0x0F00) >> 8) as usize;
        let y: usize = ((opcode & 0x00F0) >> 4) as usize;
        let nnn: u16 = opcode & 0x0FFF;
        let n: u8 = (opcode & 0x000F) as u8;
        let kk: u8 = (opcode & 0x00FF) as u8;

        match op {
            0x0 => match kk {
                0xE0 => self.display.clear(),
                0xEE => {
                    if self.sp == 0 {
                        return Err(Chip8Error::StackUnderflow);
                    }
                    self.sp -= 1;
                    self.pc = self.stack[self.sp];
                }
                _ => return Err(Chip8Error::InvalidOpcode(opcode)),
            },
            0x1 => self.pc = nnn,
            0x2 => {
                if self.sp >= 16 {
                    return Err(Chip8Error::StackOverflow);
                }
                self.stack[self.sp] = self.pc;
                self.sp += 1;
                self.pc = nnn;
            }
            0x3 => {
                if self.v[x] == kk {
                    self.pc += 2;
                }
            }
            0x4 => {
                if self.v[x] != kk {
                    self.pc += 2;
                }
            }
            0x5 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            0x6 => self.v[x] = kk,
            0x7 => self.v[x] = self.v[x].wrapping_add(kk),
            0x8 => match n {
                0x0 => self.v[x] = self.v[y],
                0x1 => {
                    self.v[x] |= self.v[y];
                    self.v[VF] = 0;
                }
                0x2 => {
                    self.v[x] &= self.v[y];
                    self.v[VF] = 0;
                }
                0x3 => {
                    self.v[x] ^= self.v[y];
                    self.v[VF] = 0;
                }
                0x4 => self.op_8xy4(x, y),
                0x5 => self.op_8xy5(x, y),
                0x6 => self.op_8xy6(x, y),
                0x7 => self.op_8xy7(x, y),
                0xE => self.op_8xye(x, y),
                _ => return Err(Chip8Error::InvalidOpcode(opcode)),
            },
            0x9 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            0xA => self.i = nnn,
            0xB => self.pc = (self.v[0] as u16).wrapping_add(nnn),
            0xC => self.v[x] = rand::random_range(0..=255) & kk,
            0xD => self.op_dxyn(x, y, n),
            0xE => match kk {
                0x9E => {
                    if self.keypad.is_pressed((self.v[x] & 0xF) as usize) {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    if !self.keypad.is_pressed((self.v[x] & 0xF) as usize) {
                        self.pc += 2;
                    }
                }
                _ => return Err(Chip8Error::InvalidOpcode(opcode)),
            },
            0xF => match kk {
                0x07 => self.v[x] = self.delay_timer,
                0x0A => {
                    if let Some(key) = self.waiting_key {
                        if !self.keypad.is_pressed(key as usize) {
                            self.v[x] = key;
                            self.waiting_key = None;
                        } else {
                            self.pc -= 2;
                        }
                    } else if let Some(key) = self.keypad.first_pressed() {
                        self.waiting_key = Some(key);
                        self.pc -= 2;
                    } else {
                        self.pc -= 2;
                    }
                }
                0x15 => self.delay_timer = self.v[x],
                0x18 => self.sound_timer = self.v[x],
                0x1E => self.i = self.i.wrapping_add(self.v[x] as u16),
                0x29 => self.i = (FONT_START + (self.v[x] & 0xF) as usize * 5) as u16,
                0x33 => self.op_fx33(x)?,
                0x55 => self.op_fx55(x)?,
                0x65 => self.op_fx65(x)?,
                _ => return Err(Chip8Error::InvalidOpcode(opcode)),
            },
            _ => return Err(Chip8Error::InvalidOpcode(opcode)),
        }
        Ok(())
    }

    pub fn step(&mut self) -> Result<(), Chip8Error> {
        if self.waiting_for_vblank {
            return Ok(());
        }
        let opcode = self.fetch();
        self.execute(opcode)
    }

    pub fn tick_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    // The values of Vx and Vy are added together.
    // If the result is greater than 8 bits (i.e., > 255,) VF is set to 1, otherwise 0.
    // Only the lowest 8 bits of the result are kept, and stored in Vx.
    fn op_8xy4(&mut self, x: usize, y: usize) {
        let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[x] = sum;
        self.v[VF] = carry as u8;
    }

    // If Vx > Vy, then VF is set to 1, otherwise 0.
    // Then Vy is subtracted from Vx, and the results stored in Vx.
    fn op_8xy5(&mut self, x: usize, y: usize) {
        let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[x] = result;
        self.v[VF] = if borrow { 0 } else { 1 };
    }

    fn op_8xy6(&mut self, x: usize, y: usize) {
        let flag: u8 = self.v[y] & 0x01;
        self.v[x] = self.v[y] >> 1;
        self.v[VF] = flag;
    }

    fn op_8xy7(&mut self, x: usize, y: usize) {
        let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[x] = result;
        self.v[VF] = if borrow { 0 } else { 1 };
    }

    fn op_8xye(&mut self, x: usize, y: usize) {
        let flag: u8 = self.v[y] >> 7;
        self.v[x] = self.v[y] << 1;
        self.v[VF] = flag;
    }

    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision.
    // The interpreter reads n bytes from memory, starting at the address stored in I. These bytes are then displayed as sprites on screen at coordinates (Vx, Vy).
    // Sprites are XORed onto the existing screen.
    // If this causes any pixels to be erased, VF is set to 1, otherwise it is set to 0.
    fn op_dxyn(&mut self, x: usize, y: usize, n: u8) {
        let vx: usize = self.v[x] as usize % WIDTH;
        let vy: usize = self.v[y] as usize % HEIGHT;
        self.v[VF] = 0;

        for row in 0..n as usize {
            let byte: u8 = self.memory[self.i as usize + row];
            if vy + row >= HEIGHT {
                break;
            }

            for col in 0..8 {
                if vx + col >= WIDTH {
                    break;
                }

                let bit: bool = (byte >> (7 - col)) & 1 == 1;

                if bit {
                    let index: usize = (vy + row) * WIDTH + (vx + col);

                    if self.display.pixels[index] {
                        self.v[VF] = 1;
                    }

                    self.display.pixels[index] ^= true;
                }
            }
        }
        self.waiting_for_vblank = true;
    }

    //Store BCD representation of Vx in memory locations I, I+1, and I+2
    fn op_fx33(&mut self, x: usize) -> Result<(), Chip8Error> {
        let i = self.i as usize;
        if i + 2 >= 4096 {
            return Err(Chip8Error::OutOfBounds { addr: i + 2 });
        }
        self.memory[i] = (self.v[x] / 100) % 10;
        self.memory[i + 1] = (self.v[x] / 10) % 10;
        self.memory[i + 2] = self.v[x] % 10;
        Ok(())
    }

    //Store registers V0 through Vx in memory starting at location I.
    fn op_fx55(&mut self, x: usize) -> Result<(), Chip8Error> {
        let start: usize = self.i as usize;

        let dest = self
            .memory
            .get_mut(start..=start + x)
            .ok_or(Chip8Error::OutOfBounds { addr: (start + x) })?;

        dest.copy_from_slice(&self.v[..=x]);
        self.i = self.i.wrapping_add(x as u16 + 1);
        Ok(())
    }

    //Read registers V0 through Vx from memory starting at location I.
    fn op_fx65(&mut self, x: usize) -> Result<(), Chip8Error> {
        let i: usize = self.i as usize;

        let values = &self
            .memory
            .get(i..=i + x)
            .ok_or(Chip8Error::OutOfBounds { addr: (i + x) })?;
        self.v[..=x].copy_from_slice(values);

        self.i = self.i.wrapping_add(x as u16 + 1);
        Ok(())
    }
}
