use bevy :: prelude :: *;
use bevy :: render :: render_resource :: PrimitiveTopology;
use bevy :: render :: mesh :: { Indices, Mesh };

#[cfg(feature = "debug")]
use bevy_debug_text_overlay	:: screen_print;
#[cfg(feature = "debug")]
use bevy_polyline :: prelude :: *;

use ab_glyph :: { Font, FontVec, Outline, OutlineCurve };

use lyon :: path :: Path;
use lyon :: tessellation :: *;

pub type ABGlyphPoint	= ab_glyph :: Point;
pub type LyonPoint		= lyon :: math :: Point;

use super :: { GlyphMeshesCache, TextMeshesCache, GlyphWithFonts, ABGlyphFont };
use super :: generator_common :: *;

fn generate_glyph_outline(
	glyph_char	: char,
	font		: &FontVec,
	debug		: bool,
) -> Option<Outline> {
	let glyph_id = font.glyph_id(glyph_char);

	if debug {
		println!("generate_glyph_outline glyph_id: {:?} char {}", glyph_id, glyph_char);
	}

	font.outline(glyph_id)
}

fn generate_path_from_outline(
	outline: Outline
) -> Path {
	let mut path_builder = Path::builder();

	// handle first point
	let first_curve = &outline.curves[0];
	let (first_point, mut last_point) = match first_curve {
		OutlineCurve::Line(p0, p1)			=> (p0, p1),
		OutlineCurve::Quad(p0, _, p2)		=> (p0, p2),
		OutlineCurve::Cubic(p0, _, _, p3)	=> (p0, p3),
	};

	path_builder.begin(LyonPoint::new(first_point.x, first_point.y));

	//

	for (i, curve) in outline.curves.iter().enumerate() {
		match curve {
			// Straight line from `.0` to `.1`.
			OutlineCurve::Line(p0, p1) => {
				if last_point != p0 && i > 0 {
					path_builder.end(false);
					path_builder.begin(LyonPoint::new(p0.x, p0.y));
				}

				path_builder.line_to(LyonPoint::new(p1.x, p1.y));

				last_point = p1;
			}
			// Quadratic Bézier curve from `.0` to `.2` using `.1` as the control.
			OutlineCurve::Quad(p0, p1, p2) => {
				if last_point != p0 && i > 0 {
					path_builder.end(false);
					path_builder.begin(LyonPoint::new(p0.x, p0.y));
				}

				path_builder.quadratic_bezier_to(
					LyonPoint::new(p1.x, p1.y),
					LyonPoint::new(p2.x, p2.y)
				);

				last_point = p2;
			}
			// Cubic Bézier curve from `.0` to `.3` using `.1` as the control at the beginning of the
			// curve and `.2` at the end of the curve.
			OutlineCurve::Cubic(p0, p1, p2, p3) => {
				if last_point != p0 && i > 0 {
					path_builder.end(false);
					path_builder.begin(LyonPoint::new(p0.x, p0.y));
				}

				path_builder.cubic_bezier_to(
					LyonPoint::new(p1.x, p1.y),
					LyonPoint::new(p2.x, p2.y),
					LyonPoint::new(p3.x, p3.y)
				);

				last_point = p3;
			}
		}
	}

	path_builder.end(/*close=*/true);

	path_builder.build()
}

fn fill_vertex_buffer_from_path(
	path		: Path,
	scale		: f32,
	tolerance	: f32,
	vertex_buffer : &mut VertexBuffer
) {
	let mut tessellator = FillTessellator::new();
	tessellator.tessellate_path(
		&path,
		&FillOptions::tolerance(tolerance),
		&mut BuffersBuilder::new(vertex_buffer, |vertex: FillVertex| {
			let pos2d = vertex.position() * scale;
			[ pos2d.x, pos2d.y, 0.0 ]
		}).with_inverted_winding(),
	).unwrap();
}

