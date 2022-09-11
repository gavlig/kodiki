use bevy				:: { prelude :: * }; 

pub mod spawn;
mod systems;
use systems				:: *;
mod utils;
use utils				:: *;

#[derive(Component, Default)]
pub struct Char3D {
	pub row: u32,
	pub column: u32,
	pub character: char
}