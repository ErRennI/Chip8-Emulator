use clap::Parser;
use cpu::Chip8;
use display::{HEIGHT, WIDTH};
use sdl2::{
    VideoSubsystem, event::Event, keyboard::Keycode, pixels::PixelFormatEnum, render::Canvas,
    video::Window,
};
use std::time::{Duration, Instant};

mod cpu;
mod display;
mod font;
mod keypad;

const SCALE: usize = 15;

fn keycode_to_chip8(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}

#[derive(clap::Parser)]
struct Args {
    rom: String,
    #[arg(short, long, default_value_t = 10)]
    speed: usize,
}

fn main() -> Result<(), String> {
    let args: Args = Args::parse();

    let rom = match std::fs::read(&args.rom) {
        Ok(bytes) => bytes,
        Err(e) => {
            eprintln!("Couldn't read ROM: {}", e);
            std::process::exit(1);
        }
    };

    let mut chip8: Chip8 = Chip8::new();
    if let Err(e) = chip8.load_rom(&rom) {
        eprintln!("Couldn't load ROM: {}", e);
        std::process::exit(1);
    }

    let sdl: sdl2::Sdl = sdl2::init()?;
    let video: VideoSubsystem = sdl.video()?;
    let window: Window = video
        .window("CHIP-8", (WIDTH * SCALE) as u32, (HEIGHT * SCALE) as u32)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas: Canvas<Window> = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas
        .set_logical_size(WIDTH as u32, HEIGHT as u32)
        .map_err(|e| e.to_string())?;

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
        .map_err(|e| e.to_string())?;

    let mut events = sdl.event_pump()?;
    let frame_time = Duration::from_micros(16_667);

    'running: loop {
        let frame_start: Instant = Instant::now();

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keycode: Some(k),
                    repeat: false,
                    ..
                } => {
                    if let Some(i) = keycode_to_chip8(k) {
                        chip8.keypad.set(i, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(k), ..
                } => {
                    if let Some(i) = keycode_to_chip8(k) {
                        chip8.keypad.set(i, false);
                    }
                }
                _ => {}
            }
        }

        for _ in 0..args.speed {
            if let Err(e) = chip8.step() {
                eprintln!("Emulator error: {e}");
                break 'running;
            }
        }

        chip8.tick_timers();
        chip8.waiting_for_vblank = false;

        texture
            .with_lock(None, |buf: &mut [u8], pitch: usize| {
                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        let v = if chip8.display.pixels[y * WIDTH + x] {
                            255
                        } else {
                            0
                        };
                        let o = y * pitch + x * 3;
                        buf[o] = v;
                        buf[o + 1] = v;
                        buf[o + 2] = v;
                    }
                }
            })
            .map_err(|e| e.to_string())?;

        canvas.clear();
        canvas.copy(&texture, None, None)?;
        canvas.present();

        if let Some(rest) = frame_time.checked_sub(frame_start.elapsed()) {
            std::thread::sleep(rest);
        }
    }

    Ok(())
}
