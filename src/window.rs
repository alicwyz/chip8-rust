use minifb::{
    Key,
    WindowOptions,
    Scale,
    Error
};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const PX_OFF: u32 = 0x000000;
const PX_ON: u32 = 0xFFFFFF;

pub struct Window {
    win: minifb::Window,
    framebuffer: [u32; WIDTH * HEIGHT]
} 

impl Window {
    // Open new window
    pub fn new(title: &str) -> Result<Window, Error> {

        let mut win = minifb::Window::new(
            title, WIDTH, HEIGHT,
            WindowOptions {scale: Scale::X16, ..WindowOptions::default() }
        )?;

        win.set_target_fps(60);

        Ok(Window { win, framebuffer: [PX_OFF; WIDTH * HEIGHT]})

    }

    // Map keyboard to chip8 keys
    pub fn handle_key_events(&self) -> [bool; 16] {

        let mut keys = [false; 16];

        // 1 2 3 C
        // 4 5 6 D
        // 7 8 9 E
        // A 0 B F  

        self.win.get_keys().iter().for_each(|k: &Key| {
            match k {
                Key::Key1 => keys[0x1] = true,
                Key::Key2 => keys[0x2] = true,
                Key::Key3 => keys[0x3] = true,
                Key::Key4 => keys[0xc] = true,
                Key::Q => keys[0x4] = true,
                Key::W => keys[0x5] = true,
                Key::E => keys[0x6] = true,
                Key::R => keys[0xd] = true,
                Key::A => keys[0x7] = true,
                Key::S => keys[0x8] = true,
                Key::D => keys[0x9] = true,
                Key::F => keys[0xe] = true,
                Key::Z => keys[0xa] = true,
                Key::X => keys[0x0] = true,
                Key::C => keys[0xb] = true,
                Key::V => keys[0xf] = true,
                _ => ()
            }
        });

        keys
    }

    // Utilities
    pub fn is_key_down(&self, key: Key) -> bool {
        self.win.is_key_down(key)
    }

    pub fn is_open(&self) -> bool {
        self.win.is_open()
    }

    pub fn clear_screen(&mut self) {
        self.framebuffer = self.framebuffer.map(|_| PX_OFF);
    }

    // Draw window

    pub fn refresh(&mut self) {
        self.win.update_with_buffer(&self.framebuffer, WIDTH, HEIGHT).unwrap();
    }

    pub fn draw(&mut self, bytes: &Vec<u8>, init_x: u8, init_y: u8) -> u8{

        // 0 1 0 0 0 1 1 1
        // 1 1 1 1 0 0 0 0
        // etc

        let mut vf: u8 = 0;

        // Loop bytes vector  
        for (vector_index, byte) in bytes.iter().enumerate() {
            // Loop within each byte
            for byte_index in 0..8 { //0-7

                let x = (init_x as usize + byte_index) % WIDTH;
                let y = (init_y as usize + vector_index) % HEIGHT;

                let coord = (y * WIDTH) + x;

                // Check if this bit is set in the sprite
                let sprite_pixel_set = (byte & (1 << (7 - byte_index))) != 0;
            
                 // Only process if sprite pixel is set
                if sprite_pixel_set {
                    let display_pixel_on = self.framebuffer[coord] == PX_ON;
                
                    // XOR: toggle the pixel
                    self.framebuffer[coord] = if display_pixel_on { PX_OFF } else { PX_ON };
                    
                    // Set VF if we're turning off a pixel (collision)
                    if display_pixel_on {
                        vf = 1;
                    }
                }
            }
        }

        vf
    }

}

