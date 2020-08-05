mod cartridge;
mod cpu;
mod registers;
mod memory_utils;
mod utils;
mod ppu;
mod memory;
mod framebuffer;
mod display;

use cartridge::Cartridge;
use std::path::Path;
use cpu::Cpu;
use ppu::Ppu;
use memory::Memory;
use framebuffer::Framebuffer;
use display::Display;
use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() -> Result<(), String>  {
  let cart = Cartridge::load(Path::new("./tetris.gb"));
  println!("Loaded {}", cart.title);

  let mut memory = Memory::new(&cart.data);
  let mut cpu = Cpu::new();
  let mut ppu = Ppu::new();

  let sdl_context = sdl2::init().unwrap();
  let video_subsystem = sdl_context.video().unwrap();
  //let mut tilemap = Display::new(&video_subsystem, "Tiles", 160, 160);
  let mut main_display = Display::new(&video_subsystem, "Main", 160, 144);

  let mut before = Instant::now();
  let mut event_pump = sdl_context.event_pump()?;

  'running: loop {
    cpu.tick(&mut memory);
    let possible_frame = ppu.tick(&mut memory);

    if possible_frame.is_some() {
      for event in event_pump.poll_iter() {
          match event {
              Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                  break 'running
              },
              _ => {}
          }
      }

      //print!("Pre Frame {:?} ", before.elapsed());
      let mut tiles_framebuffer = Framebuffer::new(160, 160, 0x00);
      
      for i in (0x8000..0x97FF).step_by(16) {
        let tile_index = (i - 0x8000) / 16;
        let tile_row = tile_index / 20;
        let tile_col = tile_index % 20;

        for b in (i..(i + 16)).step_by(2) {
          let row = (b - i) / 2;
          let lsb = memory[b];
          let msb = memory[b + 1];

          for j in 0..8 {
            let bit_mask = 0b10000000 >> j;
            let pix = (((msb & bit_mask) >> (7 - j)) << 1) | ((lsb & bit_mask) >> (7 - j));

            let p: sdl2::rect::Point = sdl2::rect::Point::new((tile_col * 8 + j).into(), (tile_row * 8 + row).into());
            match pix {
              0b00 => tiles_framebuffer.blank_pixels.push(p),
              0b01 => tiles_framebuffer.light_pixels.push(p),
              0b10 => tiles_framebuffer.medium_pixels.push(p),
              0b11 => tiles_framebuffer.dark_pixels.push(p),
              _ => unimplemented!(),
            };
          }
        }
      }

      //tilemap.push_frame(tiles_framebuffer);
      //tilemap.display_frame();

      main_display.push_frame(possible_frame.unwrap());
      main_display.display_frame();
      //print!("Post Frame {:?}", before.elapsed());
      //println!("");
      before = Instant::now();
    }
  }
  return Ok(())
}