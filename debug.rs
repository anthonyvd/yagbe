use std::io;
use std::io::Write;
use std::sync::mpsc;

use crate::registers::Registers;

pub trait Debuggable {
    fn step(&mut self);
    fn resume(&mut self);
    fn request_registers(&mut self) -> Option<Registers>;
    fn request_next_instruction(&mut self) -> Option<[u8; 3]>;
    fn set_breakpoint_on_instr(&mut self, instr: u8);
}

pub enum RemoteDebugMessage {
    Step,
    Resume,
    RequestRegisters,
    RequestNextInstr,
    SetBreakpointOnInstr(u8),
}

pub enum RemoteDebugMessageResponse {
    RequestRegisters(Registers),
    RequestNextInstr([u8; 3]),
}

pub struct DebuggerRemote {
    sender: mpsc::Sender<RemoteDebugMessage>,
    receiver: mpsc::Receiver<RemoteDebugMessageResponse>,
}

impl DebuggerRemote {
    pub fn new(
        sender: mpsc::Sender<RemoteDebugMessage>,
        receiver: mpsc::Receiver<RemoteDebugMessageResponse>,
    ) -> DebuggerRemote {
        return DebuggerRemote {
            sender: sender,
            receiver: receiver,
        };
    }

    pub fn wait_for_response(&mut self) {
        let msg = self.receiver.recv().unwrap();
        match msg {
            RemoteDebugMessageResponse::RequestRegisters(registers) => {
                println!("PC: {:04X}", registers.pc);
                println!("SP: {:04X}", registers.sp);
                println!("AF: {:04X}", registers.af);
                println!("BC: {:04X}", registers.bc);
                println!("DE: {:04X}", registers.de);
                println!("HL: {:04X}", registers.hl);
            }
            RemoteDebugMessageResponse::RequestNextInstr(bytes) => {
                println!("{:02X} {:02X} {:02X}", bytes[0], bytes[1], bytes[2]);
            }
        }
    }

    pub fn update(&mut self) {
        let mut buffer = String::new();
        print!("> ");
        let _ = std::io::stdout().flush();
        io::stdin().read_line(&mut buffer).unwrap();

        if buffer.starts_with("reg") {
            self.request_registers();
        } else if buffer.starts_with("c") {
            self.resume();
        } else if buffer.starts_with("l") {
            self.request_next_instruction();
        } else if buffer.starts_with("s") {
            self.step();
        } else if buffer.starts_with("bi") {
            if buffer.len() < 5 {
                println!("Invalid argument");
            } else {
                let mut bytes = [0u8; 1];
                hex::decode_to_slice(&buffer[3..5], &mut bytes as &mut [u8]).unwrap();

                self.set_breakpoint_on_instr(bytes[0]);
            }
        }
    }
}

impl Debuggable for DebuggerRemote {
    fn step(&mut self) {
        self.sender.send(RemoteDebugMessage::Step).unwrap();
    }

    fn resume(&mut self) {
        self.sender.send(RemoteDebugMessage::Resume).unwrap();
    }

    fn request_registers(&mut self) -> Option<Registers> {
        self.sender
            .send(RemoteDebugMessage::RequestRegisters)
            .unwrap();
        self.wait_for_response();

        return None;
    }

    fn request_next_instruction(&mut self) -> Option<[u8; 3]> {
        self.sender
            .send(RemoteDebugMessage::RequestNextInstr)
            .unwrap();
        self.wait_for_response();

        return None;
    }

    fn set_breakpoint_on_instr(&mut self, instr: u8) {
        self.sender
            .send(RemoteDebugMessage::SetBreakpointOnInstr(instr))
            .unwrap();
    }
}

pub struct DebuggerHost {
    receiver: mpsc::Receiver<RemoteDebugMessage>,
    sender: mpsc::Sender<RemoteDebugMessageResponse>,
}

impl DebuggerHost {
    pub fn new(
        receiver: mpsc::Receiver<RemoteDebugMessage>,
        sender: mpsc::Sender<RemoteDebugMessageResponse>,
    ) -> DebuggerHost {
        return DebuggerHost {
            receiver: receiver,
            sender: sender,
        };
    }

    pub fn update(&mut self, debuggable: &mut impl Debuggable) {
        let msg = self.receiver.try_recv();
        match msg {
            Ok(msg) => match msg {
                RemoteDebugMessage::Step => {
                    debuggable.step();
                }
                RemoteDebugMessage::Resume => {
                    debuggable.resume();
                }
                RemoteDebugMessage::RequestRegisters => {
                    let regs = debuggable.request_registers().unwrap();
                    self.sender
                        .send(RemoteDebugMessageResponse::RequestRegisters(regs))
                        .unwrap();
                }
                RemoteDebugMessage::RequestNextInstr => {
                    let bytes = debuggable.request_next_instruction().unwrap();
                    self.sender
                        .send(RemoteDebugMessageResponse::RequestNextInstr(bytes))
                        .unwrap();
                }
                RemoteDebugMessage::SetBreakpointOnInstr(instr) => {
                    debuggable.set_breakpoint_on_instr(instr);
                }
            },
            Err(error) => match error {
                mpsc::TryRecvError::Empty => {}
                mpsc::TryRecvError::Disconnected => {
                    println!("Remote debugger disconnected");
                }
            },
        }
    }
}
