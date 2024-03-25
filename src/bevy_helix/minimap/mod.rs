use bevy :: {
	prelude :: *,
	render :: {
		render_resource :: { Extent3d, TextureDimension, TextureFormat, TextureUsages },
		render_asset :: RenderAssetUsages,
	},
	tasks :: { AsyncComputeTaskPool, Task },
};
use bevy_rapier3d :: prelude :: *;

use bevy_tweening :: { *, lens :: TransformScaleLens };

#[cfg(feature = "tracing")]
pub use bevy_puffin :: *;

use helix_lsp :: lsp :: { SymbolKind, Location };

use helix_core :: {
	LineEnding,
	Selection,
	RopeSlice,
	syntax :: HighlightEvent,
};
use helix_view :: {
	Document,
	Theme,
	graphics :: Color as HelixColor,
};

use crate :: {
	z_order,
	kodiki_ui :: { * , color :: * , tween_lens :: * , raypick :: RaypickHover },
	bevy_helix :: { SyncDataDoc, SyncDataDiagnostics, MatchRange, Highlights, HighlightKind, SearchKind, VersionType },
	bevy_ab_glyph :: {
		{ ABGlyphFonts, ABGlyphFont, GlyphMeshesCache, TextMeshesCache },
		glyph_image_generator :: generate_string_image,
		glyph_mesh_generator :: generate_string_mesh_wcache,
	}
};

use super :: {
	helix_app	:: HelixApp,
	tween_lens	:: *,
	surface		:: *,
	utils		:: *,
};

use std :: time :: Duration;

pub mod systems;
mod systems_util;
use systems_util :: *;

const MINIMAP_FONT_HEIGHT		: f32 = 4.0;
const MINIMAP_WIDTH				: f32 = 0.7;
const MINIMAP_HEIGHT			: f32 = 0.0; // gets recalculated dynamically in on_document_changed according to the amount of lines in file
const MINIMAP_PADDING			: f32 = 0.07;
const POINTER_SIZE				: f32 = 0.1;
const VIEWPORT_WIDTH			: f32 = MINIMAP_WIDTH;
const VIEWPORT_HEIGHT			: f32 = 1.0; // gets scaled according to the amount of visible rows
const VIEWPORT_ALPHA			: f32 = 0.05;
const VIEWPORT_ALPHA_HOVERED	: f32 = 0.07;
const VIEWPORT_ALPHA_PRESSED	: f32 = 0.12;
const VIEWPORT_ALPHA_LIGHT		: f32 = 0.35;
const VIEWPORT_ALPHA_HOVERED_LIGHT : f32 = 0.42;
const VIEWPORT_ALPHA_PRESSED_LIGHT : f32 = 0.50;
const SYMBOL_BOOKMARK_WIDTH		: f32 = MINIMAP_PADDING / 2.0;
const SYMBOL_BOOKMARK_HEIGHT	: f32 = 0.01;
const SYMBOL_BOOKMARK_SIZE		: Vec2 = Vec2::new(SYMBOL_BOOKMARK_WIDTH, SYMBOL_BOOKMARK_HEIGHT);
const DIFF_HUNK_WIDTH			: f32 = MINIMAP_PADDING / 3.0;
const DIAGNOSTIC_WIDTH			: f32 = MINIMAP_WIDTH;

#[derive(Component)]
pub struct MinimapPointer {
	pub size			: f32,
}

#[derive(Component)]
pub struct MinimapViewport {
	pub size			: Vec2,
	pub alpha			: f32,
	pub alpha_hovered	: f32,
	pub alpha_pressed	: f32,
	pub hovered			: bool,
	pub pressed			: bool,
	pub current_row		: usize,
	pub last_hovered_row: Option<usize>,
	pub minimap_entity	: Entity
}

impl Default for MinimapViewport {
	fn default() -> Self {
		Self {
			size			: Vec2::new(VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
			alpha			: VIEWPORT_ALPHA,
			alpha_hovered	: VIEWPORT_ALPHA_HOVERED,
			alpha_pressed	: VIEWPORT_ALPHA_PRESSED,
			hovered			: false,
			pressed			: false,
			current_row		: 0,
			last_hovered_row: None,
			minimap_entity	: Entity::from_raw(0)
		}
	}
}

impl MinimapViewport {
	pub fn dragging_active(&self) -> bool {
		self.last_hovered_row.is_some()
	}
}

#[derive(Component)]
pub struct Bookmark {
	pub name			: String,
	pub kind			: SymbolKind,
	pub location		: Location,
	pub color			: Color,

	pub minimap_entity	: Entity,
	pub visual_entity	: Entity,
}

#[derive(Component)]
pub struct BookmarkRevealed;

#[derive(Component)]
pub struct BookmarkHint {
	pub owner			: Entity,
}

#[derive(Clone, PartialEq, Debug)]
pub struct ColoredString {
	pub text			: String,
	pub color			: Color
}

impl ColoredString {
	pub fn new() -> Self {
		Self { text : String::new(), color : Color::PINK }
	}

	pub fn clear(&mut self) {
		self.text		= String::new();
		self.color		= Color::PINK;
	}
}

#[derive(Default, Clone, Debug)]
pub struct ColoredStringRow {
	pub words			: Vec<ColoredString>,
	pub length			: usize
}

impl ColoredStringRow {
	pub fn clear(&mut self) {
		self.words.clear();
		self.length		= 0;
	}
}

#[derive(Component)]
pub struct MinimapHoveredLine {
	pub num				: usize,
	pub text			: ColoredStringRow
}

#[derive(Component)]
pub struct ClickPoint;

#[derive(Component)]
pub struct ClickPointVisual;

#[derive(Component)]
pub struct MinimapScrollAnimation {
	row			: usize,
	row_changed : bool,
}

impl MinimapScrollAnimation {
	pub fn set_row(&mut self, new_row: usize) {
		self.row = new_row;
		self.row_changed = true;
	}

	pub fn row_read_only(&self) -> usize {
		self.row
	}

