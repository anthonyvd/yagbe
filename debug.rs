use std::sync::mpsc;
use std::io;
use std::io::Write;

pub trait Debuggable {
	fn step(&mut self);
	fn request_pc(&mut self) -> u16; // TODO: change to optional
}

pub enum RemoteDebugMessage {
	Step,
	RequestPc,
}

pub enum RemoteDebugMessageResponse {
	RequestPc(u16),
}

pub struct DebuggerRemote {
	sender: mpsc::Sender<RemoteDebugMessage>,
	receiver: mpsc::Receiver<RemoteDebugMessageResponse>,
}

impl DebuggerRemote {
	pub fn new(sender: mpsc::Sender<RemoteDebugMessage>, 
			   receiver: mpsc::Receiver<RemoteDebugMessageResponse>) -> DebuggerRemote{
		return DebuggerRemote {
			sender: sender,
			receiver: receiver,
		};
	}

	pub fn wait_for_response(&mut self) {
		let msg = self.receiver.recv().unwrap();
		match msg {
			RemoteDebugMessageResponse::RequestPc(pc) => {
				println!("PC: {:04X}", pc);
			},
		}
	}

	pub fn update(&mut self) {
		let mut buffer = String::new();
		print!("> "); 
		let _ = std::io::stdout().flush();
    	io::stdin().read_line(&mut buffer).unwrap();

    	if buffer.starts_with("pc") {
    		self.request_pc();
    	}
	}
}

impl Debuggable for DebuggerRemote {
	fn step(&mut self) {
		self.sender.send(RemoteDebugMessage::Step);
	}

	fn request_pc(&mut self) -> u16 {
		self.sender.send(RemoteDebugMessage::RequestPc);

		self.wait_for_response();

		return 0;
	}
}

pub struct DebuggerHost {
	receiver: mpsc::Receiver<RemoteDebugMessage>,
	sender: mpsc::Sender<RemoteDebugMessageResponse>,
}

impl DebuggerHost {
	pub fn new(receiver: mpsc::Receiver<RemoteDebugMessage>,
			   sender: mpsc::Sender<RemoteDebugMessageResponse>) -> DebuggerHost {
		return DebuggerHost {
			receiver: receiver,
			sender: sender,
		};
	}

	pub fn update(&mut self, debuggable: &mut impl Debuggable) {
		let msg = self.receiver.try_recv();
	    match msg {
	      Ok(msg) => match msg {
	        RemoteDebugMessage::Step => { debuggable.step(); },
	        RemoteDebugMessage::RequestPc => {
	        	let pc = debuggable.request_pc();
	        	self.sender.send(RemoteDebugMessageResponse::RequestPc(pc));
	        },
	        _ => { panic!("Unexpected remote debugging message type"); },
	      },
	      Err(error) => match error {
	        mpsc::TryRecvError::Empty => {},
	        mpsc::TryRecvError::Disconnected => { println!("Remote debugger disconnected"); }
	      }
	    }
	}
}