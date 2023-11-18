#[cfg(feature = "tracing")]
pub use bevy_puffin :: { * };

#[macro_export]
macro_rules! profile_function {
	() => {
		#[cfg(feature = "tracing")]
		puffin::profile_function!();
	};
	($data:expr) => {
		#[cfg(feature = "tracing")]
		puffin::profile_function!($data);
	};
}

#[macro_export]
macro_rules! profile_scope {
	($id:expr) => {
		#[cfg(feature = "tracing")]
		puffin::profile_scope!($id);
	};
	($id:expr, $data:expr) => {
		#[cfg(feature = "tracing")]
		puffin::profile_scope!($id, $data)
	};
}