	pub fn row_read_to_apply(&mut self) -> Option<usize> {
		if self.row_changed {
			self.row_changed = false;
			Some(self.row)
		} else {
			None
		}
	}
}

impl Default for MinimapScrollAnimation {
	fn default() -> Self {
		Self {
			row			: 0,
			row_changed	: false,
		}
	}
}

#[derive(Component)]
pub struct MinimapScaledMode {
	pub active		: bool,
	pub scale_from	: f32,
	pub scale_to	: f32,
	pub transition_timer : Timer,
}

impl Default for MinimapScaledMode {
	fn default() -> Self {
		Self {
			active		: false,
			scale_from	: 1.0,
			scale_to	: 1.0,
			transition_timer : Timer::from_seconds(0.1, TimerMode::Once),
		}
	}
}

#[derive(Component)]
pub struct MinimapRenderTask(Task<(usize, Vec<ColoredStringRow>, Vec<Image>)>);

#[derive(Component)]
pub struct Minimap {
	pub document_cache	: Option<SyncDataDoc>,
	pub bookmarks_version: Option<usize>,

	pub diagnostics_highlights		: Highlights<SyncDataDiagnostics>,
	pub selection_highlights		: Highlights<SyncDataDoc>,
	pub search_highlights			: Highlights<VersionType>,
	pub selection_search_highlights : Highlights<VersionType>,

	pub font_height		: f32,
	pub size			: Vec2,
	pub image_size		: Vec2,
	pub padding			: f32,

	pub row_height		: f32,
	pub rows_total		: usize,

	pub colored_rows	: Vec<ColoredStringRow>,

	// self
	pub entity				: Entity,
	pub image_chunk_entities: Vec<Entity>,
	// children
	pub pointer_entity		: Entity,
	pub viewport_entity		: Entity,
	pub bookmark_entities	: Vec<Entity>,
	pub diff_entities		: Vec<Entity>,

	//
	pub render_task_spawned	: bool,
}

impl Default for Minimap {
	fn default() -> Self {
		Self {
			document_cache	: None,
			bookmarks_version : None,

			diagnostics_highlights		: Highlights::<_>::default(),
			selection_highlights		: Highlights::<_>::default(),
			search_highlights			: Highlights::<_>::default(),
			selection_search_highlights	: Highlights::<_>::default(),

			font_height		: MINIMAP_FONT_HEIGHT,
			size			: Vec2::new(MINIMAP_WIDTH, MINIMAP_HEIGHT),
			image_size		: Vec2::new(256.0, 512.0),
			padding			: MINIMAP_PADDING,

			row_height		: 0.0,
			rows_total		: 0,

			colored_rows	: Vec::new(),

			entity				: Entity::from_raw(0),
			image_chunk_entities: Vec::new(),

			pointer_entity		: Entity::from_raw(0),
			viewport_entity		: Entity::from_raw(0),
			bookmark_entities	: Vec::new(),
			diff_entities		: Vec::new(),

			render_task_spawned	: false,
		}
	}
}

impl Minimap {
	pub fn spawn(
		mesh_assets		: &mut Assets<Mesh>,
		material_assets	: &mut Assets<StandardMaterial>,
		commands		: &mut Commands
	) -> Entity {
		let minimap_entity = commands.spawn((
			TransformBundle::default(),
			VisibilityBundle::default(),
		)).id();

		let pointer_entity = Self::spawn_pointer(mesh_assets, material_assets, commands);
		let viewport_entity = Self::spawn_viewport(minimap_entity, mesh_assets, material_assets, commands);

		let minimap = Minimap {
			entity: minimap_entity,
			pointer_entity,
			viewport_entity,
			..default()
		};

		commands.entity(minimap_entity)
			.insert((
				minimap,
				MinimapScaledMode::default()
			))
			.push_children(&[pointer_entity, viewport_entity])
		;

		minimap_entity
	}

	pub fn spawn_pointer(
		mesh_assets		: &mut Assets<Mesh>,
		material_assets	: &mut Assets<StandardMaterial>,
		commands		: &mut Commands
	) -> Entity {
		let pointer_mesh_handle = mesh_assets.add(RegularPolygon::new(POINTER_SIZE / 2.0, 3));

		let pointer_material_handle = material_assets.add(StandardMaterial {
			base_color: Color::WHITE,
			unlit : true,
			..default()
		});

		commands.spawn((
			PbrBundle {
				mesh		: pointer_mesh_handle,
				material	: pointer_material_handle,
				transform	: Transform {
					translation : Vec3::Z * z_order::minimap::pointer(),
					rotation : Quat::from_rotation_z(-90.0f32.to_radians()),
					..default()
				},
				..default()
			},
			MinimapPointer { size: POINTER_SIZE }
		)).id()
	}

	pub fn spawn_viewport(
		minimap_entity	: Entity,
		mesh_assets		: &mut Assets<Mesh>,
		material_assets	: &mut Assets<StandardMaterial>,
		commands		: &mut Commands
	) -> Entity {
		let viewport_size = MinimapViewport::default().size;
		let viewport_thickness = z_order::thickness();
		let viewport_mesh_handle = mesh_assets.add(Rectangle::from_size(viewport_size));

		let viewport_material_handle = material_assets.add(StandardMaterial {
			base_color: Color::Rgba { red: 1.0, green: 1.0, blue: 1.0, alpha: VIEWPORT_ALPHA },
			unlit : true,
			alpha_mode : AlphaMode::Blend,
			..default()
		});

		commands.spawn((
			PbrBundle {
				mesh		: viewport_mesh_handle,
				material	: viewport_material_handle,
				transform	: Transform::from_translation(Vec3::Z * z_order::minimap::viewport()),
				..default()
			},
			MinimapViewport { minimap_entity, ..default() },
			RigidBody::Fixed,
			Collider::cuboid(viewport_size.x / 2.0, viewport_size.y / 2.0, viewport_thickness / 2.0),
			RaypickHover::default()
		)).id()
	}

