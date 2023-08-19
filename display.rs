use crate::memory::Memory;

use sdl2::pixels::Color;

const GB_WIDTH: u32 = 160;
const GB_HEIGHT: u32 = 144;

pub struct Display {
  c: sdl2::render::WindowCanvas,
}

impl Display {
  pub fn new(video_subsystem: &sdl2::VideoSubsystem, n: &str, width: u32, height: u32) -> Display {
    if width % GB_WIDTH != 0 || height % GB_HEIGHT != 0 {
      panic!("Window dimensions not multiple of 160x144");
    }

    let window = video_subsystem.window(n, width, height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string()).unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.set_scale((width / GB_WIDTH) as f32, (height / GB_HEIGHT) as f32).unwrap();
    canvas.clear();
    canvas.present();

    return Display { c: canvas };
  }

  pub fn present(&mut self) {
    self.c.present();
  }

  pub fn draw_tile(&mut self, memory: &mut Memory, tile_id: u8, x: i32, y: i32) {
    let tile_address: u16 = 0x8000 + ((tile_id as u16) * 16);

    for b in (tile_address..(tile_address + 16)).step_by(2) {
      let row: i32 = ((b as u16 - tile_address) / 2) as i32;
      let lsb = memory[b];
      let msb = memory[b + 1];

      for j in 0..8 {
        let bit_mask = 0b10000000 >> j;
        let pix = (((msb & bit_mask) >> (7 - j)) << 1) | ((lsb & bit_mask) >> (7 - j));

        let p: sdl2::rect::Point = sdl2::rect::Point::new(x + j as i32, y + row);
        match pix {
          0b00 => {
            self.c.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
            self.c.draw_point(p).unwrap();
          },
          0b01 => {
            self.c.set_draw_color(Color::RGB(0xAA, 0xAA, 0xAA));
            self.c.draw_point(p).unwrap();
          },
          0b10 => {
            self.c.set_draw_color(Color::RGB(0x55, 0x55, 0x55));
            self.c.draw_point(p).unwrap();
          },
          0b11 => {
            self.c.set_draw_color(Color::RGB(0x00, 0x00, 0x00));
            self.c.draw_point(p).unwrap();
          },
          _ => unimplemented!(),
        };
      }
    }
  }
} 