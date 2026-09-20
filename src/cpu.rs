use crate::display::Display;
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

    pub fn fetch(&mut self) -> u16 {
        let opcode: u16 = u16::from_be_bytes([
            self.memory[self.pc as usize],
            self.memory[(self.pc + 1) as usize],
        ]);

        self.pc += 2;
        opcode
    }

    pub fn execute(&mut self, opcode: u16) -> Result<(), Chip8Error> {
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
                    self.pc = self.stack[self.sp];
                    self.sp -= 1;
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
            0x7 => self.v[x] += kk,
            0x8 => match n {
                0x0 => self.v[x] = self.v[y],
                0x1 => self.v[x] |= self.v[y],
                0x2 => self.v[x] &= self.v[y],
                0x4 => self.op_8xy4(x, y),
                0x5 => self.op_8xy5(x, y),
                _ => return Err(Chip8Error::InvalidOpcode(opcode)),
            },

            _ => return Err(Chip8Error::InvalidOpcode(opcode)),
        }
        Ok(())
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
        let (result, borrow) = self.v[x].overflowing_sub(self.v[x]);
        self.v[x] = result;
        self.v[VF] = borrow as u8;
    }
}