	pub fn apply_render_task_results(
		&mut self,
		rows_total			: usize,
		colored_rows		: Vec<ColoredStringRow>,
		mut minimap_chunks	: Vec<Image>,
		mesh_assets			: &mut Assets<Mesh>,
		image_assets		: &mut Assets<Image>,
		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands
	) {
		let image_width			= Minimap::default().image_size.x as u32;
		let image_chunk_height	= Minimap::default().image_size.y as u32;

		let image_height	= self.font_height * rows_total as f32;

		let chunks_cnt		= (image_height / image_chunk_height as f32).ceil() as usize;

		// Despawn

		// previous image chunk entities
		for chunk_entity in self.image_chunk_entities.iter() {
			commands.entity(*chunk_entity).despawn_recursive();
		}

		self.image_chunk_entities.clear();
		self.image_chunk_entities.reserve(chunks_cnt as usize);

		// Spawn

		let quad_width		= MINIMAP_WIDTH;
		let quad_height		= quad_width * (image_height / image_width as f32);
		let quad_size		= Vec2::new(quad_width, quad_height);

		let quad_chunk_height = quad_width * (image_chunk_height as f32 / image_width as f32);
		let quad_chunk_size	= Vec2::new(quad_width, quad_chunk_height);

		// rendered image chunks
		for chunk_id in (0 .. chunks_cnt).rev() {
			let quad_mesh_handle = mesh_assets.add(Rectangle::from_size(quad_chunk_size));

			let image_handle = image_assets.add(minimap_chunks.pop().unwrap());
			let text_image_material_handle = material_assets.add(StandardMaterial {
				base_color_texture	: Some(image_handle.clone()),
				unlit				: true,
				alpha_mode			: AlphaMode::Blend,
				..default()
			});

			let y_offset = -(chunk_id as f32 * quad_chunk_height) - (quad_chunk_height / 2.0) + (quad_height / 2.0);

			let image_chunk_entity = commands.spawn((
				PbrBundle {
					mesh		: quad_mesh_handle,
					material	: text_image_material_handle,
					transform	: Transform {
						translation : Vec3::Y * y_offset,
						..default()
					},
					..default()
				},
			)).id();

			self.image_chunk_entities.push(image_chunk_entity);
		}

		commands.entity(self.entity).push_children(self.image_chunk_entities.as_slice());

		// add collision to minimap for raypick
		commands.entity(self.entity).insert((
			RigidBody	:: Fixed,
			Collider	:: cuboid(quad_size.x / 2., quad_size.y / 2., z_order::thickness() / 2.),
			RaypickHover:: default()
		));

		let minimap_row_height = self.font_height * (quad_height / image_height);

		self.size		= quad_size;
		self.image_size	= Vec2::new(image_width as f32, image_height);
		self.colored_rows = colored_rows;
		self.row_height	= minimap_row_height;
		self.rows_total	= rows_total;

		self.render_task_spawned = false;
	}

	pub fn spawn_render_task(
		&mut self,
		fonts				: &ABGlyphFonts,
		doc					: &Document,
		theme				: Theme,
		dark_theme			: bool,
		commands			: &mut Commands
	) {
		let text				= doc.text().clone(); // looked like a crime at first but we really have no interest in concurrent access to file text especially if it can be altered while the background task is running
		let rows_total			= doc.text().len_lines();
		let tab_size			= doc.tab_width();

		let range = {
			let start			= 0;
			let end				= text.line_to_byte(text.len_lines() as usize);

			start .. end
		};

		let highlights = match doc.syntax() {
			Some(syntax) => syntax.highlight_iter(text.slice(..), Some(range), None).collect::<Vec<_>>(),
			None =>
                [Ok(HighlightEvent::Source {
                    start: range.start,
                    end: range.end,
                })]
                .into(),
		};

		let font				= fonts.main.f.clone(); // cloning Arc so we're good
		let font_height			= self.font_height;

		let image_chunk_height	= Minimap::default().image_size.y as u32;
		let rows_per_chunk		= (image_chunk_height as f32 / font_height) as usize;

		let thread_pool			= AsyncComputeTaskPool::get();

		let task = thread_pool.spawn(async move {
			let image_width			= Minimap::default().image_size.x as u32;
			let image_chunk_height	= Minimap::default().image_size.y as u32;

			let image_height	= font_height * rows_total as f32;

			let chunks_cnt		= (image_height / image_chunk_height as f32).ceil() as usize;

			let image_chunk_size = Extent3d {
				width	: image_width,
				height	: image_chunk_height,
				..default()
			};

			let mut minimap_chunks = Vec::new();
			minimap_chunks.reserve(chunks_cnt);

			// prepare image chunks to be filled in render_document
			for _chunk_id in 0 .. chunks_cnt {
				let mut image = Image::new_fill(
					image_chunk_size,
					TextureDimension::D2,
					&[0, 0, 0, 0],
					TextureFormat::Bgra8Unorm,
					RenderAssetUsages::all()
				);

				image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING;

				minimap_chunks.push(image);
			}

			let mut spans		= Vec::new();
			let mut column		= 0usize;
			let mut row			= 0usize;

			let text_style		= theme.get("ui.text");

			let mut word		= ColoredString::new();
			let mut colored_string = ColoredStringRow::default();
			let mut row_length	= 0 as usize;

			let mut colored_rows = Vec::<ColoredStringRow>::new();

			let append_colored_string = |string: &String, color: Color, word: &mut ColoredString, colored_line: &mut ColoredStringRow| {
				// color changed so we start filling up a new word
				if word.color != color && word.text.len() > 0 {
					colored_line.words.push(word.clone());
					word.clear();
				}

				if word.text.len() == 0 {
					word.color = color;
				}

				word.text.push_str(string.as_str());
			};

			for event in highlights {
			match event.unwrap() {
				HighlightEvent::HighlightStart(span) => {
					spans.push(span);
				}
				HighlightEvent::HighlightEnd => {
					spans.pop();
				}
				HighlightEvent::Source { start, end } => {
					let text = text.byte_slice(start..end); // .unwrap_or_else(|| " ".into());
					let style = spans
						.iter()
						.fold(text_style, |acc, span| acc.patch(theme.highlight(span.0)));

					use helix_core::graphemes::{grapheme_width, RopeGraphemes};

					for grapheme in RopeGraphemes::new(text) {
						if LineEnding::from_rope_slice(&grapheme).is_some() {
							column = 0;
							row += 1;

							colored_string.words.push(word.clone());
							colored_string.length = row_length;

							colored_rows.push(colored_string.clone());

							colored_string.clear();
							word.clear();
							row_length = 0;

							continue;
						}

						let color = color_from_helix(style.fg.unwrap_or(HelixColor::White));

						let length =
						if grapheme == "\t" {
							let length = tab_size - (column % tab_size);
							word.text.push_str(" ".repeat(length).as_str());

							length
						} else if grapheme == " " || grapheme == "\u{00A0}" {
							word.text.push_str(" ");

							1
						} else {
							let chunk_id	= row / rows_per_chunk;
							let row_in_chunk= row - (chunk_id * rows_per_chunk);
							let image_chunk	= &mut minimap_chunks[chunk_id];

							let string		= String::from(grapheme);

							// in light themes it's harder to notice contents of minimap with default alpha
							let alpha_multiplier = if dark_theme { 1.5 } else { 2.5 };

							generate_string_image(&string, &font, font_height, color, row_in_chunk, column, alpha_multiplier, image_chunk);
							append_colored_string(&string, color, &mut word, &mut colored_string);

							grapheme_width(grapheme.as_str().unwrap())
						};

						column += length;
						row_length += length;
					}
				}
			}}

			// last word
			if word.text.len() > 0 {
				colored_string.words.push(word);
			}

			(rows_total, colored_rows, minimap_chunks)
		});

		commands.spawn(MinimapRenderTask(task));

		self.render_task_spawned = true;
	}

