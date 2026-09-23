use crate::{
    cpu::{Chip8, Chip8Error},
    display::{HEIGHT, WIDTH},
};
use clap::Parser;
use minifb::{Window, WindowOptions};

mod cpu;
mod display;
mod font;
mod keypad;

const SCALE: usize = 15;

const KEY_MAP: [minifb::Key; 16] = [
    minifb::Key::X,    // 0
    minifb::Key::Key1, // 1
    minifb::Key::Key2, // 2
    minifb::Key::Key3, // 3
    minifb::Key::Q,    // 4
    minifb::Key::W,    // 5
    minifb::Key::E,    // 6
    minifb::Key::A,    // 7
    minifb::Key::S,    // 8
    minifb::Key::D,    // 9
    minifb::Key::Z,    // A
    minifb::Key::C,    // B
    minifb::Key::Key4, // C
    minifb::Key::R,    // D
    minifb::Key::F,    // E
    minifb::Key::V,    // F
];

#[derive(clap::Parser)]
struct Args {
    rom: String,
    #[arg(short, long, default_value_t = 10)]
    speed: usize,
}

fn main() {
    let args: Args = Args::parse();

    let rom = match std::fs::read(&args.rom) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Couln't read ROM: {}", e);
            std::process::exit(1);
        }
    };

    let mut chip8: Chip8 = Chip8::new();
    if let Err(e) = chip8.load_rom(&rom) {
        eprintln!("Couldn't load ROM: {}", e);
        std::process::exit(1);
    }

    let mut window: Window = match Window::new(
        "CHIP-8",
        WIDTH * SCALE,
        HEIGHT * SCALE,
        WindowOptions::default(),
    ) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Couldn't create window: {}", e);
            std::process::exit(1);
        }
    };
    window.set_target_fps(60);

    let mut buffer: Vec<u32> = vec![0; WIDTH * SCALE * HEIGHT * SCALE];

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        for i in 0..16 as usize {
            let pressed: bool = window.is_key_down(KEY_MAP[i]);
            chip8.keypad.set(i, pressed);
        }

        for _ in 0..args.speed {
            if let Err(e) = chip8.step() {
                match e {
                    Chip8Error::InvalidOpcode(op) => eprintln!("Unknown opcode: {:04X}", op),
                    Chip8Error::OutOfBounds { addr } => {
                        eprintln!("Out of memory bounds: {:04X}", addr)
                    }
                    other => eprintln!("CPU error: {:?}", other),
                }
            }
        }
        chip8.tick_timers();
        chip8.waiting_for_vblank = false;

        for (i, &pixel) in chip8.display.pixels.iter().enumerate() {
            let color = if pixel { 0xFFFFFF } else { 0x000000 };
            let px: usize = i % WIDTH;
            let py: usize = i / WIDTH;

            for dy in 0..SCALE {
                for dx in 0..SCALE {
                    let bx: usize = px * SCALE + dx;
                    let by: usize = py * SCALE + dy;
                    buffer[by * WIDTH * SCALE + bx] = color;
                }
            }
        }

        window
            .update_with_buffer(&buffer, WIDTH * SCALE, HEIGHT * SCALE)
            .unwrap();
    }
}
