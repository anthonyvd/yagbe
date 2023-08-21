mod cartridge;
mod cpu;
mod registers;
mod memory_utils;
mod utils;
mod ppu;
mod memory;
mod display;
mod opcodes;
mod console;

use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use std::env;
use std::io;





fn main() -> Result<(), String>  {
  let args: Vec<String> = env::args().collect();

  if args.len() > 1 && args[1] == "--debug" {
    println!("Debugging. Send start (s) to start execution...");

    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer).unwrap();
  }

  let (stx, srx) = mpsc::channel();

  thread::spawn(move || {
    let mut console = console::Console::new(Path::new("./02_failed.gb"), stx);

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
        console::ConsoleSignal::Quit => { break 'looping }
        console::ConsoleSignal::BreakpointHit(_addr) => {  }
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