use crate::cartridge::Cartridge;
use crate::cpu::Cpu;
use crate::debug::Debuggable;
use crate::display::Display;
use crate::memory::Memory;
use crate::ppu::Ppu;
use crate::registers::Registers;
use crate::joypad::Joypad;

use std::collections::HashSet;
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;

#[derive(PartialEq)]
enum DebugState {
    Running,
    Stopped,
    Stepping,
}

pub enum ConsoleSignal {
    Quit,
}

pub struct Console {
    memory: Memory,
    cpu: Cpu,
    ppu: Ppu,
    joypad: Joypad,

    main_display: Display,
    tilemap_display: Display,

    event_pump: sdl2::EventPump,

    instr_breakpoints: HashSet<u8>,
    debug_state: DebugState,

    tx: mpsc::Sender<ConsoleSignal>,

    current_timer_tick: u64,
    current_div_tick: u64,
}

impl Console {
    pub fn new(cart_path: &Path, tx: mpsc::Sender<ConsoleSignal>, debugged: bool) -> Console {
        let cart = Cartridge::load(cart_path);
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let tilemap_display = Display::new(&video_subsystem, "Tilemap", 256, 384, 2.0, 2.0);
        let main_display = Display::new(&video_subsystem, "Main", 320, 288, 2.0, 2.0);
        let event_pump = sdl_context.event_pump().expect("start event_pump");
        let mut mem = Memory::new(&cart.data);
        mem.initialize(0xFF0F, 0xE1); // Interrupt request
        mem.initialize(0xFFFF, 0x00); // Interrupt mask
        mem.initialize(0xFF00, 0xFF); // joypad
        mem.initialize(0xFF40, 0x91); // LCDC
        mem.initialize(0xFF47, 0xFC); // BGP

        return Console {
            memory: mem,
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            joypad: Joypad::new(),
            tx: tx,
            main_display: main_display,
            tilemap_display: tilemap_display,
            event_pump: event_pump,
            instr_breakpoints: HashSet::new(),
            debug_state: if debugged {
                DebugState::Stopped
            } else {
                DebugState::Running
            },
            current_timer_tick: 0,
            current_div_tick: 0,
        };
    }

    pub fn check_for_input(&mut self) -> bool {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    self.tx
                        .send(ConsoleSignal::Quit)
                        .expect("sending quit signal");
                    return false;
                },
                Event::KeyDown {
                  keycode: Some(code), ..
                } => {
                  self.joypad.handle_key_down(code, &mut self.memory);
                },
                Event::KeyUp {
                  keycode: Some(code), ..
                } => {
                  self.joypad.handle_key_up(code, &mut self.memory);
                },
                _ => {}
            }
        }

        return true;
    }

    fn update_timer_registers(&mut self) {
        // TODO: simultaneous TMA writes and TIMA overflows are well defined but not well implemented here, see pandocs
        // TODO: any write to DIV resets it to 0
        let tac = self.memory[0xFF07];
        let timer_enabled = tac & 0b100 != 0;
        let clock_select = tac & 0b11;

        let divider = match clock_select {
            0b00 => 1024,
            0b01 => 16,
            0b10 => 64,
            0b11 => 256,
            _ => {
                panic!("Nope");
            }
        };

        self.current_div_tick = (self.current_div_tick + 1) % 256;
        if self.current_div_tick == 0 {
            // TODO: This is reset when executing a stop instruction
            self.memory.set(0xFF04, self.memory[0xFF04].wrapping_add(1));
        }

        if timer_enabled {
            self.current_timer_tick = (self.current_timer_tick + 1) % divider;
            if self.current_timer_tick == 0 {
                if self.memory[0xFF05] == 0xFF {
                    self.memory.set(0xFF05, self.memory[0xFF06]);
                    self.memory.set(0xFF0F, self.memory[0xFF0F] | 0b100);
                } else {
                    self.memory.set(0xFF05, self.memory[0xFF05].wrapping_add(1));
                }
            }
        }
    }

    // Returning true here means "keep console alive". False will kill it.
    pub fn tick(&mut self) -> bool {
        if self.debug_state == DebugState::Stopped {
            if !self.check_for_input() {
                return false;
            } // TODO: might not be correct when input is supported, but still need to poll at least for the "quit" event.
            thread::sleep(Duration::from_millis(100));
            return true;
        } else if self.debug_state == DebugState::Running
            && self
                .instr_breakpoints
                .contains(&self.memory[self.cpu.registers.pc])
        {
            self.debug_state = DebugState::Stopped;
            return true;
        }

        self.update_timer_registers();
        self.memory.tick();
        self.joypad.tick(&mut self.memory);
        let instr_run = self.cpu.tick(&mut self.memory, true);
        let has_frame = self.ppu.tick(&mut self.memory, &mut self.main_display);

        if self.debug_state == DebugState::Stepping && instr_run {
            self.debug_state = DebugState::Stopped;
        }

        if has_frame {
            if !self.check_for_input() {
                return false;
            } // TODO: does joypad poll more often? Probably.

            for i in (0x8000..0x97FF).step_by(16) {
                let tile_index = (i - 0x8000) / 16;
                let tile_row = tile_index / 16;
                let tile_col = tile_index % 16;

                for b in (i..(i + 16)).step_by(2) {
                    let row = (b - i) / 2;
                    let lsb = self.memory[b];
                    let msb = self.memory[b + 1];

                    for j in 0..8 {
                        let bit_mask = 0b10000000 >> j;
                        let pix =
                            (((msb & bit_mask) >> (7 - j)) << 1) | ((lsb & bit_mask) >> (7 - j));

                        let p: sdl2::rect::Point = sdl2::rect::Point::new(
                            (tile_col * 8 + j).into(),
                            (tile_row * 8 + row).into(),
                        );
                        match pix {
                            0b00 => {
                                self.tilemap_display
                                    .c
                                    .set_draw_color(Color::RGB(0xFF, 0xFF, 0xFF));
                                self.tilemap_display.c.draw_point(p).unwrap();
                            }
                            0b01 => {
                                self.tilemap_display
                                    .c
                                    .set_draw_color(Color::RGB(0xAA, 0xAA, 0xAA));
                                self.tilemap_display.c.draw_point(p).unwrap();
                            }
                            0b10 => {
                                self.tilemap_display
                                    .c
                                    .set_draw_color(Color::RGB(0x55, 0x55, 0x55));
                                self.tilemap_display.c.draw_point(p).unwrap();
                            }
                            0b11 => {
                                self.tilemap_display
                                    .c
                                    .set_draw_color(Color::RGB(0x00, 0x00, 0x00));
                                self.tilemap_display.c.draw_point(p).unwrap();
                            }
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

impl Debuggable for Console {
    fn step(&mut self) {
        match self.debug_state {
            DebugState::Stopped => {
                self.debug_state = DebugState::Stepping;
            }
            _ => {}
        }
    }

    fn resume(&mut self) {
        self.debug_state = DebugState::Running;
    }

    fn request_registers(&mut self) -> Option<Registers> {
        return Some(self.cpu.registers);
    }

    fn request_next_instruction(&mut self) -> Option<[u8; 3]> {
        let mut ret: [u8; 3] = [0; 3];
        let pc = self.cpu.registers.pc;
        for off in 0u16..3u16 {
            ret[off as usize] = self.memory[pc + off];
        }
        return Some(ret);
    }

    fn set_breakpoint_on_instr(&mut self, instr: u8) {
        self.instr_breakpoints.insert(instr);
    }
}
