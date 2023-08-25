use crate::memory::Memory;
use crate::display::Display;

use sdl2::pixels::Color;

struct ObjectAttribute {
  y: u8,
  x: u8,
  tile_idx: u8,
  attributes: u8,
}

pub struct Ppu {
  lx: u16,
  // TODO: use this to prevent interrupts when one was sent for the previous mode
  stat_sent_last_mode: bool,
  window_line: u16,
  drew_window_on_line: bool,
  curr_line_objects: Vec<ObjectAttribute>,
  obj_size_16: bool,
}

impl Ppu {
  pub fn new() -> Ppu {
    return Ppu { lx: 0, stat_sent_last_mode: false, window_line: 0, drew_window_on_line: false,
      curr_line_objects: vec![], obj_size_16: true };
  }

  // each tick is one dot, so 1 TCycle
  pub fn tick(&mut self, memory: &mut Memory, display: &mut Display) -> bool {
    let mut has_frame = false;

    let mode = memory[0xFF41] & 0b11;
    let mut window_on_line = false;
    match mode {
      0 => {
        // HBLANK, do nothing
      },
      1 => {
        // VBLANK, do nothing
      },
      2 => {
        if self.lx == 79 {
          // Perform all of OAM search on the last dot if obj is enabled/
          // TODO: This is incorrect, but since some things write to registers during OAM search,
          //       it's probably more useful to do the search at the end than at the start of the interval.
          self.curr_line_objects.clear();
          if memory[0xFF40] & 0b10 != 0 {
            for i in (0xFE00..0xFE9F).step_by(4) {
              self.obj_size_16 = (memory[0xFF40] & 0b100) != 0;

              if memory[i] >= 160 {
                continue;
              }

              let y_min = memory[i];
              let y_max = y_min + if self.obj_size_16 { 16 } else { 8 };

              if memory[0xFF44] + 16 >= y_min && memory[0xFF44] + 16 < y_max {
                // Object is on line
                self.curr_line_objects.push(ObjectAttribute {
                  y: memory[i],
                  x: memory[i + 1],
                  tile_idx: memory[i + 2] & if self.obj_size_16 { 0xFE } else { 0xFF },
                  attributes: memory[i + 3],
                });

                if self.curr_line_objects.len() == 10 {
                  break;
                }
              }
            }
          }
        }
      },
      3 => {
        if self.lx == 80 {
          // Draw pixels. This isn't accurate.
          let _tile_data_addr: u16 = if memory[0xFF40] & 0b10000 == 0 {
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

          let mut debug_pixel_color: Option<Color> = None;

          // Approximate whatever shittery the PPU and Pixel FIFOs do
          // by generating a full line right now.
          let screen_y = memory[0xFF44];
          for screen_x in 0u8..160u8 {
            let is_window =
              memory[0xFF40] & 0b100000 != 0 &&
              (screen_x + 7) >= memory[0xFF4B] &&
              screen_y >= memory[0xFF4A];

            if is_window {
              self.drew_window_on_line = true;
            }

            let scy = if is_window { 0 } else { memory[0xFF42] };
            let scx = if is_window { 0 } else { memory[0xFF43] };

            let mut color = 0;

            // Draw BG/Window
            if memory[0xFF40] & 1 != 0 {
              let tile_id = if is_window {
                memory[window_tile_map_addr + 
                       (self.window_line / 8) * 32 + 
                       (7 + screen_x - memory[0xFF4B]) as u16 / 8]
              } else {
                memory[bg_tile_map_addr + 
                       (screen_y.wrapping_add(scy) as u16 / 8) * 32 + 
                       screen_x.wrapping_add(scx) as u16 / 8]
              };

              let tile_addr = if memory[0xFF40] & 0b10000 == 0 {
                // 0x8800 addressing
                0x8800 + tile_id.wrapping_add(128) as u16 * 16
              } else {
                // 0x8000 addressing
                0x8000 + tile_id as u16 * 16
              };

              let y_offset = if is_window {
                (self.window_line % 8) as u8
              } else {
                screen_y.wrapping_add(scy) % 8
              };

              let x_offset = if is_window {
                (7 + screen_x - memory[0xFF4B]) % 8
              } else {
                screen_x.wrapping_add(scx) % 8
              };

              let lsb = memory[tile_addr + (2 * y_offset as u16)];
              let msb = memory[tile_addr + (2 * y_offset as u16) + 1];

              let mask = 0b10000000 >> x_offset;
              let color_idx = ((lsb & mask) >> (7 - x_offset)) |
                          ((msb & mask) >> (7 - x_offset) << 1);

              color = (memory[0xFF47] >> (color_idx * 2)) & 0b11;
            }

            let bg_window_color = color;

            // Objects
            // TODO: obj to bg priority
            let mut best_obj_prio = 0xFF;
            for obj in self.curr_line_objects.iter() {
              if screen_x + 8 >= obj.x && 
                 screen_x < obj.x && 
                 best_obj_prio > obj.x {
                if obj.attributes & 0x80 != 0 && bg_window_color != 0 {
                  continue;
                }

                best_obj_prio = obj.x;

                let mut y_offset = screen_y + 16 - obj.y;
                let x_offset = screen_x + 8 - obj.x;

                let h_flip = obj.attributes & 0b100000 != 0;
                let v_flip = obj.attributes & 0b1000000 != 0;

                let corrected_tile_id = obj.tile_idx as u16 +
                  if (!v_flip && y_offset > 7) || (self.obj_size_16 && v_flip && y_offset < 8) { 
                    1 
                  } else { 
                    0
                  };

                // No need to check for the y coord to be inside the object, it wouldn't be in the vector if it wasn't
                let tile_addr = 0x8000 + corrected_tile_id * 16;
                y_offset = y_offset % 8;

                let lsb = if v_flip {
                  memory[tile_addr + 15 - (2 * y_offset as u16) - 1]
                } else {
                  memory[tile_addr + (2 * y_offset as u16)]
                };

                let msb = if v_flip {
                  memory[tile_addr + 15 - (2 * y_offset as u16)]
                } else {
                  memory[tile_addr + (2 * y_offset as u16) + 1]
                };

                let palette_addr = if obj.attributes & 0b10000 == 0 {
                  0xFF48
                } else {
                  0xFF49
                };

                if h_flip {
                  let mask = 1 << x_offset;
                  let color_idx = ((lsb & mask) >> x_offset) |
                              ((msb & mask) >> x_offset << 1);

                  if color_idx != 0 {
                    color = (memory[palette_addr] >> (color_idx * 2)) & 0b11;
                  }
                } else {
                  let mask = 0b10000000 >> x_offset;
                  let color_idx = ((lsb & mask) >> (7 - x_offset)) |
                              ((msb & mask) >> (7 - x_offset) << 1);
                  if color_idx != 0 {
                    color = (memory[palette_addr] >> (color_idx * 2)) & 0b11;
                  }
                }
              }
            }
            
            match debug_pixel_color {
              Some(c) => {
                let p: sdl2::rect::Point = sdl2::rect::Point::new(screen_x as i32, screen_y as i32);
                display.c.set_draw_color(c);
                display.c.draw_point(p).unwrap();
              },
              None => {
                let p: sdl2::rect::Point = sdl2::rect::Point::new(screen_x as i32, screen_y as i32);
                match color {
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
              },
            }
          }
        }
      },
      _ => {
        panic!("Impossible PPU mode");
      },
    }

    self.lx = (self.lx + 1) % 456;
    if self.lx == 0 {
      memory.set(0xFF44, (memory[0xFF44] + 1) % 154);
      if self.drew_window_on_line {
        self.window_line += 1;
      }
      if memory[0xFF44] == 0 {
        self.window_line = 0;
      }
      self.drew_window_on_line = false;
    }

    // Set STAT LYC=LY flag if LY == LYC
    if memory[0xFF44] == memory[0xFF45] {
      memory.set(0xFF41, memory[0xFF41] | 0b100);
      if self.lx == 0 {
        // Also send the STAT interrupt if the source is enabled
        if memory[0xFF41] & 0b1000000 != 0 {
          memory.set(0xFF0F, memory[0xFF0F] | 0b10);
        }
      }
    } else {
      memory.set(0xFF41, memory[0xFF41] & !0b100);
    }

    if self.lx == 0 {
      if memory[0xFF44] == 144 {
        // VBLANK request interrupt
        memory.set(0xFF0F, memory[0xFF0F] | 0x01);
        // Set STAT mode flag to VBLANK
        memory.set(0xFF41, (memory[0xFF41] & !0b11) | 1);

        // Also send the STAT interrupt if it's enabled
        if memory[0xFF41] & 0b10000 != 0 {
          memory.set(0xFF0F, memory[0xFF0F] | 0b10);
        }

        has_frame = true;
      } else {
        // OAM search for 80 dots
        // Set STAT mode flag to OAM search
        memory.set(0xFF41, (memory[0xFF41] & !0b11) | 2);

        // Send STAT if enabled
        if memory[0xFF41] & 0b100000 != 0 {
          memory.set(0xFF0F, memory[0xFF0F] | 0b10);
        }
      }
    } else if self.lx == 80 && memory[0xFF44] < 144 {
      // Mode 3
      // Set STAT mode flag to mode 3
      memory.set(0xFF41, (memory[0xFF41] & !0b11) | 3);
    } else if self.lx == 369 && memory[0xFF44] < 144 {
      // HBLANK, TODO: this isn't correct when mode 3 is lengthened
      // Set STAT mode flag to HBLANK
      memory.set(0xFF41, memory[0xFF41] & !0b11);

      // Send STAT if enabled
      if memory[0xFF41] & 0b1000 != 0 {
        memory.set(0xFF0F, memory[0xFF0F] | 0b10);
      }
    }

    return has_frame;
  }
}