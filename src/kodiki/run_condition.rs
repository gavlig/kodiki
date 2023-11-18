use bevy :: prelude :: *;

use super :: { AppMode, AppCameraMode, AppContext };

pub fn main_app_mode(app_mode: Res<State<AppMode>>) -> bool {
	app_mode.0 == AppMode::Main
}

pub fn main_app_mode_no_fly(app_mode: Res<State<AppMode>>, cam_mode: Res<State<AppCameraMode>>) -> bool {
	app_mode.0 == AppMode::Main && cam_mode.0 != AppCameraMode::Fly
}

pub fn text_editor_context(app_mode: Res<State<AppMode>>, app_ctx: Res<State<AppContext>>) -> bool {
	app_mode.0 == AppMode::Main && app_ctx.0 == AppContext::CodeEditor
}

pub fn text_editor_context_no_fly(app_mode: Res<State<AppMode>>, app_ctx: Res<State<AppContext>>, cam_mode: Res<State<AppCameraMode>>) -> bool {
	app_mode.0 > AppMode::AssetsLoaded && app_ctx.0 == AppContext::CodeEditor && cam_mode.0 != AppCameraMode::Fly
}

pub fn terminal_context(app_mode: Res<State<AppMode>>, app_ctx: Res<State<AppContext>>) -> bool {
	app_mode.0 == AppMode::Main && app_ctx.0 == AppContext::Terminal
}
