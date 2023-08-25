use sdl2::pixels::Color;

pub struct Display {
    pub c: sdl2::render::WindowCanvas,
}

impl Display {
    pub fn new(
        video_subsystem: &sdl2::VideoSubsystem,
        n: &str,
        width: u32,
        height: u32,
        scale_x: f32,
        scale_y: f32,
    ) -> Display {
        let window = video_subsystem
            .window(n, width, height)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        let mut canvas = window
            .into_canvas()
            .build()
            .map_err(|e| e.to_string())
            .unwrap();

        canvas.set_draw_color(Color::RGB(255, 0, 0));
        canvas.set_scale(scale_x, scale_y).unwrap();
        canvas.clear();
        canvas.present();

        return Display { c: canvas };
    }

    pub fn present(&mut self) {
        self.c.present();
    }
}
