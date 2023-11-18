use bevy :: prelude		:: *;
use bevy :: render 		:: mesh :: shape as render_shape;
use bevy :: core_pipeline :: {
	bloom				:: { BloomSettings, BloomPrefilterSettings },
	clear_color			:: ClearColorConfig,
};

use bevy_reader_camera	:: { ReaderCamera, CameraMode };

use super				:: *;

pub fn camera(
	target_entity		: Option<Entity>,
	camera_ids			: &mut ResMut<CameraIDs>,
	commands			: &mut Commands
) {
	// for 2d background
	let camera2d = commands.spawn(Camera2dBundle {
		camera: Camera { hdr: true, ..default() },
		..default()
	}).id();

	let camera3d = commands.spawn((
		Camera3dBundle {
			transform: Transform {
				translation: Vec3::new(1.5, 0., 7.),
				..default()
			},
			camera_3d: Camera3d {
				// don't clear the color while rendering this camera
				clear_color: ClearColorConfig::None,
				..default()
			},
			camera: Camera {
				// renders after / on top of the main camera
				order: 1,
				// needed for bloom
				hdr: true,
				..default()
			},
			..default()
		},
		VisibilityBundle {
			visibility	: Visibility::Visible,
			..default()
		},
		BloomSettings {
			prefilter_settings: BloomPrefilterSettings {
				threshold: 1.0, // without filtering all materials are "blooming" a little
				..default()
			},
			..default()
		},
	)).id();

	camera_ids.camera2d = Some(camera2d);
	camera_ids.camera3d = Some(camera3d);

	let mut reader_camera = ReaderCamera::default();
	reader_camera.set_mode_wrestrictions(CameraMode::Reader, false, false, false, true);
	reader_camera.target_entity = target_entity;

	commands.entity(camera3d).insert(reader_camera);
}

pub struct AxisDesc {
	pub min_dim : f32,
	pub max_dim : f32,
	pub offset	: f32,
}

impl Default for AxisDesc {
	fn default() -> Self {
		Self {
			min_dim : 0.02,
			max_dim : 0.2,
			offset	: 0.1,
		}
	}
}

pub fn axis(
	transform_in	: Transform,
	world_axis_desc	: AxisDesc,
	meshes			: &mut Assets<Mesh>,
	materials		: &mut Assets<StandardMaterial>,
	commands		: &mut Commands,
) -> Entity
{
	let min_dim		= world_axis_desc.min_dim;
	let max_dim		= world_axis_desc.max_dim;
	let offset		= world_axis_desc.offset;
	let min_color	= 0.1;
	let max_color	= 0.8;
	let offset_x	= Vec3::new(offset, 0.0, 0.0);
	let offset_y	= Vec3::new(0.0, offset, 0.0);
	let offset_z	= Vec3::new(0.0, 0.0, offset);

	let mut transform = transform_in.clone();

	let root_entity = commands.spawn(TransformBundle {
		local		: transform_in,
		..default()
	})
	.insert(VisibilityBundle {
		visibility	: Visibility::Visible,
		..default()
	})
	.id();

	// X
	transform.translation = transform_in.translation + offset_x;
	let x_entity = commands.spawn(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(max_dim, min_dim, min_dim))),
		material	: materials.add			(Color::rgb(max_color, min_color, min_color).into()),
		transform	: transform,
		..Default::default()
	}).id();
	// Y
	transform.translation = transform_in.translation + offset_y;
	let y_entity = commands.spawn(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(min_dim, max_dim, min_dim))),
		material	: materials.add			(Color::rgb(min_color, max_color, min_color).into()),
		transform	: transform,
		..Default::default()
	}).id();
	// Z
	transform.translation = transform_in.translation + offset_z;
	let z_entity = commands.spawn(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(min_dim, min_dim, max_dim))),
		material	: materials.add			(Color::rgb(min_color, min_color, max_color).into()),
		transform	: transform,
		..Default::default()
	}).id();

	commands.entity(root_entity).add_child(x_entity);
	commands.entity(root_entity).add_child(y_entity);
	commands.entity(root_entity).add_child(z_entity);

	root_entity
}

pub fn fixed_cube(
	pose				: Transform,
	hsize				: Vec3,
	color				: Color,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	commands.spawn		(PbrBundle {
		mesh			: meshes.add	(Mesh::from(render_shape::Box::new(hsize.x * 2.0, hsize.y * 2.0, hsize.z * 2.0))),
		material		: materials.add	(color.into()),
		..default()
	})
	.insert				(pose)
	.insert				(GlobalTransform::default());
}

pub fn fixed_sphere(
	pose				: Transform,
	radius				: f32,
	color				: Color,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	commands.spawn		(PbrBundle {
		mesh			: meshes.add	(Mesh::from(render_shape::UVSphere { radius: radius, ..default() } )),
		material		: materials.add	(StandardMaterial { base_color: color, unlit: true, ..default() }),
		..default()
	})
	.insert				(pose)
	.insert				(GlobalTransform::default());
}


