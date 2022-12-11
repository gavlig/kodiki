use bevy :: prelude :: *;
use bevy :: input :: keyboard :: *;
use bevy_tweening :: { lens :: *, * };

use bevy_debug_text_overlay :: screen_print;
use bevy_reader_camera :: ReaderCamera;

use super :: SurfacesMapBevy;
use super :: SurfacesMapHelix;
use super :: CursorBevy;
use super :: HelixColorsCache;
use super :: application :: Application;
use super :: spawn;
use super :: update;
use super :: animate;
use super :: input;
use super :: TokioRuntime;

use crate :: game :: DespawnResource;
use crate :: game :: FontAssetHandles;

use crate :: bevy_ab_glyph :: { ABGlyphFont, UsedFonts, TextMeshesCache };

use helix_term  :: config		:: Config;
use helix_term  :: args			:: Args;
use helix_term	:: compositor	:: SurfaceContainer as SurfaceContainerHelix;
use helix_term	:: compositor	:: SurfacePlacement as SurfacePlacementHelix;
use helix_term	:: ui			:: EditorView;
use helix_tui   :: buffer		:: Buffer as SurfaceHelix;
use helix_view  :: graphics 	:: { Rect };

use anyhow      :: { Context, Error, Result };

use std :: path :: PathBuf;
use std :: time :: Duration;

pub fn startup_app(
	world: &mut World,
) {
	let mut surfaces_helix = SurfacesMapHelix::default();
	let 	surfaces_bevy = SurfacesMapBevy::default();
	
	let rect = Rect {
		x : 0,
		y : 0,
		width : 100,
		height : 40,
	};

	let surface_editor = SurfaceHelix::empty(rect);
	surfaces_helix.insert(
		String::from(EditorView::ID),
		SurfaceContainerHelix {
			surface: surface_editor,
			placement: SurfacePlacementHelix::Center,
		}
	);

	world.insert_resource(surfaces_helix);
	world.insert_resource(surfaces_bevy);

	let tokio_runtime : &TokioRuntime = world.resource();

	let app = tokio_runtime.block_on(startup_impl(rect));

	world.insert_non_send_resource(app.unwrap());
}

async fn startup_impl(area: Rect) -> Result<Application, Error> {
	let args = Args::parse_args().context("could not parse arguments").unwrap();

	let config_dir = helix_loader::config_dir();
	if !config_dir.exists() {
		std::fs::create_dir_all(&config_dir).ok();
	}

	helix_loader::initialize_config_file(args.config_file.clone());

	let config = match std::fs::read_to_string(helix_loader::config_file()) {
		Ok(config) => toml::from_str(&config)
			.map(helix_term::keymap::merge_keys)
			.unwrap_or_else(|err| {
				eprintln!("Bad config: {}", err);
				eprintln!("Press <ENTER> to continue with default config");
				use std::io::Read;
				let _ = std::io::stdin().read(&mut []);
				Config::default()
			}),
		Err(err) if err.kind() == std::io::ErrorKind::NotFound => Config::default(),
		Err(err) => { eprintln!("Error while loading config from {}: {}", helix_loader::config_file().display(), err); return Err(anyhow::anyhow!("!!!")); }
	};

	let app = Application::new(args, config, area).context("unable to create new application");

	app
}

pub fn startup_spawn(
	mut surfaces_helix	: ResMut<SurfacesMapHelix>,
	mut surfaces_bevy	: ResMut<SurfacesMapBevy>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles    : Res<FontAssetHandles>,
	mut	cursor          : ResMut<CursorBevy>,
	mut q_reader_camera	: Query<&mut ReaderCamera>,

	(mut text_meshes_cache, mut helix_colors_cache) 
	:
	(ResMut<TextMeshesCache>, ResMut<HelixColorsCache>),

	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets	: ResMut<Assets<StandardMaterial>>,
	mut commands        : Commands,
) {
	let surface_editor_name = String::from(EditorView::ID);
	
	let used_fonts = UsedFonts{
		main			: font_assets.get(&font_handles.main).unwrap(),
		fallback		: font_assets.get(&font_handles.fallback).unwrap()
	};
	
	let container_helix_editor = surfaces_helix.get(&surface_editor_name).unwrap();
	
	spawn::surface(
		&surface_editor_name,
		None,
		&mut surfaces_bevy,
		&container_helix_editor.surface,
		used_fonts.main,
		&mut mesh_assets,
		&mut commands
	);
	
	let surface_bevy_editor = surfaces_bevy.get(&surface_editor_name).unwrap();
	
	spawn::cursor(
		&mut cursor,
		surface_bevy_editor.entity.unwrap(),
		used_fonts.main,
		&mut text_meshes_cache,
		&mut helix_colors_cache,
		&mut material_assets,
		&mut mesh_assets,
		&mut commands
	);
	
	let mut camera		= q_reader_camera.single_mut();
	camera.target		= surface_bevy_editor.entity;
	camera.row			= (surface_bevy_editor.area.height / 2) as u32;
	camera.column		= (surface_bevy_editor.area.width / 2) as u32;
}

