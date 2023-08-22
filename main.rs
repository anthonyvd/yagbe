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
mod debug;

use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use std::env;
use std::io;

fn main() -> Result<(), String>  {
  let args: Vec<String> = env::args().collect();

  let debug = args.len() > 1 && args[1] == "--debug";

  let (stx, srx) = mpsc::channel();

  // TODO: don't set up the debugger if not debugging
  let (rth_send, rth_recv) = mpsc::channel();
  let (htr_send, htr_recv) = mpsc::channel();
  let mut debugger_remote = debug::DebuggerRemote::new(rth_send, htr_recv);

  thread::spawn(move || {
    let mut console = console::Console::new(Path::new("./tetris.gb"), stx, debug);
    let mut debugger_host = debug::DebuggerHost::new(rth_recv, htr_send);

    'running: loop {
      if debug { debugger_host.update(&mut console); }
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
      },
      Err(error) => match error {
        mpsc::TryRecvError::Empty => {},
        mpsc::TryRecvError::Disconnected => { break 'looping; }
      }
    }

    if debug { debugger_remote.update(); }

    thread::sleep(Duration::from_millis(100));
  }

  return Ok(());
}