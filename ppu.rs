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
  curr_line_objects: Vec<ObjectAttribute>
}

impl Ppu {
  pub fn new() -> Ppu {
    return Ppu { lx: 0, stat_sent_last_mode: false, window_line: 0, curr_line_objects: vec![] };
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
      memory[0xFF41] = memory[0xFF41] & !0b100;
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

        has_frame = true;
      } else {
        // OAM search for 80 dots
        // Set STAT mode flag to OAM search
        memory[0xFF41] = (memory[0xFF41] & !0b11) | 2;

        // Send STAT if enabled
        if memory[0xFF41] & 0b100000 != 0 {
          memory[0xFF0F] |= 0b10;
        }

        // Perform all of OAM search here if obj is enabled
        self.curr_line_objects.clear();
        if memory[0xFF40] & 0b10 != 0 {
          for i in (0xFE00..0xFE9F).step_by(4) {
            let is_16_tall = memory[0xFF40] & 0b100 == 1;

            if memory[i] >= 160 {
              continue;
            }

            let y_min = memory[i];
            let y_max = y_min + if is_16_tall { 16 } else { 8 };

            if memory[0xFF44] + 16 >= y_min && memory[0xFF44] + 16 < y_max {
              // Object is on line
              self.curr_line_objects.push(ObjectAttribute {
                y: memory[i],
                x: memory[i + 1],
                tile_idx: memory[i + 2] & if is_16_tall { 0xFE } else { 0xFF },
                attributes: memory[i + 3],
              });

              if self.curr_line_objects.len() == 10 {
                break;
              }
            }
          }
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

          // Approximate whatever shittery the PPU and Pixel FIFOs do
          // by generating a full line right now.
          let screen_y = memory[0xFF44] as u16;
          for screen_x in 0u16..160u16 {
            let is_window =
              memory[0xFF40] & 0b100000 != 0 &&
              (screen_x + 7) >= memory[0xFF4B] as u16 &&
              screen_y >= memory[0xFF4A] as u16;

            let scy = if is_window { 0 } else { memory[0xFF42] as u16 };
            let scx = if is_window { 0 } else { memory[0xFF43] as u16 };

            let mut color = 0;
            
            // We only draw window/bg for now, so draw blank if they are disabled.
            if memory[0xFF40] & 1 != 0 {
              let tile_id = if is_window {
                memory[window_tile_map_addr + 
                       ((screen_y - (memory[0xFF4A] as u16)) / 8) * 32 + 
                       (7 + screen_x - memory[0xFF4B] as u16) / 8]
              } else {
                memory[bg_tile_map_addr + 
                       ((screen_y + scy) / 8) * 32 + 
                       (screen_x + scx) / 8]
              };

              let tile_addr = if memory[0xFF40] & 0b10000 == 0 {
                // 0x8800 addressing
                0x8800 + tile_id.wrapping_add(128) as u16 * 16
              } else {
                // 0x8000 addressing
                0x8000 + tile_id as u16 * 16
              };

              let y_offset = if is_window {
                (screen_y - (memory[0xFF4A] as u16)) % 8
              } else {
                (screen_y + scy) % 8
              };

              let x_offset = if is_window {
                (7 + screen_x - memory[0xFF4B] as u16) % 8
              } else {
                (screen_x + scx) % 8
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
              if screen_x + 8 >= obj.x as u16 && 
                 screen_x < obj.x as u16 && 
                 best_obj_prio > obj.x {
                if obj.attributes & 0x80 != 0 && bg_window_color != 0 {
                  continue;
                }

                best_obj_prio = obj.x;

                let mut y_offset = screen_y + 16 - obj.y as u16;
                let x_offset = screen_x + 8 - obj.x as u16;

                // No need to check for the y coord to be inside the object, it wouldn't be in the vector if it wasn't
                let tile_addr = 0x8000 + (obj.tile_idx as u16 + if y_offset > 7 { 1 } else { 0 }) * 16;
                y_offset = y_offset % 8;

                let h_flip = obj.attributes & 0b100000 != 0;
                let v_flip = obj.attributes & 0b1000000 != 0;

                let lsb = if v_flip {
                  memory[tile_addr + 15 - (2 * y_offset as u16)]
                } else {
                  memory[tile_addr + (2 * y_offset as u16)]
                };

                let msb = if v_flip {
                  memory[tile_addr + 15 - (2 * y_offset as u16) - 1]
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
            
            // TODO: lookup palettes
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