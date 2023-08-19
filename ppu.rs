use crate::memory::Memory;
use crate::display::Display;

pub struct Ppu {
  lx: u16,
  frame_count: u64,
}

impl Ppu {
  pub fn new() -> Ppu {
    return Ppu { lx: 0, frame_count: 0 };
  }

  pub fn tick(&mut self, memory: &mut Memory, display: &mut Display) -> bool {
    // Let's assume that each tick is one TCycle for now
    if self.lx > 455 {
      self.lx = 0;
      memory[0xFF44] = (memory[0xFF44] + 1) % 154;
      memory[0xFF45] = (memory[0xFF45] + 1) % 154;
    }

    self.lx += 1;

    if memory[0xFF44] == 0 {
      memory[0xFF0F] = memory[0xFF0F] | 0x01;
      // new frame
      self.frame_count += 1;

      // TODO: This is incorrect, the full frame shouldn't be generated at the end of the cycles.
      // TODO: This isn't how this work, see FIFOs on pandocs
      for tile_x in 0..20 {
        for tile_y in 0..18 {
          let tile_id = memory[0x9800 + tile_y * 32 + tile_x];
          display.draw_tile(memory, tile_id, (tile_x * 8).into(), (tile_y * 8).into());
        }
      }

      return true;
    }

    return false;
  }
}