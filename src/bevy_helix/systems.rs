use bevy :: prelude :: *;
use bevy :: input :: keyboard :: *;
use bevy_tweening :: { * };

use bevy_debug_text_overlay :: screen_print;
use bevy_reader_camera :: ReaderCamera;

use super :: HelixColorsCache;
use super :: application :: Application;
use super :: surface :: *;
use super :: cursor :: *;

use super :: animate;
use super :: input;
use super :: TokioRuntime;

use crate :: game :: DespawnResource;
use crate :: game :: FontAssetHandles;

use crate :: bevy_ab_glyph :: { ABGlyphFont, UsedFonts, GlyphMeshesCache, TextMeshesCache };

use helix_term  :: config		:: { Config };
use helix_term  :: args			:: { Args };
use helix_term	:: ui			:: { EditorView };
use helix_view  :: graphics 	:: { Rect };

use helix_tui   :: buffer		:: { Buffer as SurfaceHelix, SurfaceFlags, SurfaceAnchor, SurfacePlacement, SurfaceLifetime };

use anyhow      :: { Context, Error, Result };

use std :: time :: Duration;

pub fn startup_app(
	world: &mut World,
)
{
	let mut surfaces_helix	= SurfacesMapHelix::default();
	let 	surfaces_bevy	= SurfacesMapBevy::default();
	
	let rect = Rect {
		x : 0,
		y : 0,
		width : 130,
		height : 60,
	};

	let surface_editor = SurfaceHelix::empty_with_spatial(rect, SurfaceFlags::default());
	surfaces_helix.insert(String::from(EditorView::ID),	surface_editor);

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
		surfaces_helix	: Res<SurfacesMapHelix>,
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
	
	let surface_helix_editor = surfaces_helix.get(&surface_editor_name).unwrap();
	
	let surface_bevy_editor = SurfaceBevy::spawn(
		&surface_editor_name,
		None,
		true, /* scroll_enabled */
		true, /* cache_enabled */
		&surface_helix_editor,
		used_fonts.main,
		&mut mesh_assets,
		&mut commands
	);
	
	CursorBevy::spawn(
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
	camera.row			= 25u32;
	camera.column		= (surface_bevy_editor.area.width / 2) as u32;
	
	surfaces_bevy.insert(surface_editor_name.clone(), surface_bevy_editor);
}

pub fn update_main(
	app		: Option<NonSendMut<Application>>,
	time	: Res<Time>,
	
	(
		mut surfaces_helix,
		mut surfaces_bevy,
		mut glyph_meshes_cache,
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
		ResMut<GlyphMeshesCache>,
		ResMut<TextMeshesCache>,
		ResMut<HelixColorsCache>,
		ResMut<CursorBevy>,
		Res<Assets<ABGlyphFont>>,
		Res<FontAssetHandles>,
	),
		
	mut	q_transform		: Query<&mut Transform>,
	mut q_camera		: Query<&mut ReaderCamera>,
	
	mut mesh_assets		: ResMut<Assets<Mesh>>,
	mut material_assets	: ResMut<Assets<StandardMaterial>>,
	mut despawn         : ResMut<DespawnResource>,
	mut commands        : Commands,
) {
	if app.is_none() {
		return;
	}
	
	let used_fonts	= UsedFonts{
		main		: font_assets.get(&font_handles.main).unwrap(),
		fallback	: font_assets.get(&font_handles.fallback).unwrap()
	};

	let mut app					= app.unwrap();
	
	let mut row_offset			= 0;
	for (view, is_focused) in app.editor.tree.views() {
		if is_focused {
			row_offset			= view.offset.row;
		}
	}
	
	let mut reader_camera		= q_camera.single_mut();
	reader_camera.row_offset	= row_offset as u32;
	reader_camera.row			= reader_camera.visible_rows / 2 - 1; // camera always looks at the center of page 
	
	let mut editor_area			= app.area;
	editor_area.height			= reader_camera.visible_rows as u16;

	// erase previous frame
	for (name, surface_helix) in surfaces_helix.iter_mut() {
		if name == EditorView::ID {
			surface_helix.resize(editor_area);
		}
		surface_helix.reset();
	}
	
	#[derive(PartialEq, Eq)]
	enum RenderMode {
		// Vanilla, BROKEN MOST LIKELY FOREVER
		Kodiki,
		Benchmark
	}
	
	let render_mode = RenderMode::Kodiki;

	// first let helix render into surface_helix
	match render_mode {
		// RenderMode::Vanilla => {
		// 	let surface_helix_editor = surfaces_helix.get_mut(&String::from(EditorView::ID)).unwrap();
		// 	app.render(editor_area, surface_helix_editor);
		// },
		RenderMode::Kodiki => {
			app.render_ext(editor_area, &mut surfaces_helix);
		},
		RenderMode::Benchmark => {
			let surface_bevy_editor = surfaces_bevy.get_mut(&String::from(EditorView::ID)).unwrap();
			if surface_bevy_editor.update {
				let surface_helix_editor = surfaces_helix.get_mut(&String::from(EditorView::ID)).unwrap();
				for cell in surface_helix_editor.content.iter_mut() {
					cell.symbol = String::from("A");
					cell.bg = app.editor.theme.get("ui.background").bg.unwrap();
				}
			}
		}
	}
	
	// show currently active helix layers with screen_print!
	screen_print_active_layers(&surfaces_helix);
	screen_print_stats(&surfaces_bevy);

	if render_mode != RenderMode::Benchmark {
		cleanup_unused_surfaces(&mut surfaces_helix, &mut surfaces_bevy, &mut despawn);
	}
	
	let camera_transform = q_transform.get(reader_camera.entity).unwrap();
	
	// create bevy surfaces for every helix surface
	spawn_bevy_surfaces(
		&mut surfaces_helix,
		&mut surfaces_bevy,
		&reader_camera,
		used_fonts.main,
		&mut mesh_assets,
		&mut material_assets,
		&mut commands
	);
	
	// render surfaces
	for (layer_name, surface_helix) in surfaces_helix.iter_mut() {
		let surface_bevy = surfaces_bevy.get_mut(layer_name).unwrap();

		surface_bevy.update(
			surface_helix,

			row_offset as i32,
			&app.editor.theme,
			&used_fonts,

			&mut glyph_meshes_cache,
			&mut text_meshes_cache,
			&mut helix_colors_cache,

			&mut mesh_assets,
			&mut material_assets,
			&mut commands
		);
	}

	// render and animate cursor
	if app.editor_focused() { 
		cursor.animate(
			&mut q_transform,
			used_fonts.main,
			&time,
			row_offset as u32,
			&mut app
		);

		cursor.update(
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
	for (name, surface) in surfaces_helix.iter() {
		surface_names_str.push_str(" - ");
		surface_names_str.push_str(format!("{} len: {} w: {} h: {}", name, surface.content.len(), surface.area.width, surface.area.height).as_str());
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
		for row in surface.rows.iter() {
			words_cnt += row.words.len();
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
    for (layer_name, surface_helix) in surfaces_helix.iter_mut() {
		// if "dirty" is false it means that during render surface wasn't modified/filled up, meaning it's not longer used
		if surface_helix.dirty {
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

fn spawn_bevy_surfaces(
	surfaces_helix		: &mut SurfacesMapHelix,
	surfaces_bevy		: &mut SurfacesMapBevy,

	reader_camera		: &ReaderCamera,
	font				: &ABGlyphFont,

	mut mesh_assets		: &mut Assets<Mesh>,
	mut material_assets	: &mut Assets<StandardMaterial>,
	mut commands		: &mut Commands,
)
{
	for (surface_name, surface_helix) in surfaces_helix.iter() {
		if surfaces_bevy.contains_key(surface_name) {
			continue;
		}
		
		let row_height		= font.vertical_advance();
		let column_width	= font.horizontal_advance_char('a');

		let start_pos = Vec3::new(0.0, 0.0, -reader_camera.zoom - 1.0);
		
		let surface_bevy = SurfaceBevy::spawn(
			surface_name,
			Some(start_pos),
			false, /* scroll_enabled */
			false, /* cache_enabled */
			&surface_helix,
			&font,
			
			&mut mesh_assets,
			&mut commands
		);
		
		let surface_entity = surface_bevy.entity.unwrap();
		commands.entity(reader_camera.entity).add_child(surface_entity);
		
		if surface_helix.lifetime == SurfaceLifetime::Temporary {
			let target_pos = match surface_helix.placement {
				SurfacePlacement::Top => {
					let x = -column_width * surface_helix.area.width as f32 / 2.0;
					let y = row_height * ((reader_camera.visible_rows / 2) as f32);
					let z = -reader_camera.zoom + 0.01;
					Vec3::new(x, y, z)
				},
				SurfacePlacement::Center => {
					let mut y = row_height * (surface_helix.area.height as f32 / 2.0);
					if surface_helix.anchor == SurfaceAnchor::Bottom {
						y *= -1.0;
					}
					Vec3::new(0.7, y, -reader_camera.zoom + 0.5)
				},
				_ => panic!(),
			};
			
			let tween_point = animate::TweenPoint {
				pos: target_pos,
				ease_function: EaseFunction::ExponentialOut,
				delay: Duration::from_millis(250),
			};
			
			surface_bevy.animate(
				start_pos,
				Vec::from([tween_point]),
				commands
			);
		}
		
		surfaces_bevy.insert(surface_name.clone(), surface_bevy);

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

pub fn update_editor_background_quad(
		surfaces_bevy	: Res<SurfacesMapBevy>,
		q_camera		: Query<(&ReaderCamera, &Transform)>,
	mut	q_transform		: Query<&mut Transform, Without<ReaderCamera>>,
)
{
	let surface_bevy_editor = surfaces_bevy.get(&String::from(EditorView::ID)).unwrap();

	let (reader_camera, camera_transform) = q_camera.single();

	let bg_quad_entity = surface_bevy_editor.background_quad_entity.unwrap();
	let mut bg_quad_transform = q_transform.get_mut(bg_quad_entity).unwrap();

	bg_quad_transform.scale.y = (reader_camera.visible_rows * 2) as f32;
	bg_quad_transform.translation.y = camera_transform.translation.y;
}

pub fn update_permanent_surfaces_position(
		surfaces_bevy	: Res<SurfacesMapBevy>,
		surfaces_helix	: Res<SurfacesMapHelix>,
		font_assets		: Res<Assets<ABGlyphFont>>,
		font_handles	: Res<FontAssetHandles>,
		q_camera		: Query<&ReaderCamera>,
	mut	q_transform		: Query<&mut Transform>,
)
{
	let used_fonts = UsedFonts {
		main			: font_assets.get(&font_handles.main).unwrap(),
		fallback		: font_assets.get(&font_handles.fallback).unwrap()
	};
	
	let row_height		= used_fonts.main.vertical_advance();
	let column_width	= used_fonts.main.horizontal_advance_char('a');
	
	for (surface_name, surface_helix) in surfaces_helix.iter() {
		if surface_name == EditorView::ID {
			continue;
		}
		
		if let Some(surface_bevy) = surfaces_bevy.get(surface_name) {
			let reader_camera = q_camera.single();
			let z = -reader_camera.zoom + 0.05;
			let target_pos = match surface_helix.placement {
				SurfacePlacement::Top => {
					let x = -column_width * (surface_helix.area.width as f32 / 2.0);
					let y = row_height * ((reader_camera.visible_rows as f32 - 1.5)  / 2.0);
					Vec3::new(x, y, z)
				},
				SurfacePlacement::Center => {
					let mut y = row_height * (surface_helix.area.height as f32 / 2.0);
					if surface_helix.anchor == SurfaceAnchor::Bottom {
						y *= -1.0;
					}
					Vec3::new(0.7, y, z)
				},
				SurfacePlacement::Bottom => {
					let mut y = -row_height * ((reader_camera.visible_rows as f32 + 1.5) / 2.0);
					Vec3::new(0.7, y, z)
				},
				_ => panic!(),
			};
			if let Some(surface_entity) = surface_bevy.entity {
				let mut surface_transform = q_transform.get_mut(surface_entity).unwrap();
				surface_transform.translation = target_pos;
			}
		}
	}
}