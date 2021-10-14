
use crate::registers::Registers;


use std::io;
use std::sync::mpsc;
use std::thread;

use lazy_static::lazy_static;
use regex::Regex;

use tui::Terminal;
use tui::backend::TermionBackend;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use termion::event::Key;
use termion::screen::AlternateScreen;

use tui::widgets::{Widget, Block, Borders, Paragraph};
use tui::layout::{Layout, Constraint, Direction};
use tui::text::{Span, Spans};

pub struct Breakpoint {
  pub address: u16,
}

pub enum DebuggerMessage {
  RequestRegisters,
  RespondRegisters(Registers),
  SetBreakpoint(Breakpoint),
  BreakpointHit(Breakpoint),
  Resume,
  Quit,
}

pub trait Debuggable {
  fn get_registers(&self) -> Registers;
  fn set_breakpoint(&mut self, b: Breakpoint);
  fn resume(&mut self);
  fn quit(&mut self);
}

pub struct DebuggerFrontend {
  terminal: Terminal<TermionBackend<AlternateScreen<RawTerminal<std::io::Stdout>>>>,
  tx: mpsc::Sender<DebuggerMessage>,
  rx: mpsc::Receiver<DebuggerMessage>,

  stdin_rx: mpsc::Receiver<termion::event::Key>,
  command_buffer: String,

  registers: Registers,
}

impl DebuggerFrontend {
	pub fn new(tx: mpsc::Sender<DebuggerMessage>, rx: mpsc::Receiver<DebuggerMessage>) -> DebuggerFrontend {
    let stdout = io::stdout().into_raw_mode().expect("can get stdout");
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);

    let (stdin_tx, stdin_rx) = mpsc::channel();
    thread::spawn(move || {
      loop {
        let s = io::stdin();
        for e in s.keys() {
          if let Ok(key) = e {
            stdin_tx.send(key).unwrap();
          }
        }
      }
    });

