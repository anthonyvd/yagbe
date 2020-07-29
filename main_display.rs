use crate::framebuffer::Framebuffer;

use sdl2::pixels::Color;
use std::convert::TryInto;

pub struct MainDisplay {
  c: sdl2::render::WindowCanvas,
  f: Option<Framebuffer>,
  name: String,
}

impl MainDisplay {
  pub fn new(video_subsystem: &sdl2::VideoSubsystem, n: &str, width: u32, height: u32) -> MainDisplay {
    let window = video_subsystem.window(n, width, height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string()).unwrap();

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string()).unwrap();

    canvas.set_draw_color(Color::RGB(255, 0, 0));
    canvas.clear();
    canvas.present();

    return MainDisplay { c: canvas, f: None, name: n.to_string() };
  }

  pub fn push_frame(&mut self, f: Framebuffer) {
    self.f = Some(f);
  }

  pub fn display_frame(&mut self) {
    self.c.clear();

    match &self.f {
      Some(frame) => {

        for x in 0..frame.w {
          for y in 0..frame.h {
            let color = frame.pixels[x][y];
            self.c.set_draw_color(Color::RGB(color, color, color));
            self.c.draw_point(sdl2::rect::Point::new(x.try_into().unwrap(), y.try_into().unwrap()));
          }
        }

        self.f = None;
      },
      None => {}
    }

    self.c.present();
  }
} 