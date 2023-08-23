use crate::memory::Memory;
use crate::display::Display;

use sdl2::pixels::Color;

pub struct Ppu {
  lx: u16,
  stat_sent_last_mode: bool,
}

impl Ppu {
  pub fn new() -> Ppu {
    return Ppu { lx: 0, stat_sent_last_mode: false };
  }

  // each tick is one dot, so 1 TCycle
  pub fn tick(&mut self, memory: &mut Memory, display: &mut Display) -> bool {
    let mut has_frame = false;

    // Set STAT LYC=LY flag if LY == LYC
    if memory[0xFF44] == memory[0xFF45] {
      memory[0xFF41] = (memory[0xFF41] & !0b100) | 0b100;
      if self.lx == 0 {
        // Also send the STAT interrupt if the source is enabled
        if memory[0xFF41] & 0b1000000 != 0 {
          memory[0xFF0F] |= 0b10;
        }
      }
    } else {
      memory[0xFF41] = (memory[0xFF41] & !0b100);
    }

    if self.lx == 0 {
      if memory[0xFF44] == 144 {
        // VBLANK request interrupt
        memory[0xFF0F] = memory[0xFF0F] | 0x01;
        // Set STAT mode flag to VBLANK
        memory[0xFF41] = (memory[0xFF41] & !0b11) | 1;

        // Also send the STAT interrupt if it's enabled
        if memory[0xFF41] & 0b10000 != 0 {
          memory[0xFF0F] |= 0b10;
        }
/*
        display.c.clear();
        // TODO: This is incorrect, the full frame shouldn't be generated at the end of the cycles.
        // TODO: This isn't how this work, see FIFOs on pandocs
        for tile_x in 0..20 {
          for tile_y in 0..18 {
            let tile_id = memory[0x9800 + tile_y * 32 + tile_x];
            display.draw_tile(memory, tile_id, (tile_x * 8).into(), (tile_y * 8).into());
          }
        }
*/
        has_frame = true;
      } else {
        // OAM search for 80 dots
        // Set STAT mode flag to OAM search
        memory[0xFF41] = (memory[0xFF41] & !0b11) | 2;

        // Send STAT if enabled
        if memory[0xFF41] & 0b100000 != 0 {
          memory[0xFF0F] |= 0b10;
        }
      }
    } else if self.lx == 80 && memory[0xFF44] < 144 {
      // Mode 3
      // Set STAT mode flag to mode 3
      memory[0xFF41] = (memory[0xFF41] & !0b11) | 3;
    } else if self.lx == 252 && memory[0xFF44] < 144 {
      // HBLANK, TODO: this isn't correct when mode 3 is lengthened
      // Set STAT mode flag to HBLANK
      memory[0xFF41] = (memory[0xFF41] & !0b11) | 0;

      // Send STAT if enabled
      if memory[0xFF41] & 0b1000 != 0 {
        memory[0xFF0F] |= 0b10;
      }
    }

    let mode = memory[0xFF41] & 0b11;
    match mode {
      0 => {
        // HBLANK, do nothing
      },
      1 => {
        // VBLANK, do nothing
      },
      2 => {
        // Searching OAM, not implemented yet
      },
      3 => {
        if self.lx == 80 {
          // Draw pixels. This isn't accurate.
          let tile_data_addr: u16 = if memory[0xFF40] & 0b10000 == 0 {
            0x8800
          } else { 
            0x8000
          };
          let bg_tile_map_addr: u16 = if memory[0xFF40] & 0b1000 == 0 {
            0x9800
          } else {
            0x9C00
          };
          let window_tile_map_addr: u16 = if memory[0xFF40] & 0b1000000 == 0 {
            0x9800
          } else {
            0x9C00
          };
/*
          if tile_data_addr != 0x8000 { panic!("tile_data_addr"); }
          if bg_tile_map_addr != 0x9800 { panic!("bg_tile_map_addr"); }
*/
          // Approximate whatever shittery the PPU and Pixel FIFOs do
          // by generating a full line right now.
          let y = memory[0xFF44] as u16;
          assert!(y < 144);

/*
          assert_eq!(0, scx);
          assert_eq!(0, scy);
*/
          for x in 0..160 {
            let is_window = memory[0xFF40] & 0b100000 != 0 &&
                (x + 7) >= memory[0xFF4B] as u16 &&
                y >= memory[0xFF4A] as u16;

            let base_map_addr = if is_window {
              window_tile_map_addr
            } else {
              bg_tile_map_addr
            };


            let scy = if is_window { 0 } else { memory[0xFF42] as u16 };
            let scx = if is_window { 0 } else { memory[0xFF43] as u16 };

            let mut color_idx = 0;
            
            // We only draw window/bg for now, so draw blank if they are disabled.
            if memory[0xFF40] & 1 != 0 {
              // TODO: internal line counter
              let y_tile_offset = (((y + scy) / 8) * 32);
              let x_tile_offset = ((x + scx) / 8);

              let tile_id = memory[
                base_map_addr + y_tile_offset + x_tile_offset];
              let x_offset = (x + scx) % 8;
              let y_offset = (y + scy) % 8;

              let tile_addr = if tile_data_addr == 0x8800 {
                tile_data_addr + (tile_id.wrapping_add(128) as u16) * 16
              } else {
                tile_data_addr + (tile_id as u16) * 16
              };

              let lsb = memory[tile_addr + (2 * y_offset)];
              let msb = memory[tile_addr + (2 * y_offset) + 1];

              let mask = 0b10000000 >> x_offset;
              color_idx = ((lsb & mask) >> (7 - x_offset)) |
                          ((msb & mask) >> (7 - x_offset) << 1);
            }
            
            let p: sdl2::rect::Point = sdl2::rect::Point::new(x as i32, y as i32);
            match color_idx {
              0b00 => {
                display.c.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
                display.c.draw_point(p).unwrap();
              },
              0b01 => {
                display.c.set_draw_color(Color::RGB(0xAA, 0xAA, 0xAA));
                display.c.draw_point(p).unwrap();
              },
              0b10 => {
                display.c.set_draw_color(Color::RGB(0x55, 0x55, 0x55));
                display.c.draw_point(p).unwrap();
              },
              0b11 => {
                display.c.set_draw_color(Color::RGB(0x00, 0x00, 0x00));
                display.c.draw_point(p).unwrap();
              },
              _ => panic!("Impossible color"),
            };
          }
        }
      },
      _ => {
        panic!("Impossible PPU mode");
      },
    }

    self.lx = (self.lx + 1) % 456;
    if self.lx == 0 {
      memory[0xFF44] = (memory[0xFF44] + 1) % 154;
    }

    return has_frame;
  }
}