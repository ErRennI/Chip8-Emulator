# Chip8-Emulator

A CHIP-8 interpreter/emulator written in Rust, built from scratch as a learning project. It implements the full classic CHIP-8 instruction set, passes the [Timendus CHIP-8 test suite](https://github.com/Timendus/chip8-test-suite) (IBM logo, `corax+`, flags, quirks, and keypad tests), and runs real ROMs like Pong and Tetris.

## Features

- Full CHIP-8 instruction set (all 35 opcodes)
- Accurate `VF` flag handling, including edge cases where `VX`/`VY` is `VF` itself
- Configurable quirk behavior aligned with original COSMAC VIP semantics:
  - Sprite clipping at screen edges (no wraparound)
  - `VF` reset after `8xy1`/`8xy2`/`8xy3` (OR/AND/XOR)
  - Display wait (vblank) quirk for `DXYN`
  - `8xy6`/`8xyE` (shift) operate on `VY`, storing the result in `VX`
- Proper `FX0A` behavior: waits for a key to be pressed **and released**
- 64x32 display rendered with [`minifb`](https://crates.io/crates/minifb), scaled up for visibility
- Keyboard input mapped to the classic CHIP-8 layout
- Command-line ROM loading via [`clap`](https://crates.io/crates/clap)
- Descriptive runtime errors (invalid opcodes, stack overflow/underflow, out-of-bounds memory access) instead of silent panics

## Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- A CHIP-8 ROM file (`.ch8`)

## Building

```sh
git clone https://github.com/ErRennI/Chip8-Emulator.git
cd Chip8-Emulator
cargo build --release
```

## Running

```sh
cargo run -- path/to/rom.ch8
```

Example:

```sh
cargo run -- rom/2-ibm-logo.ch8
```

If a ROM's filename contains spaces or special characters (parentheses, brackets), either wrap the whole path in quotes or escape each special character, e.g.:

```sh
cargo run -- "rom/Pong (1 player).ch8"
cargo run -- "rom/Pong 2 (Pong hack) [David Winter, 1997].ch8"
cargo run -- "rom/Tetris [Fran Dachille, 1991].ch8"

# or, escaping instead of quoting:
cargo run -- rom/Pong\ \(1\ player\).ch8
cargo run -- rom/Pong\ 2\ \(Pong\ hack\)\ \[David\ Winter,\ 1997\].ch8
cargo run -- rom/Tetris\ \[Fran\ Dachille,\ 1991\].ch8
```

Press **Esc** to close the emulator window.

### CPU Speed

By default the emulator runs 10 CPU instructions per frame (~600 Hz at 60 FPS). This can be adjusted with `--speed` (or `-s`):

```sh
cargo run -- rom/pong.ch8 --speed 15
```

Lower values make games feel slower/sluggish; higher values make them feel faster and more responsive, but pushing it too high can make some games unplayable (e.g. overly twitchy controls).

### Known display issue on GNOME/Wayland

On GNOME running under Wayland, the emulator window may appear without a title bar or border, and you may see a `Failed to create server-side surface decoration: Missing` warning in the terminal. This is **not a bug in the emulator** — GNOME's Wayland compositor (Mutter) doesn't support server-side window decorations, which `minifb` relies on. The window still works correctly (input, rendering, and closing via **Esc** are unaffected).

Workarounds:
- Force X11/XWayland: `WAYLAND_DISPLAY= cargo run -- rom/.ch8`
- Or simply ignore it — it's cosmetic only.

## Keyboard Layout

CHIP-8 uses a 16-key hexadecimal keypad. It's mapped to the keyboard as follows:

**CHIP-8 keypad**

```
1 2 3 C
4 5 6 D
7 8 9 E
A 0 B F
```

**Maps to keyboard**

```
1 2 3 4
Q W E R
A S D F
Z X C V
```

## Project Structure

```
src/
├── main.rs    # Argument parsing, window/event loop, keyboard mapping
├── cpu.rs     # Chip8 struct: memory, registers, fetch/decode/execute
├── display.rs # 64x32 pixel buffer
├── font.rs    # Built-in hexadecimal font sprites
└── keypad.rs  # 16-key input state
```

## Testing

This emulator has been validated against the [Timendus CHIP-8 test suite](https://github.com/Timendus/chip8-test-suite):

| Test           | Status |
|----------------|--------|
| IBM logo       | ✅ Pass |
| CHIP-8 splash  | ✅ Pass |
| corax+         | ✅ Pass |
| Flags          | ✅ Pass |
| Quirks (CHIP-8)| ✅ Pass |
| Keypad         | ✅ Pass |

## Known Limitations

- No sound output yet (`sound_timer` is tracked correctly, but no audio is played)
- SUPER-CHIP / XO-CHIP extensions are not implemented — this targets the original CHIP-8 spec

## Acknowledgments

- [Timendus/chip8-test-suite](https://github.com/Timendus/chip8-test-suite) for the invaluable test ROMs
- [Cowgod's CHIP-8 Technical Reference](http://devernay.free.fr/hacks/chip8/C8TECH10.HTM)
