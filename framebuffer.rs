pub struct Framebuffer {
  pub pixels: std::vec::Vec<std::vec::Vec<u8>>,
  pub w: usize,
  pub h: usize,
}

impl Framebuffer {
  pub fn new(width: usize, height: usize, clear_color: u8) -> Framebuffer {
    return Framebuffer { pixels: vec![vec![clear_color; height]; width], w: width, h: height };
  }
}