#[derive(Debug, Clone, Copy)]
struct Edge {
	pub i0 : u16,
	pub i1 : u16,
	pub adjacent : bool,
}

impl Edge {
	pub fn is_adjacent(&self, tri: &Triangle) -> bool {
		for e in 0 .. 3 {
			if (tri.edges[e].i0 == self.i0 && tri.edges[e].i1 == self.i1)
			|| (tri.edges[e].i0 == self.i1 && tri.edges[e].i1 == self.i0)
			{
				return true;
			}
		}

		return false;
	}
}

#[derive(Debug, Clone, Copy)]
struct Triangle {
	pub edges : [Edge; 3]
}

impl Triangle {
	pub fn test_and_set_adjacent(&mut self, other_tri: &Triangle) {
		for e in 0 .. 3 {
			let edge = &mut self.edges[e];
			if edge.is_adjacent(other_tri) {
				edge.adjacent = true;
			}
		}
	}
}

fn collect_triangles_from_vertices(
	vertex_buffer: &VertexBuffer
) -> Vec<Triangle> {
	let triangles_cnt = vertex_buffer.indices.len() / 3;

	let mut triangles : Vec<Triangle> = Vec::with_capacity(triangles_cnt);
	for iter in 0 .. triangles_cnt {
		let i0 = vertex_buffer.indices[(iter * 3) + 0];
		let i1 = vertex_buffer.indices[(iter * 3) + 1];
		let i2 = vertex_buffer.indices[(iter * 3) + 2];

		let edge0 = Edge { i0: i0, i1: i1, adjacent: false };
		let edge1 = Edge { i0: i1, i1: i2, adjacent: false };
		let edge2 = Edge { i0: i2, i1: i0, adjacent: false };

		triangles.push(Triangle { edges: [edge0, edge1, edge2] });
	}

	for iter0 in 0 .. triangles_cnt {
		for iter1 in 0 .. triangles_cnt {
			if iter0 == iter1 {
				continue;
			}

			let tri1 = triangles[iter1].clone();
			let tri0 = &mut triangles[iter0];

			tri0.test_and_set_adjacent(&tri1);
		}
	}

	triangles
}

fn run_triangle_adjacency_tests(
	triangles : &mut Vec<Triangle>
) {
	let triangles_cnt = triangles.len();

	// test every trangle against each other to find adjacent edges (adjacent = edges shared between more than 1 triangle)
	for iter0 in 0 .. triangles_cnt {
		for iter1 in 0 .. triangles_cnt {
			if iter0 == iter1 {
				continue;
			}

			let tri1 = triangles[iter1].clone();
			let tri0 = &mut triangles[iter0];

			tri0.test_and_set_adjacent(&tri1);
		}
	}
}

fn generate_glyph_back_face(
	depth: f32,
	vertex_buffer: &mut VertexBuffer,
	normals: &mut Vec<[f32; 3]>
) {
	let vertices_cnt = vertex_buffer.vertices.len();
	let indices_cnt = vertex_buffer.indices.len();
	let triangles_cnt = indices_cnt / 3;

	let mut back_vertices = vertex_buffer.vertices.clone();
	for v in back_vertices.iter_mut() {
		// z coordinate gets negative offset of "depth"
		v[2] -= depth;
	}

	// inverted winding + offset to index over back_vertices
	let mut back_indices : Vec<u16> = Vec::with_capacity(indices_cnt);
	for i in 0 .. triangles_cnt {
		back_indices.push(vertex_buffer.indices[i * 3 + 0] + vertices_cnt as u16);
		back_indices.push(vertex_buffer.indices[i * 3 + 2] + vertices_cnt as u16);
		back_indices.push(vertex_buffer.indices[i * 3 + 1] + vertices_cnt as u16);
	}

	// inverted normals
	let mut back_normals = normals.clone();
	for n in back_normals.iter_mut() {
		// only z coordinate gets inverted since since we assume glyph always faces forward as in +z
		n[2] *= -1.0;
	}

	// store results back into vertex_buffer
	vertex_buffer.vertices.append(&mut back_vertices);
	vertex_buffer.indices.append(&mut back_indices);
	normals.append(&mut back_normals);
}

