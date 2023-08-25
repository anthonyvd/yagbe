use crate::memory::Memory;

use std::vec::Vec;

use sdl2::keyboard::Keycode;

pub enum Button {
	A = 0, 
	B = 1, 
	Start = 2, 
	Select = 3, 
	Up = 4, 
	Right = 5, 
	Down = 6, 
	Left = 7,
}

pub struct Joypad {
	buttons: Vec<bool>,
	last_button_set_select_state: u8,
}

impl Joypad {
	pub fn handle_key_down(&mut self, keycode: Keycode, memory: &mut Memory) {
		let button = Joypad::keycode_to_button(keycode);

		match button {
			Some(b) => { self.button_pressed(b, memory); },
			None => {},
		};
	}

	pub fn handle_key_up(&mut self, keycode: Keycode, memory: &mut Memory) {
		let button = Joypad::keycode_to_button(keycode);

		match button {
			Some(b) => { self.button_released(b, memory); },
			None => {},
		};
	}

	fn keycode_to_button(keycode: Keycode) -> Option<Button> {
		return match keycode {
			Keycode::A => Some(Button::A),
			Keycode::S => Some(Button::B),
			Keycode::Z => Some(Button::Start),
			Keycode::X => Some(Button::Select),

			Keycode::Up => Some(Button::Up),
			Keycode::Right => Some(Button::Right),
			Keycode::Down => Some(Button::Down),
			Keycode::Left => Some(Button::Left),

			_ => None,
		};
	}

	fn make_p1_low_nibble(&self, memory: &Memory) -> u8 {
		let p1 = memory[0xFF00];

		// For some (electronics-related, or weird) reason, the p1 register is 0 == set/pressed 
		// and 1 == not set/pressed, even for the bits that select the button set to query.

		// Directions
		let mut direction_nibble = 0;
		if p1 & 0b10000 == 0 {
			if self.buttons[Button::Down as usize] {
				direction_nibble |= 1;
			}
			direction_nibble = direction_nibble << 1;

			if self.buttons[Button::Up as usize] {
				direction_nibble |= 1;
			}
			direction_nibble = direction_nibble << 1;

			if self.buttons[Button::Left as usize] {
				direction_nibble |= 1;
			}
			direction_nibble = direction_nibble << 1;

			if self.buttons[Button::Right as usize] {
				direction_nibble |= 1;
			}
		}

		// Actions
		let mut action_nibble = 0;
		if p1 & 0b100000 == 0 {
			if self.buttons[Button::Start as usize] {
				action_nibble |= 1;
			}
			action_nibble = action_nibble << 1;

			if self.buttons[Button::Select as usize] {
				action_nibble |= 1;
			}
			action_nibble = action_nibble << 1;

			if self.buttons[Button::B as usize] {
				action_nibble |= 1;
			}
			action_nibble = action_nibble << 1;

			if self.buttons[Button::A as usize] {
				action_nibble |= 1;
			}
		}

		return (!(action_nibble | direction_nibble)) & 0x0F;
	}

	fn button_pressed(&mut self, b: Button, memory: &mut Memory) {
		let p1 = memory[0xFF00];
		let low_nibble = p1 & 0x0F;
		self.buttons[b as usize] = true;
		let new_low_nibble = self.make_p1_low_nibble(memory);
		memory.set_joypad_low_nibble(new_low_nibble);

		if low_nibble != new_low_nibble {
			memory.set(0xFF0F, memory[0xFF0F] | 0b10000);
		}
	}

	fn button_released(&mut self, b: Button, memory: &mut Memory) {
		self.buttons[b as usize] = false;
		let new_low_nibble = self.make_p1_low_nibble(memory);
		memory.set_joypad_low_nibble(new_low_nibble);
	}

	pub fn tick(&mut self, memory: &mut Memory) {
		let p1 = memory[0xFF00];
		let select_state = p1 & 0b110000;
		if select_state != self.last_button_set_select_state {
			let new_low_nibble = self.make_p1_low_nibble(memory);
			memory.set_joypad_low_nibble(new_low_nibble);
			self.last_button_set_select_state = select_state;
		}
	}

	pub fn new() -> Joypad {
		return Joypad {
			buttons: vec![false; 8],
			last_button_set_select_state: 0xFF,
		};
	}

}