	pub fn update_viewport(
		&self,
		dark_theme			: bool,
		viewport			: &mut MinimapViewport,
		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands
	) {
		let color = if dark_theme {
			viewport.alpha = VIEWPORT_ALPHA;
			viewport.alpha_hovered = VIEWPORT_ALPHA_HOVERED;
			viewport.alpha_pressed = VIEWPORT_ALPHA_PRESSED;

			Color::rgba(1.0, 1.0, 1.0, viewport.alpha)
		} else {
			viewport.alpha = VIEWPORT_ALPHA_LIGHT;
			viewport.alpha_hovered = VIEWPORT_ALPHA_HOVERED_LIGHT;
			viewport.alpha_pressed = VIEWPORT_ALPHA_PRESSED_LIGHT;

			Color::rgba(0.0, 0.0, 0.0, viewport.alpha)
		};

		// not using cache here on purpose to have each click point blinking independently
		let viewport_material_handle = material_assets.add(
			StandardMaterial {
				base_color : color,
				alpha_mode : AlphaMode::Blend,
				unlit : true,
				..default()
			}
		);

		commands.entity(self.viewport_entity).insert(viewport_material_handle);
	}

	pub fn update_bookmarks(
		&mut self,
		doc					: &Document,
		theme				: &Theme,
		mesh_assets			: &mut Assets<Mesh>,
		color_materials_cache : &mut ColorMaterialsCache,
		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands,
	) {
		// despawn old bookmarks
		for bookmark_entity in self.bookmark_entities.iter() {
			commands.entity(*bookmark_entity).despawn_recursive();
		}
		self.bookmark_entities.clear();

		let mut last_end_line = 0;

		for symbol in doc.symbols() {
			// nested constants are cluttering minimap bookmarks so skip them
			if symbol.kind == SymbolKind::CONSTANT && symbol.location.range.start.line < last_end_line {
				continue;
			}

			last_end_line = symbol.location.range.end.line;

			// ignoring these types to avoid clutter
			match symbol.kind {
				SymbolKind::FIELD
			|	SymbolKind::OBJECT
			|	SymbolKind::TYPE_PARAMETER
			|	SymbolKind::ENUM_MEMBER	=> continue,
				_ => (),
			}

			let base_color = match symbol.kind {
				SymbolKind::FUNCTION	=> color_from_helix(theme.get("function").fg.unwrap_or(HelixColor::Cyan)),
				SymbolKind::CLASS
			|	SymbolKind::INTERFACE
			|	SymbolKind::STRUCT		=> Color::GOLD,
				SymbolKind::CONSTANT
			|	SymbolKind::ENUM		=> Color::TURQUOISE,
				_						=> Color::ALICE_BLUE,
			};

			let bookmark_mesh_handle = mesh_assets.add(Rectangle::from_size(SYMBOL_BOOKMARK_SIZE));

			let bookmark_material_handle = get_color_material_handle(
				base_color,
				color_materials_cache,
				material_assets
			);

			let row				= symbol.location.range.start.line as usize + 1;
			let bookmark_x		= - self.size.x / 2.0 - MINIMAP_PADDING / 2.0;
			let bookmark_y		= -(row as f32 * self.row_height) + self.size.y / 2.0;

			let symbol_visual_entity = commands.spawn(
				PbrBundle {
					mesh		: bookmark_mesh_handle,
					material	: bookmark_material_handle,
					..default()
				}
			).id();

			let symbol_collision_entity = commands.spawn((
				TransformBundle	:: default(),
				RigidBody		:: Fixed,
				Collider		:: cuboid(SYMBOL_BOOKMARK_SIZE.x, SYMBOL_BOOKMARK_SIZE.y * 2.0, z_order::thickness() / 2.0), // collision is intentionally bigger
			)).id();

			let symbol_entity	= commands.spawn((
				TransformBundle { local: Transform::from_translation(Vec3::new(bookmark_x, bookmark_y, z_order::minimap::bookmark())), ..default() },
				VisibilityBundle:: default(),
				RaypickHover	:: default(),
				Bookmark {
					name		: symbol.name.clone(),
					kind		: symbol.kind,
					location	: symbol.location.clone(),
					color		: base_color,
					minimap_entity : self.entity,
					visual_entity : symbol_visual_entity
				},
			)).id();

			commands.entity(symbol_entity).push_children(&[symbol_visual_entity, symbol_collision_entity]);

			commands.entity(self.entity).add_child(symbol_entity);

			self.bookmark_entities.push(symbol_entity);
		}
	}

	pub fn update_diff_gutter(
		&mut self,
		doc					: &Document,
		theme				: &Theme,
		mesh_assets			: &mut Assets<Mesh>,
		color_materials_cache : &mut ColorMaterialsCache,
		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands,
	) {
		profile_function!();

		// despawn old diff hunks
		for diff_entity in self.diff_entities.iter() {
			commands.entity(*diff_entity).despawn_recursive();
		}
		self.diff_entities.clear();

		let added		= theme.get("diff.plus");
		let deleted		= theme.get("diff.minus");
		let modified	= theme.get("diff.delta");
		if let Some(diff_handle) = doc.diff_handle() {
			let diff = diff_handle.load();

			for hunk_i in 0 .. diff.len() {
				let hunk = diff.nth_hunk(hunk_i);

				let hunk_rows = hunk.after.end - hunk.after.start;
				let hunk_size = Vec2::new(DIFF_HUNK_WIDTH, self.row_height * hunk_rows as f32);
				let hunk_mesh_handle = mesh_assets.add(Rectangle::from_size(hunk_size));

				let base_color = color_from_helix(
					if hunk.is_pure_insertion() {
						added
					} else if hunk.is_pure_removal() {
						deleted
					} else {
						modified
					}
					.fg.unwrap_or(HelixColor::Cyan)
				);

				let hunk_material_handle = get_color_material_handle(
					base_color,
					color_materials_cache,
					material_assets
				);

				let row				= ((hunk.after.start + hunk.after.end) / 2) as usize + 1;
				let hunk_x			= self.size.x / 2.0 + MINIMAP_PADDING / 2.0;
				let hunk_y			= -(row as f32 * self.row_height) + self.size.y / 2.0;

				let hunk_entity	= commands.spawn(
					PbrBundle {
						mesh		: hunk_mesh_handle,
						material	: hunk_material_handle,
						transform	: Transform::from_translation(Vec3::new(hunk_x, hunk_y, z_order::minimap::diff_gutter())),
						..default()
					}
				).id();

				commands.entity(self.entity).add_child(hunk_entity);

				self.diff_entities.push(hunk_entity);
			}
		};
	}