fn generate_connecting_quads(
	triangles: &Vec<Triangle>,
	vertex_buffer: &mut VertexBuffers<[f32; 3], u16>,
	normals: &mut Vec<[f32; 3]>,
) {
	let vertices_cnt = vertex_buffer.vertices.len() / 2; // dividing by two because it is expected that vertex buffer already has geometry for front and back face
	let backface_offset = vertices_cnt as u16;

	for tri in triangles.iter() {
		for edge in tri.edges.iter() {
			if edge.adjacent {
				continue;
			}

			// add new vertices with same coords but different normals for better shading
			// one connecting quad between two non ajdacent edges == two triangles:

			// first triangle
			let i0 = edge.i1;
			let i1 = edge.i0;
			let i2 = edge.i0 + backface_offset;

			let p0 = vertex_buffer.vertices[i0 as usize].clone();
			let p1 = vertex_buffer.vertices[i1 as usize].clone();
			let p2 = vertex_buffer.vertices[i2 as usize].clone();

			let last_geom_id = vertex_buffer.vertices.len() as u16;
			vertex_buffer.vertices.extend_from_slice(&[p0, p1, p2]);
			vertex_buffer.indices.extend_from_slice(&[last_geom_id, last_geom_id + 1, last_geom_id + 2]);

			let vec0 = (Vec3::from_array(p1) - Vec3::from_array(p0)).normalize();
			let vec1 = (Vec3::from_array(p2) - Vec3::from_array(p0)).normalize();
			let normal = vec0.cross(vec1).normalize();
			normals.extend_from_slice(&[normal.into(); 3]);

			// second triange
			let i3 = edge.i0 + backface_offset;
			let i4 = edge.i1 + backface_offset;
			let i5 = edge.i1;

			let p3 = vertex_buffer.vertices[i3 as usize].clone();
			let p4 = vertex_buffer.vertices[i4 as usize].clone();
			let p5 = vertex_buffer.vertices[i5 as usize].clone();

			let last_geom_id = vertex_buffer.vertices.len() as u16;
			vertex_buffer.vertices.extend_from_slice(&[p3, p4, p5]);
			vertex_buffer.indices.extend_from_slice(&[last_geom_id, last_geom_id + 1, last_geom_id + 2]);

			let vec0 = (Vec3::from_array(p4) - Vec3::from_array(p3)).normalize();
			let vec1 = (Vec3::from_array(p5) - Vec3::from_array(p3)).normalize();
			let normal = vec0.cross(vec1).normalize();
			normals.extend_from_slice(&[normal.into(); 3]);
		}
	}
}

pub fn generate_glyph_mesh_internal(
	glyph_char	: char,
	font		: &ABGlyphFont
) -> MeshInternal {
	if let Some(glyph_outline) = generate_glyph_outline(glyph_char, &font.f, false) {
		let path			= generate_path_from_outline(glyph_outline);

		// geometry of a glyph's front face
		let mut vertex_buffer = VertexBuffers::new();
		let scale			= (1.0 / font.f.units_per_em().unwrap()) * font.scale;
		fill_vertex_buffer_from_path(path, scale, font.tolerance, &mut vertex_buffer);
		let vertices_cnt	= vertex_buffer.vertices.len();
		let normals			= vec![[0.0, 0.0, 1.0]; vertices_cnt];

		// Now to "extrude" the said geometry to get a 3d glyph first we need to find the edges that are not adjacent with others.
		// Or in other words we need to find the contour edges

		// // collect vertices into triangles with 3 edges to find adjacent edges
		// let mut triangles = collect_triangles_from_vertices(&vertex_buffer);

		// // find adjacent edges and mark them in "triangles"
		// run_triangle_adjacency_tests(&mut triangles);

		// // make back face with inverted winding and normals
		// generate_glyph_back_face(font.depth, &mut vertex_buffer, &mut normals);

		// // make connecting quads
		// generate_connecting_quads(&triangles, &mut vertex_buffer, &mut normals);

		MeshInternal { vertex_buffer, normals, uvs: None }
	} else {
		let mut mesh = MeshInternal::default();
		generate_quad_vertices(&mut mesh, font.scale);

		mesh
	}
}

