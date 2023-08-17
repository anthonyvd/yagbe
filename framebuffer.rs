use crate::memory::Memory;

pub struct Framebuffer {
  pub blank_pixels: std::vec::Vec<sdl2::rect::Point>,
  pub light_pixels: std::vec::Vec<sdl2::rect::Point>,
  pub medium_pixels: std::vec::Vec<sdl2::rect::Point>,
  pub dark_pixels: std::vec::Vec<sdl2::rect::Point>,
  pub w: usize,
  pub h: usize,
}

impl Framebuffer {
  pub fn new(width: usize, height: usize) -> Framebuffer {
    return Framebuffer { 
      blank_pixels: vec![], 
      light_pixels: vec![], 
      medium_pixels: vec![], 
      dark_pixels: vec![], 
      w: width,
      h: height
    };
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
          0b00 => self.blank_pixels.push(p),
          0b01 => self.light_pixels.push(p),
          0b10 => self.medium_pixels.push(p),
          0b11 => self.dark_pixels.push(p),
          _ => unimplemented!(),
        };
      }
    }
  }
}