pub fn update_main(
	app		: Option<NonSendMut<Application>>,
	time	: Res<Time>,
	
	(
		mut surfaces_helix,
		mut surfaces_bevy,
		mut text_meshes_cache,
		mut helix_colors_cache,
		mut cursor,
			font_assets,
			font_handles,
	)
	:
	(
		ResMut<SurfacesMapHelix>,
		ResMut<SurfacesMapBevy>,
		ResMut<TextMeshesCache>,
		ResMut<HelixColorsCache>,
		ResMut<CursorBevy>,
		Res<Assets<ABGlyphFont>>,
		Res<FontAssetHandles>,
	),
		
	mut	q_transform		: Query<&mut Transform>,
	
	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets	: ResMut<Assets<StandardMaterial>>,
	mut despawn         : ResMut<DespawnResource>,
	mut commands        : Commands,
) {
	if app.is_none() {
		return;
	}

	let used_fonts	= UsedFonts{
		main	: font_assets.get(&font_handles.main).unwrap(),
		fallback: font_assets.get(&font_handles.fallback).unwrap()
	};

	let mut app = app.unwrap();

	let editor_area = app.area;

	// erase previous frame
	for (_name, surface_container) in surfaces_helix.iter_mut() {
		surface_container.surface.reset();
	}

	let old_style = false;

	// first let helix render into surface_helix
	if old_style {
		let surface_helix_editor = surfaces_helix.get_mut(&String::from(EditorView::ID)).unwrap();
		app.render(&mut surface_helix_editor.surface);
	} else {
		app.render_ext(editor_area, &mut surfaces_helix);
	}

	// show currently active helix layers with screen_print!
	screen_print_active_layers(&surfaces_helix);
	screen_print_stats(&surfaces_bevy);

	cleanup_unused_surfaces(&mut surfaces_helix, &mut surfaces_bevy, &mut despawn);
	
	// if surface area changed - respawn its background quad
	respawn_stale_surface_quads(
		&mut surfaces_helix,
		&mut surfaces_bevy,
		used_fonts.main,
		&mut mesh_assets,
		&mut commands
	);

	// create bevy surfaces for every helix surface
	spawn_bevy_surfaces(
		&mut surfaces_helix,
		&mut surfaces_bevy,
		used_fonts.main,
		&mut mesh_assets,
		&mut commands
	);

	// render and animate surfaces
	for (layer_name, container_helix) in surfaces_helix.iter_mut() {
		let surface_bevy = surfaces_bevy.get_mut(layer_name).unwrap();

		update::surface(
			&mut container_helix.surface,
			surface_bevy,

			&app.editor.theme,
			&used_fonts,

			&mut text_meshes_cache,
			&mut helix_colors_cache,

			&mut mesh_assets,
			&mut material_assets,
			&mut commands
		);
	}

	// render and animate cursor
	if app.editor_focused() { 
		let mut surface_bevy_editor = surfaces_bevy.get_mut(&String::from(EditorView::ID)).unwrap();
		let container_helix_editor = surfaces_helix.get(&String::from(EditorView::ID)).unwrap();

		animate::cursor(
			&mut cursor,
			&mut q_transform,
			used_fonts.main,
			&time,
			&mut app
		);

		update::cursor(
			&mut cursor,
			&app.editor.theme,
			&mut helix_colors_cache,
			&mut material_assets,
			&mut commands
		);
	}
}

fn screen_print_active_layers(
	surfaces_helix : &SurfacesMapHelix,
)
{
	let mut surface_names_str = String::default();
	surface_names_str.push_str(format!("{} helix layers:\n", surfaces_helix.len()).as_str());
	for (name, container) in surfaces_helix.iter() {
		surface_names_str.push_str(" - ");
		surface_names_str.push_str(format!("{} len: {} w: {} h: {}", name, container.surface.content.len(), container.surface.area.width, container.surface.area.height).as_str());
		surface_names_str.push('\n');
	}
	screen_print!("\n{}", surface_names_str);
}