	fn highlight_entities_mut(&mut self, kind: HighlightKind) -> &mut Vec<Entity> {
		match kind {
			HighlightKind::Diagnostic	=>
				&mut self.diagnostics_highlights.entities,
			HighlightKind::Search		=>
				&mut self.search_highlights.entities,
			HighlightKind::Selection	=>
				&mut self.selection_highlights.entities,
			HighlightKind::SelectionSearch =>
				&mut self.selection_search_highlights.entities,
			HighlightKind::Cursor		=>
				panic!("Cursor highlights are not implemented for Minimap!")
		}
	}

	fn highlight_z_offset(&self, kind: HighlightKind) -> f32 {
		match kind {
			HighlightKind::Diagnostic	=>
				z_order::surface::highlight_diagnostic(),
			HighlightKind::Search		=>
				z_order::surface::highlight_search(),
			HighlightKind::Selection	=>
				z_order::surface::highlight_selection(),
			HighlightKind::SelectionSearch =>
				z_order::surface::highlight_search(),
			HighlightKind::Cursor		=>
				panic!("Cursor highlights are not implemented for Minimap!")
		}
	}

	fn spawn_highlight_precise(
		&mut self,
		start_char			: usize,
		end_char			: usize,
		kind				: HighlightKind,
		material_handle		: &Handle<StandardMaterial>,
		doc_slice			: &RopeSlice,
		tab_len				: usize,
		fonts				: &ABGlyphFonts,
		mesh_assets			: &mut Assets<Mesh>,
		commands			: &mut Commands,
	) {
		let column_width	= self.column_width(fonts.main);

		let (highlight_start_line, highlight_end_line) = (doc_slice.char_to_line(start_char), doc_slice.char_to_line(end_char));

		let highlight_width_max = self.size.x;
		let max_line_len		= (self.size.x / column_width) as usize;

		// rendering each selected line separately and accurately
		for line in highlight_start_line ..= highlight_end_line {
			let line_chars		= doc_slice.line(line).len_chars();

			let line_start_char	= doc_slice.line_to_char(line);
			let line_end_char	= line_start_char + line_chars.saturating_sub(1);

			let inner_start_char = line_start_char.max(start_char);
			let inner_end_char	= line_end_char.min(end_char);

			// this can happen when line is too long and doesnt fit in viewport yet we moved cursor beyond viewport bounds
			if inner_start_char >= inner_end_char {
				continue
			}

			// calculate offset before the first character in the line (inner_start_char is a position of first non-whitespace character in the line)
			let mut offset_len = 0;
			for char_index in line_start_char .. inner_start_char {
				if '\t' == doc_slice.char(char_index) {
					offset_len += tab_len - (offset_len % tab_len);
				} else {
					offset_len += 1;
				}
			}

			// since there are tabs that are wider than 1 character we need to calculate width by iterating over every char in the line (probably there will be more than just tab later)
			let mut highlight_len = 0;
			for char_index in inner_start_char .. inner_end_char {
				if '\t' == doc_slice.char(char_index) {
					highlight_len += tab_len - ((offset_len + highlight_len) % tab_len);
				} else {
					highlight_len += 1;
				}
			}

			// cutting off right side of line with max_line_len
			let total_len = offset_len + highlight_len;
			if total_len > max_line_len {
				highlight_len = highlight_len.saturating_sub(total_len - max_line_len);
			}

			if highlight_len == 0 {
				continue;
			}

			let highlight_width = highlight_len as f32 * column_width;
			let highlight_size	= Vec2::new(highlight_width, self.row_height);
			let highlight_mesh_handle = mesh_assets.add(Rectangle::from_size(highlight_size));

			let highlight_x		= (offset_len as f32 * column_width) + (highlight_width / 2.0) - (highlight_width_max / 2.0);
			let highlight_y		= -((line + 1) as f32 * self.row_height) + self.row_height / 2.0 + self.size.y / 2.0;

			let highlight_entity = commands.spawn(
				PbrBundle {
					mesh		: highlight_mesh_handle,
					material	: material_handle.clone_weak(),
					transform	: Transform::from_translation(Vec3::new(highlight_x, highlight_y, self.highlight_z_offset(kind))),
					..default()
				}
			).id();

			commands.entity(self.entity).add_child(highlight_entity);

			self.highlight_entities_mut(kind).push(highlight_entity);
		}
	}

	fn spawn_highlight_whole_line(
		&mut self,
		highlight_row_start	: u32,
		highlight_row_end	: u32,
		highlight_kind		: HighlightKind,
		highlight_material_handle : &Handle<StandardMaterial>,
		mesh_assets			: &mut Assets<Mesh>,
		commands			: &mut Commands,
	) {
		let rows_cnt		= (highlight_row_end - highlight_row_start + 1) as f32;
		let highlight_size	= Vec2::new(MINIMAP_WIDTH, rows_cnt * self.row_height);
		let highlight_mesh_handle = mesh_assets.add(Rectangle::from_size(highlight_size));

		let highlight_x		= 0.0;

		let vertical_offset	= (highlight_row_start as f32 + (rows_cnt / 2.0)) * self.row_height;
		let highlight_y		= -vertical_offset + self.size.y / 2.0;

		let highlight_entity = commands.spawn(
			PbrBundle {
				mesh		: highlight_mesh_handle,
				material	: highlight_material_handle.clone_weak(),
				transform	: Transform::from_translation(Vec3::new(highlight_x, highlight_y, self.highlight_z_offset(highlight_kind) - z_order::half_thickness())),
				..default()
			}
		).id();

		commands.entity(self.entity).add_child(highlight_entity);

		self.highlight_entities_mut(highlight_kind).push(highlight_entity);
	}

