use bevy :: prelude :: { * };
use bevy :: render :: render_resource :: PrimitiveTopology;
use bevy :: render :: mesh :: { Indices, Mesh, VertexAttributeValues };
use bevy_mod_picking :: { * };

use bevy_debug_text_overlay	:: { screen_print };
use bevy_polyline :: prelude :: { * };

use ab_glyph :: { Font, FontVec, OutlineCurve, GlyphId };

use lyon :: geom :: { LineSegment };
use lyon :: path :: traits :: PathBuilder;
use lyon :: path :: { Path };
use lyon :: tessellation :: { * };

pub type ABGlyphPoint   = ab_glyph :: Point;
pub type LyonPoint      = lyon :: math :: Point;

use bevy::render::mesh::shape as render_shape;

pub fn generate_glyph_mesh(
	glyph_str: &String,
	font: &FontVec,
) -> Mesh {
	// println!("generate_glyph_mesh for {} called!", glyph_str);

	let placeholder_glyph_id = font.glyph_id('?');

	let glyph_id = font.glyph_id(glyph_str.chars().next().unwrap());

	let mut outline = font.outline(glyph_id);
	// couldn't find outline for requested character, use placeholder instead
	if outline.is_none() {
		// println!("glyph id for {} not found!", glyph_str);
		outline = font.outline(placeholder_glyph_id);
	}

	let outline = outline.unwrap();
	// println!("got outline with {} curves!", outline.curves.len());

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

	let path = path_builder.build();

	#[derive(Copy, Clone, Debug)]
	struct Vertex3D { position: [f32; 3] }
	let mut geometry: VertexBuffers<[f32; 3], u16> = VertexBuffers::new();
	let mut tessellator = FillTessellator::new();

	{ // tesselate glyph and get geometry of a front face
		tessellator.tessellate_path(
			&path,
			&FillOptions::tolerance(1.0),
			&mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
				let pos2d = vertex.position() / 500.;
				[ pos2d.x, pos2d.y, 0.0 ]
			}).with_inverted_winding(),
		).unwrap();
	}

	// Now to "extrude" the said geometry to get a 3d glyph first we need to find the edges that are not adjacent with others. 
	// Or in other words we need to find the contour edges

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
				let mut edge = &mut self.edges[e];
				if edge.is_adjacent(other_tri) {
					edge.adjacent = true;
				}
			}
		}
	}

	let indices_cnt = geometry.indices.len();
	let triangles_cnt = indices_cnt / 3;

	// collect geometry into triangles with 3 edges to find adjacent edges
	let mut triangles : Vec<Triangle> = Vec::with_capacity(triangles_cnt);
	for iter in 0 .. triangles_cnt {
		let i0 = geometry.indices[(iter * 3) + 0];
		let i1 = geometry.indices[(iter * 3) + 1];
		let i2 = geometry.indices[(iter * 3) + 2];

		let edge0 = Edge { i0: i0, i1: i1, adjacent: false };
		let edge1 = Edge { i0: i1, i1: i2, adjacent: false };
		let edge2 = Edge { i0: i2, i1: i0, adjacent: false };

		let mut new_tri = Triangle { edges: [edge0, edge1, edge2] };

		triangles.push(new_tri);
	}

	// Go over all triangles and compare their edges. Edges found in both triangles are going to be marked adjacent
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

	let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; geometry.vertices.len()];

	let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, geometry.vertices);
	mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
	mesh.set_indices(Some(Indices::U16(geometry.indices)));

	mesh
}

