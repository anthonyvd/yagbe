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
      return Some(Framebuffer::new(160, 144, if self.frame_count % 60 < 30 { 0x00 } else { 0xFF }));
    }

    return None;
  }
}