pub fn generate_glyph_mesh_wcache(
	glyph_with_fonts	: &GlyphWithFonts,
	mesh_assets			: &mut Assets<Mesh>,
	text_meshes_cache 	: &mut TextMeshesCache
) -> Handle<Mesh> {
	match text_meshes_cache.meshes.get(glyph_with_fonts.glyph_str) {
		Some(handle) => handle.clone_weak(),
		None => {
			let glyph_char = glyph_with_fonts.glyph_str.chars().next().unwrap();
			let mesh_internal = generate_glyph_mesh_internal(glyph_char, glyph_with_fonts.current_font());

			let handle = mesh_assets.add(
				bevy_mesh_from_internal(&mesh_internal)
			);

			text_meshes_cache.meshes.insert_unique_unchecked(glyph_with_fonts.glyph_str.clone(), handle).1.clone()
		}
	}
}

fn offset_vertices(
	vertices: &mut Vec<[f32; 3]>,
	offset_x: f32,
	offset_y: f32,
) {
	for v in vertices.iter_mut() {
		v[0] += offset_x;
		v[1] += offset_y;
	}
}

fn offset_indices(
	indices: &mut Vec<u16>,
	offset: u16
) {
	if offset == 0 {
		return;
	}

	for i in indices.iter_mut() {
		*i += offset;
	}
}

pub fn generate_string_mesh(
		string				: &String,
		font				: &ABGlyphFont,
	mut glyph_meshes_cache	: Option<&mut GlyphMeshesCache>,
) -> Mesh {
	let mut vertex_buffer_string: VertexBuffer = VertexBuffers::new();
	let mut normals_string		: NormalBuffer = Vec::new();

	let mut mesh	= Mesh::new(PrimitiveTopology::TriangleList);
	let mut x		= 0.0;

	let generate_glyph_mesh_with_optional_cache = |glyph_char: char, glyph_meshes_cache: &mut Option<&mut GlyphMeshesCache>| -> MeshInternal {
		if let Some(cache) = glyph_meshes_cache {
			match cache.meshes.get(&glyph_char) {
				Some(mesh) => mesh.clone(),
				None => {
					let mesh_internal = generate_glyph_mesh_internal(glyph_char, font);
					cache.meshes.insert_unique_unchecked(glyph_char, mesh_internal).1.clone()
				}
			}
		} else {
			generate_glyph_mesh_internal(glyph_char, font)
		}
	};

	for glyph_char in string.chars() {
		if glyph_char == ' ' {
			x += font.horizontal_advance_mono();
			continue;
		}

		let mut mesh_internal = generate_glyph_mesh_with_optional_cache(glyph_char, &mut glyph_meshes_cache);

		// add horizontal offset if it's not the first char in a string and inverted descent to embed it into mesh geometry
		offset_vertices(mesh_internal.vertices_mut(), x, -font.descent());

		// offset indices since vertex_buffer_string already has some geometry
		assert!((vertex_buffer_string.vertices.len() as u16) < u16::MAX);
		offset_indices(mesh_internal.indices_mut(), vertex_buffer_string.vertices.len() as u16);

		vertex_buffer_string.vertices.append(mesh_internal.vertices_mut());
		vertex_buffer_string.indices.append(mesh_internal.indices_mut());
		normals_string.append(&mut mesh_internal.normals);

		// accumulate horizontal offset from current glyph for next glyphs
		x += font.horizontal_advance_char(glyph_char);
	}

	mesh.insert_attribute	(Mesh::ATTRIBUTE_POSITION, vertex_buffer_string.vertices);
	mesh.insert_attribute	(Mesh::ATTRIBUTE_NORMAL, normals_string);
	mesh.set_indices(Some	(Indices::U16(vertex_buffer_string.indices)));

	mesh
}

