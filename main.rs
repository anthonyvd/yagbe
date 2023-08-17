mod cartridge;
mod cpu;
mod registers;
mod memory_utils;
mod utils;
mod ppu;
mod memory;
mod framebuffer;
mod display;
mod debugger;

use cartridge::Cartridge;
use std::path::Path;
use cpu::Cpu;
use ppu::Ppu;
use memory::Memory;
use framebuffer::Framebuffer;
use display::Display;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::collections::HashSet;

//use std::time::Instant;

use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;

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
  event_pump: sdl2::EventPump,

  breakpoints: HashSet<u16>,
  debug_state: DebugState,

  tx: mpsc::Sender<ConsoleSignal>,
}

impl debugger::Debuggable for Console {
  fn get_registers(&self) -> registers::Registers {
    return self.cpu.registers;
  }

  fn set_breakpoint(&mut self, b: debugger::Breakpoint) {
    self.breakpoints.insert(b.address);
  }

  fn resume(&mut self) {
    self.debug_state = DebugState::Resuming;
  }

  fn quit(&mut self) {
    self.debug_state = DebugState::Quitting;
  }
}

impl Console {
  pub fn new(cart_path: &Path, tx: mpsc::Sender<ConsoleSignal>) -> Console {
    let cart = Cartridge::load(cart_path);
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let main_display = Display::new(&video_subsystem, "Main", 320, 288);
    let event_pump = sdl_context.event_pump().expect("start event_pump");

    return Console { 
      memory: Memory::new(&cart.data), 
      cpu: Cpu::new(), ppu: Ppu::new(), 
      tx: tx, 
      main_display: main_display, 
      event_pump: event_pump,
      breakpoints: HashSet::new(),
      debug_state: DebugState::Running,
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

    self.cpu.tick(&mut self.memory);
    let possible_frame = self.ppu.tick(&mut self.memory);

    if possible_frame.is_some() {
      for event in self.event_pump.poll_iter() {
          match event {
              Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                self.tx.send(ConsoleSignal::Quit).expect("sending quit signal");
                return false;
              },
              _ => {}
          }
      }

      let mut tiles_framebuffer = Framebuffer::new(160, 160);
      
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
              0b00 => tiles_framebuffer.blank_pixels.push(p),
              0b01 => tiles_framebuffer.light_pixels.push(p),
              0b10 => tiles_framebuffer.medium_pixels.push(p),
              0b11 => tiles_framebuffer.dark_pixels.push(p),
              _ => unimplemented!(),
            };
          }
        }
      }

      self.main_display.push_frame(possible_frame.unwrap());
      self.main_display.display_frame();
    }
    return true;
  }
}

fn main() -> Result<(), String>  {
  //let (tx0, rx0) = mpsc::channel();
  //let (tx1, rx1) = mpsc::channel();
  let (stx, srx) = mpsc::channel();

  thread::spawn(move || {
    let mut console = Console::new(Path::new("./tetris.gb"), stx);
    //let mut debugger_backend = debugger::DebuggerBackend::new(tx1, rx0);

    'running: loop {
      //debugger_backend.update(&mut console);
      if !console.tick() {
        break 'running;
      }
    }
  });
  
  //let mut debugger_frontend = debugger::DebuggerFrontend::new(tx0, rx1);
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
    //debugger_frontend.update();
    //debugger_frontend.render();
  }

  return Ok(());
}