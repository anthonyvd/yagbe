use crate::memory::Memory;
use crate::framebuffer::Framebuffer;

pub struct Ppu {
  lx: u16,
  frame_count: u64,
}

impl Ppu {
  pub fn new() -> Ppu {
    return Ppu { lx: 0, frame_count: 0 };
  }

  pub fn tick(&mut self, memory: &mut Memory) -> std::option::Option<Framebuffer> {
    // Let's assume that each tick is one TCycle for now
    if self.lx > 455 {
      self.lx = 0;
      memory[0xFF44] = (memory[0xFF44] + 1) % 154;
    }

    self.lx += 1;

    if memory[0xFF44] == 0 {
      // new frame
      self.frame_count += 1;
      let mut frame = Framebuffer::new(160, 144, 0xFF);

      // TODO: This is incorrect, the full frame shouldn't be generated at the end of the cycles.
      for tile_x in 0..20 {
        for tile_y in 0..18 {
          let tile_id = memory[0x9800 + tile_y * 32 + tile_x];
          frame.draw_tile(memory, tile_id, (tile_x * 8).into(), (tile_y * 8).into());
        }
      }

      return Some(frame);
    }

    return None;
  }
}