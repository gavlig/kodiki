use bevy				:: { prelude :: * }; 

pub mod spawn;
pub mod systems;
pub use systems			:: *;
mod utils;

#[derive(Component)]
pub struct Caret {
	pub row: u32,
	pub column: u32,

	pub key_delay_seconds: f32,

	pub key_delay_column_inc: f32,
	pub key_delay_column_dec: f32,
	pub key_delay_row_inc: f32,
	pub key_delay_row_dec: f32,
}

impl Default for Caret {
	fn default() -> Self {
		Self {
			row : 0,
			column : 0,

			key_delay_seconds : 0.03,

			key_delay_column_inc: 0.0,
			key_delay_column_dec: 0.0,
			key_delay_row_inc: 0.0,
			key_delay_row_dec: 0.0,
		}
	}
}

impl Caret {
	pub fn column_inc(&mut self, delta_seconds: f32) {
		self.key_delay_column_inc += delta_seconds;
		if self.key_delay_column_inc >= self.key_delay_seconds {
			self.column += 1;
			self.key_delay_column_inc -= self.key_delay_seconds;
		}
	}

	pub fn column_dec(&mut self, delta_seconds: f32) {
		self.key_delay_column_dec += delta_seconds;
		if self.column > 0 && self.key_delay_column_dec >= self.key_delay_seconds {
			self.column -= 1;
			self.key_delay_column_dec -= self.key_delay_seconds;
		}
	}

	pub fn row_inc(&mut self, delta_seconds: f32) {
		self.key_delay_row_inc += delta_seconds;
		if self.key_delay_row_inc >= self.key_delay_seconds {
			self.row += 1;
			self.key_delay_row_inc -= self.key_delay_seconds;
		}
	}

	pub fn row_dec(&mut self, delta_seconds: f32) {
		self.key_delay_row_dec += delta_seconds;
		if self.row > 0 && self.key_delay_row_dec >= self.key_delay_seconds {
			self.row -= 1;
			self.key_delay_row_dec -= self.key_delay_seconds;
		}
	}
}