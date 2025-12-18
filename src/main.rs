extern crate minifb;
extern crate rand;
extern crate rodio;

mod audio;
use audio::Audio;

mod window;
use window::Window;

mod cpu;
use cpu::CPU;

fn main() {
    println!("CHIP-8 emulator in Rust!");

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        return eprintln!("Usage: {} <rom-file-name>", args[0]);
    }

    let filename = String::from(&args[1]);

    let audio = match Audio::new() {
        Ok(a) => a,
        Err(err) => {return eprint!("Could not initialize audio device: {}", err);}
    };

    let win = match Window::new(&format!("chip8-rust: {}", filename)) {
        Ok(w) => w,
        Err(err) => {return eprint!("Could not initialize window: {}", err);}
    };

    let mut cpu = CPU::new(win, audio);

    println!("Loading ROM: {}", filename);
    match cpu.load_rom(filename) {
        Ok(()) => (),
        Err(err) => {return eprint!("Could not load ROM: {}", err.to_string());}
    };

    match cpu.run_loop() {
        Ok(()) => (),
        Err(err) => {return eprint!("CPU crashed: {}", err);}
    };
}
