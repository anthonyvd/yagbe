mod cartridge;
mod cpu;
mod registers;
mod memory_utils;
mod utils;
mod ppu;
mod memory;
mod display;
mod opcodes;

use cartridge::Cartridge;
use std::path::Path;
use cpu::Cpu;
use ppu::Ppu;
use memory::Memory;
use display::Display;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::collections::HashSet;

use std::time::Instant;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

enum ConsoleSignal {
  Quit,
  BreakpointHit(u16),
}

enum DebugState {
  Running,
  Resuming,
  Stopped,
  Quitting,
}

struct Console {
  memory: Memory,
  cpu: Cpu,
  ppu: Ppu,

  main_display: display::Display,
  tilemap_display: display::Display,

  event_pump: sdl2::EventPump,

  breakpoints: HashSet<u16>,
  debug_state: DebugState,

  tx: mpsc::Sender<ConsoleSignal>,

  last_frame: std::time::Instant,
  current_tick: u32,
}

impl Console {
  pub fn new(cart_path: &Path, tx: mpsc::Sender<ConsoleSignal>) -> Console {
    let cart = Cartridge::load(cart_path);
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let tilemap_display = Display::new(&video_subsystem, "Tilemap", 320, 288);
    let main_display = Display::new(&video_subsystem, "Main", 320, 288);
    let event_pump = sdl_context.event_pump().expect("start event_pump");
    let mut mem = Memory::new(&cart.data);
    mem[0xFF0F] = 0xE1;
    mem[0xFFFF] = 0x00;

    return Console { 
      memory: mem, 
      cpu: Cpu::new(), ppu: Ppu::new(), 
      tx: tx, 
      main_display: main_display,
      tilemap_display: tilemap_display,
      event_pump: event_pump,
      breakpoints: HashSet::new(),
      debug_state: DebugState::Running,
      last_frame: std::time::Instant::now(),
      current_tick: 0,
    };
  }

  pub fn tick(&mut self) -> bool {
    match self.debug_state {
      DebugState::Stopped => { 
        thread::sleep(Duration::from_millis(100));
        return true;
      },
      DebugState::Running => {
        if self.breakpoints.contains(&self.cpu.registers.pc) {
          self.debug_state = DebugState::Stopped;
          return true;
        }
      },
      DebugState::Resuming => { self.debug_state = DebugState::Running; },
      DebugState::Quitting => { return false; },
    }

    self.cpu.tick(&mut self.memory, true);
    let has_frame = self.ppu.tick(&mut self.memory, &mut self.main_display);

    self.current_tick = (self.current_tick + 1) % 70224;
/*
    if self.current_tick == 0 {
      println!("70224 ticks");
    }
*/
    if has_frame {
      //println!("frame");
      //println!("{}", self.last_frame.elapsed().as_millis());
      self.last_frame = std::time::Instant::now();

      for event in self.event_pump.poll_iter() {
          match event {
              Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                self.tx.send(ConsoleSignal::Quit).expect("sending quit signal");
                return false;
              },
              _ => {}
          }
      }

      for i in (0x8000..0x97FF).step_by(16) {
        let tile_index = (i - 0x8000) / 16;
        let tile_row = tile_index / 20;
        let tile_col = tile_index % 20;

        for b in (i..(i + 16)).step_by(2) {
          let row = (b - i) / 2;
          let lsb = self.memory[b];
          let msb = self.memory[b + 1];

          for j in 0..8 {
            let bit_mask = 0b10000000 >> j;
            let pix = (((msb & bit_mask) >> (7 - j)) << 1) | ((lsb & bit_mask) >> (7 - j));

            let p: sdl2::rect::Point = sdl2::rect::Point::new((tile_col * 8 + j).into(), (tile_row * 8 + row).into());
            match pix {
              0b00 => {
                self.tilemap_display.c.set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
                self.tilemap_display.c.draw_point(p).unwrap();
              },
              0b01 => {
                self.tilemap_display.c.set_draw_color(Color::RGB(0xAA, 0xAA, 0xAA));
                self.tilemap_display.c.draw_point(p).unwrap();
              },
              0b10 => {
                self.tilemap_display.c.set_draw_color(Color::RGB(0x55, 0x55, 0x55));
                self.tilemap_display.c.draw_point(p).unwrap();
              },
              0b11 => {
                self.tilemap_display.c.set_draw_color(Color::RGB(0x00, 0x00, 0x00));
                self.tilemap_display.c.draw_point(p).unwrap();
              },
              _ => unimplemented!(),
            };
          }
        }
      }

      self.tilemap_display.present();
      self.main_display.present();
    }
    return true;
  }
}

fn main() -> Result<(), String>  {
  let (stx, srx) = mpsc::channel();

  thread::spawn(move || {
    let mut console = Console::new(Path::new("./11.gb"), stx);

    'running: loop {
      if !console.tick() {
        break 'running;
      }
    }
  });
  
  'looping: loop {
    let signal = srx.try_recv();
    match signal {
      Ok(signal) => match signal {
        ConsoleSignal::Quit => { break 'looping }
        ConsoleSignal::BreakpointHit(addr) => {  }
      },
      Err(error) => match error {
        mpsc::TryRecvError::Empty => {},
        mpsc::TryRecvError::Disconnected => { break 'looping; }
      }
    }

    thread::sleep(Duration::from_millis(100));
  }

  return Ok(());
}