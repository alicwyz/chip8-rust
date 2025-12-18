use minifb::Key;
use rand::{self, Rng};

use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{Duration, Instant};


use crate::audio::Audio;
use crate::window::Window;

// CPU Structure
const RAM_SIZE: usize = 4096;
const REGISTER_COUNT: usize = 16;
const STACK_SIZE: usize = 16;
const PROGRAM_START: usize = 0x200;
const RUNLOOP_TIMER: Duration = Duration::from_micros(16667); //~60fps
const INSTRUCTIONS_PER_FRAME: usize = 12;

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];


pub struct CPU {
    // RAM Memory
    ram: [u8; RAM_SIZE],
    // Registers
    v: [u8; REGISTER_COUNT], // V0, V1 ... VF
    i: usize, // Index
    pc: usize, // Progrm counter
    dt: u8, // Delay timer
    st: u8, // Sound timer
    sp: usize, // Stack pointer
    // Stack
    stack: [usize; STACK_SIZE],
    // Window and Audio
    win: Window,
    audio: Audio,
    // Keys
    keypad: [bool; 16],
    rng: rand::rngs::ThreadRng
}

impl CPU {
    // New CPU
    pub fn new(win: Window, audio: Audio) -> CPU {

        let mut new_cpu = CPU {
            ram: [0; RAM_SIZE],
            v: [0; REGISTER_COUNT],
            i: 0,
            pc: PROGRAM_START,
            dt: 0,
            st: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            win, audio,
            keypad: [false; 16],
            rng: rand::rng()
        };
        new_cpu.preload_ram();

        new_cpu
    }

    fn preload_ram(&mut self) {
        self.ram[0..FONTSET.len()].copy_from_slice(&FONTSET);
    }