	pub fn despawn_highlights(
		&mut self,
		kind				: HighlightKind,
		commands			: &mut Commands,
	) {
		let entities = self.highlight_entities_mut(kind);

		for entity in entities.iter() {
			commands.entity(*entity).despawn_recursive();
		}

		entities.clear();
	}

	pub fn update_diagnostics_highlights(
		&mut self,
		doc					: &Document,
		theme				: &Theme,
		mesh_assets			: &mut Assets<Mesh>,
		color_materials_cache : &mut ColorMaterialsCache,
		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands,
	) {
		let doc_slice = doc.text().slice(..);

		use helix_core::diagnostic::Severity;
		let get_scope_of = |scope| {
			theme
			.find_scope_index(scope)
			// get one of the themes below as fallback values
			.or_else(|| theme.find_scope_index("diagnostic"))
			.or_else(|| theme.find_scope_index("ui.cursor"))
			.or_else(|| theme.find_scope_index("ui.selection"))
			.expect(
				"at least one of the following scopes must be defined in the theme: `diagnostic`, `ui.cursor`, or `ui.selection`",
			)
		};

		// query the theme color defined in the config
		let hint_scope		= get_scope_of("hint");
		let info_scope		= get_scope_of("info");
		let warning_scope	= get_scope_of("warning");
		let error_scope		= get_scope_of("error");

		type DiagnosticsVec = Vec<std::ops::Range<usize>>;

		let mut info_vec	: DiagnosticsVec = Vec::new();
		let mut hint_vec	= Vec::new();
		let mut warning_vec	= Vec::new();
		let mut error_vec	= Vec::new();

		for diagnostic in doc.diagnostics() {
			// Separate diagnostics into different Vecs by severity.
			let vec = match diagnostic.severity {
				Some(Severity::Info)	=> &mut info_vec,
				Some(Severity::Hint)	=> &mut hint_vec,
				Some(Severity::Warning)	=> &mut warning_vec,
				Some(Severity::Error)	=> &mut error_vec,
				_ => continue,
			};

			let range_start_line = doc_slice.char_to_line(diagnostic.range.start);
			let range_end_line = doc_slice.char_to_line(diagnostic.range.end);

			// If any diagnostic overlaps ranges with the prior diagnostic,
			// merge the two together. Otherwise push a new span.
			match vec.last_mut() {
				Some(range) if range_start_line <= range.end => {
					// This branch merges overlapping diagnostics, assuming that the current
					// diagnostic starts on range.start or later. If this assertion fails,
					// we will discard some part of `diagnostic`. This implies that
					// `doc.diagnostics()` is not sorted by `diagnostic.range`.
					debug_assert!(range.start <= range_start_line);
					range.end = range_end_line.max(range.end)
				}
				_ => {
					vec.push(range_start_line..range_end_line)
				},
			}
		}

		self.despawn_highlights(HighlightKind::Diagnostic, commands);

		// lambda to spawn each kind of diagnostics overlay over minimap
		let mut spawn_diagnostics_vec = |diagnostics_vec: &DiagnosticsVec, theme_scope: usize| {
			let style = theme.highlight(theme_scope);

			let mut base_color = color_from_helix(style.fg.unwrap_or(HelixColor::Cyan));
			base_color.set_a(0.1);

			let diag_material_handle = get_color_material_walpha_handle(
				base_color,
				AlphaMode::Blend,
				color_materials_cache,
				material_assets
			);

			for diag in diagnostics_vec.iter() {
				self.spawn_highlight_whole_line(
					diag.start as u32,
					diag.end as u32,
					HighlightKind::Diagnostic,
					&diag_material_handle,
					mesh_assets,
					commands
				);
			}
		};

		spawn_diagnostics_vec(&info_vec, info_scope);
		spawn_diagnostics_vec(&hint_vec, hint_scope);
		spawn_diagnostics_vec(&warning_vec, warning_scope);
		spawn_diagnostics_vec(&error_vec, error_scope);
	}

	pub fn update_selection_highlights(
		&mut self,
		doc					: &Document,
		selection			: &Selection,
		theme				: &Theme,
		fonts				: &ABGlyphFonts,
		mesh_assets			: &mut Assets<Mesh>,
		color_materials_cache : &mut ColorMaterialsCache,
		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands,
	) {
        let selection_scope = theme
            .find_scope_index("ui.selection.primary")
			.or_else(|| theme.find_scope_index("ui.selection"))
            .expect("could not find `ui.selection` scope in the theme!");

		let style = theme.highlight(selection_scope);

		let mut base_color = color_from_helix(style.bg.unwrap_or(HelixColor::Cyan));
		base_color.set_a(0.9);

		let partial_selection_material_handle = get_color_material_walpha_handle(
			base_color,
			AlphaMode::Blend,
			color_materials_cache,
			material_assets
		);

		base_color.set_a(0.3);
		let whole_selection_material_handle = get_color_material_walpha_handle(
			base_color,
			AlphaMode::Blend,
			color_materials_cache,
			material_assets
		);

		let highlight_kind = HighlightKind::Selection;

		self.despawn_highlights(highlight_kind, commands);

		let tab_len = doc.tab_width();

		for range in selection.iter() {
			if range.head == range.anchor {
				continue;
			}

			let doc_slice = doc.text().slice(..);

			let (selection_start_char, selection_end_char) = if range.head > range.anchor {
				(range.anchor, range.head)
			} else {
				(range.head, range.anchor)
			};

			self.spawn_highlight_precise(
				selection_start_char,
				selection_end_char,
				highlight_kind,
				&partial_selection_material_handle,
				&doc_slice,
				tab_len,
				fonts,
				mesh_assets,
				commands
			);

			let (selection_start_line, selection_end_line) = (doc_slice.char_to_line(selection_start_char), doc_slice.char_to_line(selection_end_char));

			self.spawn_highlight_whole_line(
				selection_start_line as u32,
				selection_end_line as u32,
				highlight_kind,
				&whole_selection_material_handle,
				mesh_assets,
				commands
			);
		}
	}

	pub fn get_search_highlights_mut(&mut self, kind: SearchKind) -> &mut Highlights<usize> {
		match kind {
			SearchKind::Common => &mut self.search_highlights,
			SearchKind::Selection => &mut self.selection_search_highlights
		}
	}

