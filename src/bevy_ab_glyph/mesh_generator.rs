use bevy :: prelude :: { * };
use bevy :: render :: render_resource :: PrimitiveTopology;
use bevy :: render :: mesh :: { Indices, Mesh, VertexAttributeValues };

use ab_glyph :: { Font, FontVec, OutlineCurve, GlyphId };

use lyon :: geom :: { LineSegment };
use lyon :: path :: traits :: PathBuilder;
use lyon :: path :: { Path };
use lyon :: tessellation :: { * };

pub type ABGlyphPoint   = ab_glyph :: Point;
pub type LyonPoint      = lyon :: math :: Point;

pub fn generate_glyph_mesh(
	glyph_str: &String,
	font: &FontVec,
	meshes: &mut Assets<Mesh>,
	// cache: Option<&mut TTF2MeshCache>,
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
	let first_point = match first_curve {
		OutlineCurve::Line(p0, p1)            => LyonPoint::new(p0.x, p0.y),
		OutlineCurve::Quad(p0, p1, p2)        => LyonPoint::new(p0.x, p0.y),
		OutlineCurve::Cubic(p0, p1, p2, p3)   => LyonPoint::new(p0.x, p0.y),
	};

	path_builder.begin(first_point);

	// 

	// let mut i = 0;
	for curve in outline.curves {
		match curve {
			// Straight line from `.0` to `.1`.
			OutlineCurve::Line(p0, p1) => {
				// println!("[{}] OutlineCurve::Line {:?} {:?}", i, p0, p1);
				path_builder.line_to(LyonPoint::new(p1.x, p1.y));
			}
			// Quadratic Bézier curve from `.0` to `.2` using `.1` as the control.
			OutlineCurve::Quad(p0, p1, p2) => {
				// println!("[{}] OutlineCurve::Quad {:?} {:?} {:?}", i, p0, p1, p2);
				path_builder.quadratic_bezier_to(
					LyonPoint::new(p1.x, p1.y),
					LyonPoint::new(p2.x, p2.y)
				);
			}
			// Cubic Bézier curve from `.0` to `.3` using `.1` as the control at the beginning of the
			// curve and `.2` at the end of the curve.
			OutlineCurve::Cubic(p0, p1, p2, p3) => {
				// println!("[{}] OutlineCurve::Cubic {:?} {:?} {:?} {:?}", i, p0, p1, p2, p3);
				path_builder.cubic_bezier_to(
					LyonPoint::new(p1.x, p1.y),
					LyonPoint::new(p2.x, p2.y),
					LyonPoint::new(p3.x, p3.y)
				);
			}
		}

		// i += 1;
	}

	path_builder.end(/*close=*/true);

	let path = path_builder.build();

	// Let's use our own custom vertex type instead of the default one.
	#[derive(Copy, Clone, Debug)]
	struct Vertex3D { position: [f32; 3] };

	// VertexAttributeValues::Float32x3

	// Will contain the result of the tessellation.
	let mut geometry: VertexBuffers<[f32; 3], u16> = VertexBuffers::new();
	let mut tessellator = FillTessellator::new();

	{
		// Compute the tessellation.
		tessellator.tessellate_path(
			&path,
			&FillOptions::default(),
			&mut BuffersBuilder::new(&mut geometry, |vertex: FillVertex| {
				let pos2d = vertex.position() / 500.;
				// Vertex3D {
				// 	position: [ pos2d.x, pos2d.y, 0.0],
				// }

				println!("pos2d {:?}", pos2d);

				[ pos2d.x, pos2d.y, 0.0 ]
			}),
		).unwrap();
	}

	// The tessellated geometry is ready to be uploaded to the GPU.
	// println!(" -- {} vertices {} indices",
	// 	geometry.vertices.len(),
	// 	geometry.indices.len()
	// );

	let normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 1.0]; geometry.vertices.len()];

	let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
	mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, geometry.vertices);
	mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
	mesh.set_indices(Some(Indices::U16(geometry.indices)));

	meshes.add(mesh)
}