    // Load ROM file
    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) -> std::io::Result<()> {
        // Read file and write to buffer RAM
        let mut file = File::open(path)?;
        file.read(&mut self.ram[0x200..])?;
        Ok(())
    }

    // Run loop
    pub fn run_loop(&mut self) -> Result<(), &str> {
        let mut current_time = Instant::now();

        while self.win.is_open() {
            // Break out of the loop
            if self.win.is_key_down(Key::Escape) {
                break;
            }

            // Check for system errors
            if self.pc > RAM_SIZE {
                return Err("Error: Address out of bounds");
            } else if self.sp > STACK_SIZE {
                return Err("Error: Stack overflow")
            }


            if current_time.elapsed() > RUNLOOP_TIMER {

                self.keypad = self.win.handle_key_events();

                for _ in 0..INSTRUCTIONS_PER_FRAME {
                    self.emulate_cycle();
                }

                self.update_timers();
                self.win.refresh();

                current_time = Instant::now();
            }
        }

        Ok(())
    }

    fn emulate_cycle(&mut self) {
        let opcode = self.fetch_opcode();
        self.execute_opcode(opcode);
    }

    fn fetch_opcode(&mut self) -> u16 {
        // u8 + u8 -> u16
        let high_byte: u16 = self.ram[self.pc] as u16;
        let low_byte: u16 = self.ram[self.pc + 1] as u16;

        return (high_byte << 8) | low_byte;
    }

    fn update_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            self.st -= 1;
            self.audio.play();

        } else {
            self.audio.pause();
        }

    }

    fn execute_opcode(&mut self, opcode: u16) {

        let nibbles = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8
        );

        let _pc_change = match nibbles {
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(opcode),
            (0x02, _, _, _) => self.op_2nnn(opcode),
            (0x03, _, _, _) => self.op_3xkk(opcode),
            (0x04, _, _, _) => self.op_4xkk(opcode),
            (0x05, _, _, 0x00) => self.op_5xy0(opcode),
            (0x06, _, _, _) => self.op_6xkk(opcode),
            (0x07, _, _, _) => self.op_7xkk(opcode),
            (0x08, _, _, 0x00) => self.op_8xy0(opcode),
            (0x08, _, _, 0x01) => self.op_8xy1(opcode),
            (0x08, _, _, 0x02) => self.op_8xy2(opcode),
            (0x08, _, _, 0x03) => self.op_8xy3(opcode),
            (0x08, _, _, 0x04) => self.op_8xy4(opcode),
            (0x08, _, _, 0x05) => self.op_8xy5(opcode),
            (0x08, _, _, 0x06) => self.op_8xy6(opcode),
            (0x08, _, _, 0x07) => self.op_8xy7(opcode),
            (0x08, _, _, 0x0e) => self.op_8xye(opcode),
            (0x09, _, _, 0x00) => self.op_9xy0(opcode),
            (0x0a, _, _, _) => self.op_annn(opcode),
            (0x0b, _, _, _) => self.op_bnnn(opcode),
            (0x0c, _, _, _) => self.op_cxkk(opcode),
            (0x0d, _, _, _) => self.op_dxyn(opcode),
            (0x0e, _, 0x09, 0x0e) => self.op_ex9e(opcode),
            (0x0e, _, 0x0a, 0x01) => self.op_exa1(opcode),
            (0x0f, _, 0x00, 0x07) => self.op_fx07(opcode),
            (0x0f, _, 0x00, 0x0a) => self.op_fx0a(opcode),
            (0x0f, _, 0x01, 0x05) => self.op_fx15(opcode),
            (0x0f, _, 0x01, 0x08) => self.op_fx18(opcode),
            (0x0f, _, 0x01, 0x0e) => self.op_fx1e(opcode),
            (0x0f, _, 0x02, 0x09) => self.op_fx29(opcode),
            (0x0f, _, 0x03, 0x03) => self.op_fx33(opcode),
            (0x0f, _, 0x05, 0x05) => self.op_fx55(opcode),
            (0x0f, _, 0x06, 0x05) => self.op_fx65(opcode),
            _ => self.pc += 2,
        };
    }

    // Clear the display
    fn op_00e0(&mut self) {
        self.win.clear_screen();
        self.pc += 2;
    }

    // Return from a subroutine
    fn op_00ee(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp] + 2;
    }

    // Jump to address NNN
    fn op_1nnn(&mut self, opcode: u16) {
        self.pc = (opcode & 0x0FFF) as usize;
    }

    // Call subroutine at NNN
    fn op_2nnn(&mut self, opcode: u16) {
        self.stack[self.sp] = self.pc;
        self.sp += 1;
        self.pc = (opcode & 0x0FFF) as usize;
    }

    // Skip next instruction if Vx == KK
    fn op_3xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        if self.v[x] == kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // Skip next instruction if Vx != KK
    fn op_4xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        if self.v[x] != kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // Skip next instruction if Vx == Vy
    fn op_5xy0(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        if self.v[x] == self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // Set Vx = kk
    fn op_6xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        self.v[x] = kk;
        self.pc += 2;
    }

    // Set Vx = Vx + kk
    fn op_7xkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;
        self.v[x] = self.v[x].wrapping_add(kk);
        self.pc += 2;
    }

    // Set Vx = Vy
    fn op_8xy0(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] = self.v[y];
        self.pc += 2;
    }

    // Set Vx = Vx OR Vy.
    fn op_8xy1(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] |= self.v[y];
        self.pc += 2;
    }

    // Set Vx = Vx AND Vy.
    fn op_8xy2(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] &= self.v[y];
        self.pc += 2;
    }

    // Set Vx = Vx XOR Vy.
    fn op_8xy3(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        self.v[x] ^= self.v[y];
        self.pc += 2;
    }

    // Set Vx = Vx + Vy, set VF = carry.
    fn op_8xy4(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (sum, carry) = self.v[x].overflowing_add(self.v[y]);
        self.v[0xF] = if carry { 1 } else { 0 };
        self.v[x] = sum;
        self.pc += 2;
    }

    // Set Vx = Vx - Vy, set VF = NOT borrow
    fn op_8xy5(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
        self.v[0xF] = if borrow { 0 } else { 1 };
        self.v[x] = result;
        self.pc += 2;
    }

    // Set Vx = Vx SHR 1.
    fn op_8xy6(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[0xF] = self.v[x] & 0x01;
        self.v[x] >>= 1;
        self.pc += 2;
    }

    // Set Vx = Vy - Vx, set VF = NOT borrow
    fn op_8xy7(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
        self.v[0xF] = if borrow { 0 } else { 1 };
        self.v[x] = result;
        self.pc += 2;
    }

    // Set Vx = Vx SHL 1
    fn op_8xye(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[0xF] = (self.v[x] & 0x80) >> 7;
        self.v[x] <<= 1;
        self.pc += 2;
    }

    // Skip next instruction if Vx != Vy
    fn op_9xy0(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;

        if self.v[x] != self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // Set I = nnn
    fn op_annn(&mut self, opcode: u16) {
        self.i = (opcode & 0x0FFF) as usize;
        self.pc += 2;
    }

    // Jump to location nnn + V0
    fn op_bnnn(&mut self, opcode: u16) {
        let nnn = (opcode & 0x0FFF) as usize;
        self.pc = nnn + self.v[0] as usize;
    }

    // Set Vx = random byte AND kk
    fn op_cxkk(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let kk = (opcode & 0x00FF) as u8;

        let random: u8 = self.rng.random::<u8>();
        self.v[x] = random & kk;
        self.pc += 2;
    }

    // Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
    fn op_dxyn(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let height = (opcode & 0x000F) as usize;
        let vx = self.v[x];
        let vy = self.v[y];

        let mut bytes_to_draw: Vec<u8> = Vec::new();

        for i in 0..height {
            bytes_to_draw.push(self.ram[self.i + i]);
        }

        self.v[0xF] = self.win.draw(&bytes_to_draw, vx, vy);
        self.pc += 2;
    }

    // Skip next instruction if key with the value of Vx is pressed
    fn op_ex9e(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        if self.keypad[self.v[x] as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // Skip next instruction if key with the value of Vx is not pressed
    fn op_exa1(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        if !self.keypad[self.v[x] as usize] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
    }

    // Set Vx = delay timer value
    fn op_fx07(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.v[x] = self.dt;
        self.pc += 2;
    }

    // Wait for a key press, store the value of the key in Vx
    fn op_fx0a(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;

        let key_pressed = self.keypad.iter().position(|&k| k);
        match key_pressed {
            Some(key) => {
                self.v[x] = key as u8;
                self.pc += 2;
            }
            None => {
                self.pc -= 2
            }
        }
    }

    // Set delay timer = Vx
    fn op_fx15(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.dt = self.v[x];
        self.pc += 2;
    }

    // Set sound timer = Vx
    fn op_fx18(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.st = self.v[x];
        self.pc += 2;
    }

    // Set I = I + Vx
    fn op_fx1e(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.i = self.i.wrapping_add(self.v[x] as usize);
        self.pc += 2;
    }

    // Set I = location of sprite for digit Vx
    fn op_fx29(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        self.i = self.v[x] as usize * 5; // each sprite has 5 bytes of data
        self.pc += 2;
    }

    // Store BCD representation of Vx in memory locations I, I+1, and I+2
    fn op_fx33(&mut self, opcode: u16) {
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let vx = self.v[x];

        self.ram[self.i] = vx / 100;
        self.ram[self.i + 1] = (vx % 100) / 10;
        self.ram[self.i + 2] = vx % 10;
        self.pc += 2;
    }

    // Stores V0 to VX in memory starting at address I
    fn op_fx55(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;

        for index in 0..=x as usize {
            self.ram[self.i+ index] = self.v[index];
        }
        self.pc += 2;
    }

    // Fills V0 to VX with values from memory starting at address I
    fn op_fx65(&mut self, opcode: u16) {
        let x = (opcode & 0x0F00) >> 8;

        for i in 0..=x as usize {
            self.v[i] = self.ram[self.i + i];
        }

        self.pc += 2;
    }
}