	pub fn update_search_highlights(
		&mut self,
		matches				: &Vec<MatchRange>,
		search_kind			: SearchKind,
		doc					: &Document,
		theme				: &Theme,
		fonts				: &ABGlyphFonts,
		mesh_assets			: &mut Assets<Mesh>,
		color_materials_cache : &mut ColorMaterialsCache,
		material_assets		: &mut Assets<StandardMaterial>,
		commands			: &mut Commands,
	) {
		let highlight_kind	= HighlightKind::from(search_kind);

		self.despawn_highlights(highlight_kind, commands);

		let search_scope = theme
            .find_scope_index("hint")
			.or_else(|| theme.find_scope_index("info"))
			.or_else(|| theme.find_scope_index("diagnostic"))
            .expect("could not find `hint` scope in the theme!");

		let style = theme.highlight(search_scope);

		let mut base_color = color_from_helix(style.fg.unwrap_or(HelixColor::Cyan));
		base_color.set_a(0.7);

		let partial_search_material_handle = get_color_material_walpha_handle(
			base_color,
			AlphaMode::Blend,
			color_materials_cache,
			material_assets
		);

		base_color.set_a(0.03);
		let whole_search_material_handle = get_color_material_walpha_handle(
			base_color,
			AlphaMode::Blend,
			color_materials_cache,
			material_assets
		);

		let doc_slice = doc.text().slice(..);

		let tab_len = doc.tab_width();

		for range in matches {
			self.spawn_highlight_precise(
				range.start,
				range.end,
				highlight_kind,
				&partial_search_material_handle,
				&doc_slice,
				tab_len,
				fonts,
				mesh_assets,
				commands
			);

			let (search_start_line, search_end_line) = (doc_slice.char_to_line(range.start), doc_slice.char_to_line(range.end));

			self.spawn_highlight_whole_line(
				search_start_line as u32,
				search_end_line as u32,
				highlight_kind,
				&whole_search_material_handle,
				mesh_assets,
				commands
			);
		}
	}

	pub fn spawn_preview_area(
		&self,
		hovered_line_num			: usize,
		minimap_scale				: Vec3,
		q_minimap_hovered_line		: Query<(Entity, &MinimapHoveredLine)>,
		font						: &ABGlyphFont,

		mesh_assets					: &mut Assets<Mesh>,
		material_assets				: &mut Assets<StandardMaterial>,
		glyph_meshes_cache			: &mut GlyphMeshesCache,
		text_meshes_cache			: &mut TextMeshesCache,
		color_materials_cache		: &mut ColorMaterialsCache,

		app							: NonSendMut<HelixApp>,
		commands					: &mut Commands,
	) {
		let hovered_line = &self.colored_rows[hovered_line_num];

		// spawn new hovered line entity if none exists and attach it to minimap
		let hovered_line_entity = match q_minimap_hovered_line.get_single() {
			Ok((entity, hovered_line_old)) => {
				// make sure the lines we're about to spawn are different from existing ones
				if hovered_line_old.num == hovered_line_num {
					return
				}

				// despawn outdated lines
				commands.entity(entity).despawn_descendants();
				commands.entity(entity).insert(MinimapHoveredLine {
					num		: hovered_line_num,
					text	: hovered_line.clone(),
				});

				entity
			},
			_ => {
				let entity = commands.spawn((
					VisibilityBundle::default(),
					MinimapHoveredLine {
						num		: hovered_line_num,
						text	: hovered_line.clone(),
					},
				)).id();
				commands.entity(self.entity).add_child(entity);

				entity
			}
		};

		let hovered_line_x	= - self.size.x / 2.0 - self.padding;
		let hovered_line_y	= -(hovered_line_num as f32 * self.row_height) + self.size.y / 2.0;

		commands.entity(hovered_line_entity).insert(
			TransformBundle {
				local: Transform {
					translation : Vec3::new(hovered_line_x, hovered_line_y, z_order::minimap::hovered_line()),
					scale : 1.0 / minimap_scale,
					..default()
				},
				..default()
			}
		);

		let row_height		= font.vertical_advance();
		let sub_line_cnt	= 5 as usize;
		let sub_line_scale	= 0.8;
		let max_line_len	= 55 as usize;

		let min_line_num 	= hovered_line_num.saturating_sub(sub_line_cnt / 2);
		let max_line_num	= (min_line_num + sub_line_cnt).min(self.colored_rows.len() - 1);
		let mut sub_line_num = min_line_num;

		let line_num_str_len = self.colored_rows.len().to_string().len();

		while sub_line_num < max_line_num {
			let offset_y = (sub_line_num as isize - hovered_line_num as isize) as f32 * -row_height * sub_line_scale;

			let sub_line_entity =
			commands.spawn((
				TransformBundle {
					local : Transform::from_translation(
						Vec3::new(0.0, offset_y, z_order::minimap::hovered_line_text())
					),
					..default()
				},
				VisibilityBundle::default()
			)).id();

			commands.entity(hovered_line_entity).add_child(sub_line_entity);

			let sub_line = &self.colored_rows[sub_line_num];

			let sub_line_num_str = format!("{:>len$} ", sub_line_num, len = line_num_str_len);
			let line_num_color = color_from_helix(app.editor.theme.get("ui.linenr").fg.unwrap_or(HelixColor::Cyan));

			// prepend line with its row number
			let mut sub_line_modified = ColoredStringRow {
				length	: sub_line_num_str.len(),
				words	: Vec::from([ColoredString { text : sub_line_num_str, color : line_num_color }])
			};
			sub_line_modified.length += sub_line.length;
			sub_line_modified.words.append(&mut sub_line.words.clone());

			let word_entities = Minimap::spawn_colored_string_row(
				&sub_line_modified,
				sub_line_scale,
				max_line_len,
				font,
				mesh_assets,
				glyph_meshes_cache,
				text_meshes_cache,
				color_materials_cache,
				material_assets,
				commands
			);

			commands.entity(sub_line_entity).push_children(word_entities.as_slice());

			sub_line_num += 1;
		}

		// background quad

		let background_style = app.editor.theme.get("ui.popup");

		let color			= color_from_helix(background_style.bg.unwrap_or_else(|| { HelixColor::Black }));
		let background_quad_material_handle = get_color_material_handle(
			color,
			color_materials_cache,
			material_assets
		);

		let column_width	= font.horizontal_advance_mono();
		let quad_width		= column_width * (max_line_len) as f32;
		let quad_height		= row_height * (sub_line_cnt) as f32;// * 1.1;
		let quad_position	= Vec3::new(
			-quad_width / 2.0,
			row_height / 2.0, // glyph mesh anchor is not in the middle of it, but at the bottom + descent so we need to account for that here
			0.0
		);

		let quad_size = Vec2::new(quad_width, quad_height);
		let quad_mesh_entity = spawn::background_quad(
			quad_position * sub_line_scale,
			quad_size * sub_line_scale,
			false, /* with_collision */
			Some(&background_quad_material_handle),
			mesh_assets,
			commands
		);

		commands.entity(hovered_line_entity).add_child(quad_mesh_entity);
	}