fn screen_print_stats(
	surfaces_bevy : &SurfacesMapBevy,
)
{
	let mut stats	= String::default();
	stats.push_str	("stats:\n");
	
	let mut words_cnt = 0;
	for (_name, surface) in surfaces_bevy.iter() {
		for row in surface.word_rows.iter() {
			words_cnt += row.len();
		}
	}
	stats.push_str(format!("words: {}", words_cnt).as_str());
	screen_print!("\n{}", stats);
}

fn cleanup_unused_surfaces(
	surfaces_helix	: &mut SurfacesMapHelix,
	surfaces_bevy	: &mut SurfacesMapBevy,
	despawn			: &mut DespawnResource
) {
    let mut to_remove = Vec::<String>::default();

	// surfaces helix
    for (layer_name, container_helix) in surfaces_helix.iter_mut() {
		// if "dirty" is false it means that during render surface wasn't modified/filled up, meaning it's not longer used
		if container_helix.surface.dirty {
			continue;
		}
				
		to_remove.push(layer_name.clone());
		println!("unused helix surface removed: {}", layer_name);
	}
    for layer in to_remove.iter() {
		surfaces_helix.remove(layer);
	}

	// surfaces bevy
    for (layer_name, surface_bevy) in surfaces_bevy.iter_mut() {
		if surfaces_helix.contains_key(layer_name) {
			continue;
		}
		
		despawn.entities.push(surface_bevy.entity.unwrap());
	
		to_remove.push(layer_name.clone());
		println!("unused bevy surface removed: {}", layer_name);
	}
    for layer in to_remove {
		surfaces_bevy.remove(&layer);
	}
}

// "stale" bevy surface is the one that has a size different from its helix counterpart
fn respawn_stale_surface_quads(
	surfaces_helix	: &mut SurfacesMapHelix,
	surfaces_bevy	: &mut SurfacesMapBevy,
	
	font			: &ABGlyphFont,

	mesh_assets		: &mut Assets<Mesh>,
	commands		: &mut Commands,
) {
    for (layer_name, surface_bevy) in surfaces_bevy.iter_mut() {
		if !surfaces_helix.contains_key(layer_name) {
			continue;
		}

		let container_helix = surfaces_helix.get(layer_name).unwrap();
		if container_helix.surface.area != surface_bevy.area {
			println!("respawning stale bevy surface quad: {} helix.area: {:?} bevy.area: {:?}", layer_name, container_helix.surface.area, surface_bevy.area);
			spawn::surface_quad(
				layer_name,
				surface_bevy,
				&container_helix.surface,
				font,
				mesh_assets,
				commands
			);
		}
	}
}

fn spawn_bevy_surfaces(
	surfaces_helix		: &mut SurfacesMapHelix,
	surfaces_bevy		: &mut SurfacesMapBevy,

	font				: &ABGlyphFont,

	mut mesh_assets		: &mut Assets<Mesh>,
	mut commands		: &mut Commands,
)
{
	for (surface_name, container_helix) in surfaces_helix.iter() {
		if surfaces_bevy.contains_key(surface_name) {
			continue;
		}

		let start_pos = Vec3::new(0.0, 0.0, -0.5);
		let surface_entity = spawn::surface(
			surface_name,
			Some(start_pos),
			surfaces_bevy,
			&container_helix.surface,
			&font,
			
			&mut mesh_assets,
			&mut commands
		);
		
		let target_pos = match container_helix.placement {
			SurfacePlacementHelix::Top => Vec3::new(0.0, -0.0, 0.3),
			SurfacePlacementHelix::Center => Vec3::new(0.7, -1.2, 0.5),
			_ => Vec3::new(0.0, 0.0, 0.3),
		};
		
		let tween_point = animate::TweenPoint {
			pos: target_pos,
			ease_function: EaseFunction::ExponentialOut,
			delay: Duration::from_millis(450),
		};
		
		animate::surface(
			start_pos,
			Vec::from([tween_point]),
			surface_entity,
			commands
		);

		println!("new bevy surface created: {}", surface_name);
	}
}

pub fn input_keyboard(
	mut ev_keyboard : EventReader<KeyboardInput>,
	key				: Res<Input<KeyCode>>,
	tokio_runtime	: Res<TokioRuntime>,
	app				: Option<NonSendMut<Application>>,
) {
	if app.is_none() {
		return;
	}

	input::keyboard(&mut ev_keyboard, &key, &tokio_runtime, &mut app.unwrap());
}

pub fn tokio_events(
	app				: Option<NonSendMut<Application>>,
	tokio_runtime	: Res<TokioRuntime>,
)
{
	if app.is_none() {
		return;
	}
	let mut app = app.unwrap();

	tokio_runtime.block_on(app.handle_tokio_events());
}