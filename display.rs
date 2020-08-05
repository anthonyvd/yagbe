use crate::framebuffer::Framebuffer;

use sdl2::pixels::Color;
use std::convert::TryInto;

pub struct Display {
  c: sdl2::render::WindowCanvas,
  f: Option<Framebuffer>,
  name: String,
}

impl Display {
  pub fn new(video_subsystem: &sdl2::VideoSubsystem, n: &str, width: u32, height: u32) -> Display {
    let window = video_subsystem.window(n, width, height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string()).unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();

    return Display { c: canvas, f: None, name: n.to_string() };
  }

  pub fn push_frame(&mut self, f: Framebuffer) {
    self.f = Some(f);
  }

  pub fn display_frame(&mut self) {
    //self.c.clear();

    match &self.f {
      Some(frame) => {
        // TODO: we can probably just let this be the clear color
        self.c.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
        self.c.draw_points(&frame.blank_pixels[..]);

        self.c.set_draw_color(Color::RGB(0xAA, 0xAA, 0xAA));
        self.c.draw_points(&frame.light_pixels[..]);

        self.c.set_draw_color(Color::RGB(0x55, 0x55, 0x55));
        self.c.draw_points(&frame.medium_pixels[..]);

        self.c.set_draw_color(Color::RGB(0x00, 0x00, 0x00));
        self.c.draw_points(&frame.dark_pixels[..]);

        self.f = None;
      },
      None => {}
    }

    self.c.present();
  }
} 