    return DebuggerFrontend { 
      terminal: Terminal::new(backend).expect("can create a terminal"), 
      tx: tx, rx: rx,
      stdin_rx: stdin_rx,
      command_buffer: String::from(""), 
      registers: Registers::new()
    };
	}

  pub fn update(&mut self) {
    let mut iter = self.rx.try_iter();
    let mut next = iter.next();
    let mut processed_messages = false;
    while next.is_some() {
      processed_messages = true;
      match next {
        Some(message) => {
          match message {
            DebuggerMessage::RespondRegisters(registers) => {
              self.registers = registers;
            },
            _ => { unimplemented!(); }
          }
        },
        _ => { unimplemented!(); }
      }
      next = iter.next();
    }

    if !processed_messages {
      // If there were no messages processed, let's ask for an update
      self.tx.send(DebuggerMessage::RequestRegisters).expect("sending a RequestRegisters");
    }

    lazy_static! {
      static ref BP_SET_RE: Regex = Regex::new("^b ([a-fA-F0-9]{4})$").unwrap();
      static ref RESUME_RE: Regex = Regex::new("^r$").unwrap();
      static ref QUIT_RE: Regex = Regex::new("^q$").unwrap();
    }

    'looping: loop {
      let data = self.stdin_rx.try_recv();
      if let Ok(key) = data {
          match key {
            Key::Char('\n') => {
              for cap in BP_SET_RE.captures_iter(self.command_buffer.trim()) {
                let addr = u16::from_str_radix(&cap[1], 16).unwrap();
                self.tx.send(DebuggerMessage::SetBreakpoint(Breakpoint { address: addr })).unwrap();
              }

              if RESUME_RE.is_match(self.command_buffer.trim()) {
                self.tx.send(DebuggerMessage::Resume).unwrap();
              }

              if QUIT_RE.is_match(self.command_buffer.trim()) {
                self.tx.send(DebuggerMessage::Quit).unwrap();
              }

              self.command_buffer = String::from("");
            },
            Key::Backspace => { self.command_buffer.pop(); },
            Key::Esc => { self.command_buffer = String::from(""); }
            Key::Char(c) => { self.command_buffer.push(c); },
            _ => {},
          }

      } else {
        break 'looping;
      }
    }
  }

  pub fn render(&mut self) {
    let registers = self.registers;
    let command_buffer = self.command_buffer.clone();
    self.terminal.draw(|f| {
      let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(10),Constraint::Percentage(80),Constraint::Percentage(10),].as_ref())
        .split(f.size());

      let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60),Constraint::Percentage(40),].as_ref())
        .split(main_chunks[1]);

      let registers_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(25),Constraint::Percentage(25),Constraint::Percentage(25),Constraint::Percentage(25),].as_ref())
        .split(main_chunks[0]);

      let block = Block::default()
        .title("Registers")
        .borders(Borders::ALL);
      f.render_widget(block, main_chunks[0]);
      let block = Block::default()
        .title("Program")
        .borders(Borders::ALL);
      f.render_widget(block, middle_chunks[0]);
      let block = Block::default()
        .title("Watches")
        .borders(Borders::ALL);
      f.render_widget(block, middle_chunks[1]);
      let block = Block::default()
        .title("Commands")
        .borders(Borders::ALL);
      f.render_widget(block, main_chunks[2]);

      let text = vec![
        Spans::from(Span::raw("AF")),
        Spans::from(Span::raw("BC")),
        Spans::from(Span::raw("DE")),
      ];
      let p = Paragraph::new(text);
      f.render_widget(p, registers_chunks[0]);

      let text = vec![
        Spans::from(Span::raw(format!("{:04X}", registers.af))),
        Spans::from(Span::raw(format!("{:04X}", registers.bc))),
        Spans::from(Span::raw(format!("{:04X}", registers.de))),
      ];
      let p = Paragraph::new(text);
      f.render_widget(p, registers_chunks[1]);

      let text = vec![
        Spans::from(Span::raw("HL")),
        Spans::from(Span::raw("SP")),
        Spans::from(Span::raw("PC")),
      ];
      let p = Paragraph::new(text);
      f.render_widget(p, registers_chunks[2]);

      let text = vec![
        Spans::from(Span::raw(format!("{:04X}", registers.hl))),
        Spans::from(Span::raw(format!("{:04X}", registers.sp))),
        Spans::from(Span::raw(format!("{:04X}", registers.pc))),
      ];
      let p = Paragraph::new(text);
      f.render_widget(p, registers_chunks[3]);

      let input_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(main_chunks[2]);
      let text = vec![ Spans::from(Span::raw(command_buffer)), ];
      let p = Paragraph::new(text);
      f.render_widget(p, input_chunk[0]);

    }).expect("can draw terminal");
  }
}

pub struct DebuggerBackend {
  tx: mpsc::Sender<DebuggerMessage>,
  rx: mpsc::Receiver<DebuggerMessage>,
}

impl DebuggerBackend {
  pub fn new(tx: mpsc::Sender<DebuggerMessage>, rx: mpsc::Receiver<DebuggerMessage>) -> DebuggerBackend {
    return DebuggerBackend { tx: tx, rx: rx };
  }

  pub fn update(&mut self, target: &mut impl Debuggable) {
    let mut iter = self.rx.try_iter();
    let mut next = iter.next();
    while next.is_some() {
      match next {
        Some(message) => {
          match message {
            DebuggerMessage::RequestRegisters => {
              self.tx.send(DebuggerMessage::RespondRegisters(target.get_registers())).expect("sending registers");
            },
            DebuggerMessage::SetBreakpoint(b) => {
              target.set_breakpoint(b);
            },
            DebuggerMessage::Resume => {
              target.resume();
            },
            DebuggerMessage::Resume => {
              target.quit();
            },
            _ => { unimplemented!(); }
          }
        },
        _ => { unimplemented!(); }
      }
      next = iter.next();
    }
  }
}