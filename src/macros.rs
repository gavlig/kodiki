// #[cfg(feature = "tracing")]

// #[macro_export]
// macro_rules! profile_scope {
// 	($id:expr) => {
// 		#[cfg(feature = "tracing")]
// 		puffin::profile_scope!($id);
// 	};
// 	($id:expr, $data:expr) => {
// 		#[cfg(feature = "tracing")]
// 		puffin::profile_scope!($id, $data)
// 	};
// }