pub fn generate_string_mesh_wcache(
	string				: &String,
	font				: &ABGlyphFont,
	mesh_assets			: &mut Assets<Mesh>,
	glyph_meshes_cache	: &mut GlyphMeshesCache,
	text_meshes_cache	: &mut TextMeshesCache
) -> Handle<Mesh> {
	match text_meshes_cache.meshes.get(string) {
		Some(handle) => handle.clone_weak(),
		None => {
			let handle = mesh_assets.add(
				generate_string_mesh(string, font, Some(glyph_meshes_cache))
			);

			text_meshes_cache.meshes.insert_unique_unchecked(string.clone(), handle).1.clone()
		}
	}
}

#[cfg(feature = "debug")]
use bevy::render::mesh::shape as render_shape;

#[cfg(feature = "debug")]
fn spawn_sphere(
	i: usize,
	p: &ABGlyphPoint,
	meshes: &mut Assets<Mesh>,
	materials: &mut Assets<StandardMaterial>,
	commands: &mut Commands
) {
	commands.spawn(
		PbrBundle {
			mesh			: meshes.add(Mesh::from(render_shape::UVSphere{ radius: 0.01, ..default() })), // 0.225
			material		: materials.add(
			StandardMaterial {
				base_color	: Color::LIME_GREEN.into(),
				// unlit		: true,
				..default()
			}),
			transform		: Transform::from_translation(Vec3::new(p.x / 500., p.y / 500., 1.0)),
			..Default::default()
		})
		.insert				(ABGlyphCurveDebug { i, p0: *p, ..default() })
	;
}

#[cfg(feature = "debug")]
fn spawn_sphere2(
	i: usize,
	p: Vec3,
	color: Color,
	meshes: &mut Assets<Mesh>,
	materials: &mut Assets<StandardMaterial>,
	commands: &mut Commands
) {
	commands.spawn(
		PbrBundle {
			mesh			: meshes.add(Mesh::from(render_shape::UVSphere{ radius: 0.005, ..default() })),
			material		: materials.add(
			StandardMaterial {
				base_color	: color.into(),
				// unlit		: true,
				..default()
			}),
			transform		: Transform::from_translation(p),
			..Default::default()
		})
		.insert				(ABGlyphCurveDebug { i, p1: p, ..default() })
	;
}

#[cfg(feature = "debug")]
fn spawn_line(
	p0: Vec3,
	p1: Vec3,
	polylines: &mut Assets<Polyline>,
	polyline_materials: &mut Assets<PolylineMaterial>,
	commands: &mut Commands
) {
	commands.spawn(PolylineBundle {
		polyline: polylines.add(Polyline {
			vertices: vec![p0, p1],
			..default()
		}),
		material: polyline_materials.add(PolylineMaterial {
			width: 5.,
			color: Color::LIME_GREEN,
			perspective: true,
			..default()
		}),
		..default()
	});
}

#[cfg(feature = "debug")]
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ABGlyphCurveDebug {
	pub i: usize,
	pub p0: ABGlyphPoint,
	pub p1: Vec3
}

#[cfg(feature = "debug")]
use crate::kodiki_ui::raypick::RaypickHover;

#[cfg(feature = "debug")]
pub fn ab_glyph_curve_debug_system(
	q_hover : Query<(&RaypickHover, &ABGlyphCurveDebug)>,
) {
	if q_hover.is_empty() {
		return;
	}

	for (hover, dbg) in q_hover.iter() {
		if !hover.hovered() {
			continue;
		}

		screen_print!("hovered over {} {:?}", dbg.i, dbg.p1);
	}
}