fn spawn_sphere(
	i: usize,
	p: &ABGlyphPoint,
	meshes: &mut Assets<Mesh>,
	materials: &mut Assets<StandardMaterial>,
	commands: &mut Commands
) {
	commands.spawn_bundle(
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
		.insert_bundle		(PickableBundle::default());
}

fn spawn_sphere2(
	i: usize,
	p: Vec3,
	color: Color,
	meshes: &mut Assets<Mesh>,
	materials: &mut Assets<StandardMaterial>,
	commands: &mut Commands
) {
	commands.spawn_bundle(
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
		// .insert_bundle		(PickableBundle::default())
		;
}

fn spawn_line(
	i: usize,
	p0: Vec3,
	p1: Vec3,
	polylines: &mut Assets<Polyline>,
	polyline_materials: &mut Assets<PolylineMaterial>,
	commands: &mut Commands
) {
	commands.spawn_bundle(PolylineBundle {
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

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ABGlyphCurveDebug {
    pub i: usize,
    pub p0: ABGlyphPoint,
	pub p1: Vec3
}

pub fn ab_glyph_curve_debug_system(
    q_hover : Query<(&Hover, &ABGlyphCurveDebug)>,
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

pub fn generate_glyph_mesh_dbg(
	glyph_str: &String,
	font: &FontVec,
	depth: f32,

	meshes: &mut Assets<Mesh>,
	materials: &mut Assets<StandardMaterial>,
	polylines: &mut Assets<Polyline>,
	polyline_materials: &mut Assets<PolylineMaterial>,
	commands: &mut Commands
) -> Handle<Mesh> {
	// println!("generate_glyph_mesh for {} called!", glyph_str);

	let placeholder_glyph_id = font.glyph_id('?');

	let glyph_id = font.glyph_id(glyph_str.chars().next().unwrap());

	let mut outline = font.outline(glyph_id);
	// couldn't find outline for requested character, use placeholder instead
	if outline.is_none() {
		// println!("glyph id for {} not found!", glyph_str);
		outline = font.outline(placeholder_glyph_id);
	}

	let outline = outline.unwrap();
	// println!("got outline with {} curves!", outline.curves.len());

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

	let path = path_builder.build();

	// Let's use our own custom vertex type instead of the default one.
	#[derive(Copy, Clone, Debug)]
	struct Vertex3D { position: [f32; 3] }

	// Will contain the result of the tessellation.
	let mut geometry: VertexBuffers<[f32; 3], u16> = VertexBuffers::new();
	let mut tessellator = FillTessellator::new();

	{
		tessellator.tessellate_path(
			&path,
			&FillOptions::tolerance(1.0),
			&mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
				let pos2d = vertex.position() / 500.;
				[ pos2d.x, pos2d.y, 0.0 ]
			}).with_inverted_winding(),
		).unwrap();
	}

	let mut normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; geometry.vertices.len()];

	//

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
				let mut edge = &mut self.edges[e];
				if edge.is_adjacent(other_tri) {
					edge.adjacent = true;

					// println!("tri {} found adjacent edge {} ", iter, e);
					// let p0 = Vec3::new(geometry.vertices[edge.i0 as usize][0], geometry.vertices[edge.i0 as usize][1], 1.0);
					// let p1 = Vec3::new(geometry.vertices[edge.i1 as usize][0], geometry.vertices[edge.i1 as usize][1], 1.0);
					// spawn_line(0, p0, p1, polylines, polyline_materials, commands);
				}
			}
		}
	}

	let vertices_cnt = geometry.vertices.len();
	let indices_cnt = geometry.indices.len();
	let triangles_cnt = indices_cnt / 3;

	// let mut edges : Vec<Edge> = Vec::with_capacity(indices_cnt);
	let mut triangles : Vec<Triangle> = Vec::with_capacity(triangles_cnt);

	for iter in 0 .. triangles_cnt {
		let i0 = geometry.indices[(iter * 3) + 0];
		let i1 = geometry.indices[(iter * 3) + 1];
		let i2 = geometry.indices[(iter * 3) + 2];

		let edge0 = Edge { i0: i0, i1: i1, adjacent: false };
		let edge1 = Edge { i0: i1, i1: i2, adjacent: false };
		let edge2 = Edge { i0: i2, i1: i0, adjacent: false };

		// let p0 = Vec3::new(geometry.vertices[edge0.i0 as usize][0], geometry.vertices[edge0.i0 as usize][1], 1.0);
		// let p1 = Vec3::new(geometry.vertices[edge0.i1 as usize][0], geometry.vertices[edge0.i1 as usize][1], 1.0);
		// spawn_line(0, p0, p1, polylines, polyline_materials, commands);

		// let p2 = Vec3::new(geometry.vertices[edge1.i0 as usize][0], geometry.vertices[edge1.i0 as usize][1], 1.0);
		// let p3 = Vec3::new(geometry.vertices[edge1.i1 as usize][0], geometry.vertices[edge1.i1 as usize][1], 1.0);
		// spawn_line(0, p2, p3, polylines, polyline_materials, commands);

		// let p4 = Vec3::new(geometry.vertices[edge2.i0 as usize][0], geometry.vertices[edge2.i0 as usize][1], 1.0);
		// let p5 = Vec3::new(geometry.vertices[edge2.i1 as usize][0], geometry.vertices[edge2.i1 as usize][1], 1.0);
		// spawn_line(0, p4, p5, polylines, polyline_materials, commands);
		
		let mut new_tri = Triangle { edges: [edge0, edge1, edge2] };

		println!("pushing tri {} {:?}", iter, new_tri);
		triangles.push(new_tri);
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

	println!("\n\n");

	for (i, tri) in triangles.iter().enumerate() {
		println!("{} tri", i);

		for edge in tri.edges.iter() {
			if edge.adjacent {
				continue;
			}

			println!("	edge {:?}", edge);

			let p0 = Vec3::new(geometry.vertices[edge.i0 as usize][0], geometry.vertices[edge.i0 as usize][1], 1.05);
			let p1 = Vec3::new(geometry.vertices[edge.i1 as usize][0], geometry.vertices[edge.i1 as usize][1], 1.05);
			spawn_line(0, p0, p1, polylines, polyline_materials, commands);
		}
	}

	// make back face with inverted winding and normals
	let mut back_vertices = geometry.vertices.clone();
	for v in back_vertices.iter_mut() {
		// z coordinate gets negative offset of "depth"
		v[2] -= depth;

		spawn_sphere2(0, Vec3::new(v[0], v[1], v[2] + 1.0), Color::YELLOW, meshes, materials, commands);
	}

	// inverted winding + offset to index over back_vertices
	let mut back_indices : Vec<u16> = Vec::with_capacity(indices_cnt);
	for i in 0 .. triangles_cnt {
		back_indices.push(geometry.indices[i * 3 + 0] + vertices_cnt as u16);
		back_indices.push(geometry.indices[i * 3 + 2] + vertices_cnt as u16);
		back_indices.push(geometry.indices[i * 3 + 1] + vertices_cnt as u16);
	}

	// inverted normals
	let mut back_normals = normals.clone();
	for n in back_normals.iter_mut() {
		// z coordinate gets inverted since it's a backface
		n[2] *= -1.0;
	}

	println!("vertices before: {} indices before: {} normals before: {}", geometry.vertices.len(), geometry.indices.len(), normals.len());

	geometry.vertices.append(&mut back_vertices);
	geometry.indices.append(&mut back_indices);
	normals.append(&mut back_normals);

	println!("vertices after: {} indices after: {} normals after: {}", geometry.vertices.len(), geometry.indices.len(), normals.len());

	// println!("all vertices:");
	// for (iter, v) in geometry.vertices.iter().enumerate() {
	// 	println!("{} {:?}", iter, v);
	// }

	// println!("all indices:");
	// for (iter, i) in geometry.indices.iter().enumerate() {
	// 	println!("{} {}", iter, i);
	// }

	// make connecting edges

	// make side triangles off connecting edges

	// for (i, v) in geometry.vertices.iter().enumerate() {
	// 	spawn_sphere2(i, Vec3::new(v[0], v[1], 1.0), meshes, materials, commands);
	// }

	

	let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, geometry.vertices);
	mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
	mesh.set_indices(Some(Indices::U16(geometry.indices)));

	meshes.add(mesh)
}