	pub fn spawn_colored_string_row(
		colored_line_in	: &ColoredStringRow,
		line_scale		: f32,
		max_line_len	: usize,
		font			: &ABGlyphFont,

		mut mesh_assets				: &mut Assets<Mesh>,
		mut glyph_meshes_cache		: &mut GlyphMeshesCache,
		mut text_meshes_cache		: &mut TextMeshesCache,
		mut color_materials_cache	: &mut ColorMaterialsCache,
		mut material_assets			: &mut Assets<StandardMaterial>,

		commands					: &mut Commands
	) -> Vec<Entity> {
		// cutoff all symbols that dont fit into max_line_len
		let colored_line = if colored_line_in.length > max_line_len {
			let mut colored_line	= ColoredStringRow::default();
			for word in colored_line_in.words.iter() {
				let word_len		= word.text.len();
				let new_line_len 	= colored_line.length + word_len;

				let (new_word, overflow) = if new_line_len < max_line_len {
					(word.clone(), false)
				} else {
					let shortened_len = word_len.saturating_sub((new_line_len - max_line_len) + 1);
					if shortened_len == 0 {
						break;
					}

					let shortened_text = &word.text[..shortened_len];
					(ColoredString { text : shortened_text.into(), color : word.color }, true)
				};

				colored_line.length += new_word.text.len();
				colored_line.words.push(new_word);

				if overflow {
					break;
				}
			}

			colored_line
		} else {
			colored_line_in.clone()
		};

		let mut word_entities = Vec::new();
		let mut offset_x	= 0.0;

		let column_width	= font.horizontal_advance_mono();
		let line_width		= max_line_len as f32 * column_width;

		for colored_string in colored_line.words.iter() {
			let word_x		= line_width - offset_x; // inverted offsets because we start in furthest negative x

			let text		= &colored_string.text;
			let text_width	= column_width * text.len() as f32;

			let color		= colored_string.color;

			offset_x		+= text_width;

			let (word_mesh_handle, material_handle) = (
				generate_string_mesh_wcache(&text, font, &mut mesh_assets, &mut glyph_meshes_cache, &mut text_meshes_cache),
				get_color_material_handle(
					color,
					&mut color_materials_cache,
					&mut material_assets
				)
			);

			let mesh_entity = spawn::mesh_material_entity(
				&word_mesh_handle,
				&material_handle,
				commands
			);

			commands.entity(mesh_entity).insert(
				Transform {
					translation : Vec3::new(-word_x, 0.0, 0.0) * line_scale,
					scale : Vec3::ONE * line_scale,
					..default()
				}
			);

			word_entities.push(mesh_entity);
		}

		word_entities
	}

	pub fn spawn_click_point(
		&self,
		cursor_position_minimap : Vec3,
		dark_theme				: bool,

		mesh_assets				: &mut Assets<Mesh>,
		material_assets			: &mut Assets<StandardMaterial>,
		commands				: &mut Commands
	) {
		let mut base_color = if dark_theme { Color::ANTIQUE_WHITE } else { Color::DARK_GRAY };
		base_color.set_a(0.0);

		// not using cache here on purpose to have each click point blinking independently
		let color_handle = material_assets.add(
			StandardMaterial {
				base_color,
				alpha_mode : AlphaMode::Blend,
				unlit : true,
				..default()
			}
		);

		let click_point_entity = commands.spawn((
			SpatialBundle {
				transform : Transform {
					translation : cursor_position_minimap,
					// scale : click_point_scale,
					..default()
				},
				..default()
			},
			ClickPoint
		)).id();

		// making a separate child entity for visual component to play animation on it
		// while holder entity keeps its scale 1 / minimap_scale to avoid getting squeezed
		let click_point_visual_entity = commands.spawn((
			PbrBundle {
				mesh : mesh_assets.add(Circle::new(0.02)),
				material : color_handle.clone(),
				..default()
			},
			ClickPointVisual
		)).id();

		let tween_duration = Duration::from_millis(500);
		let tween_easing = EaseFunction::QuadraticInOut;

		let tween_fade = Tween::new(
			tween_easing,
			tween_duration,
			StandardMaterialAlphaLens {
				start : 0.0,
				end : 0.7
			}
		)
		.with_repeat_count(RepeatCount::Finite(2))
		.with_repeat_strategy(RepeatStrategy::MirroredRepeat)
		;

		let tween_scale = Tween::new(
			tween_easing,
			tween_duration,
			TransformScaleLens {
				start : Vec3::ONE,
				end : Vec3::ONE * 1.5
			}
		)
		.with_repeat_count(RepeatCount::Finite(2))
		.with_repeat_strategy(RepeatStrategy::MirroredRepeat)
		;


		commands.entity(click_point_visual_entity)
			.insert(AssetAnimator::new(tween_fade))
			.insert(Animator::new(tween_scale))
		;

		commands.entity(click_point_entity).add_child(click_point_visual_entity);

		commands.entity(self.entity).add_child(click_point_entity);
	}

	pub fn scroll_to_row(
		&self,
		start_row		: usize,
		end_row			: usize,
		commands		: &mut Commands
	) {
		let scroll_length = (end_row as i32 - start_row as i32).abs();
		let scroll_duration = (scroll_length as f32 * 1.5).clamp(200.0, 500.0) as u64;

		let tween = Tween::new(
			EaseFunction::QuadraticInOut,
			Duration::from_millis(scroll_duration),
			MinimapRowLens {
				start	: start_row,
				end		: end_row
			}
		);

		commands.entity(self.entity)
			.insert(Animator::new(tween))
			.insert(MinimapScrollAnimation::default())
		;
	}

	pub fn column_width(&self, font: &ABGlyphFont) -> f32 {
		(font.horizontal_advance_mono_rescaled(self.font_height) * self.size.x) / self.image_size.x
	}

	pub fn width(&self) -> f32 {
		self.size.x
	}
}