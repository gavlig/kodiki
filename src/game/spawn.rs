use bevy				:: prelude :: { * };
use bevy				:: core_pipeline :: clear_color :: ClearColorConfig;
use bevy_fly_camera		:: { FlyCamera };
use bevy_mod_picking	:: { * };
use bevy::render::mesh::shape as render_shape;

// use bevy_infinite_grid	:: { InfiniteGridBundle };

use super				:: { * };

pub fn camera(
	root_entity			: Entity,
	camera_ids			: &mut ResMut<CameraIDs>,
	commands			: &mut Commands
) {
	// for 2d background
	let camera2d = commands.spawn_bundle(Camera2dBundle::default()).id();

	let camera3d = commands.spawn_bundle(Camera3dBundle {
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
				priority: 1,
				..default()
			},
			..default()
		})
		.insert(
			FlyCamera {
				yaw				: 0.0,
				pitch			: 0.0,
				enabled_follow	: false,
				max_speed		: 0.07,
				target			: Some(root_entity),
				..default()
			}
		)
		.insert_bundle	(PickingCameraBundle::default())
		.id				();

	camera_ids.camera2d = Some(camera2d);
	camera_ids.camera3d = Some(camera3d);
}

pub fn ground(
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	let ground_size 	= 2000.1;
	let ground_height 	= 0.1;

	let ground			= commands
		.spawn			()
		.insert_bundle	(PbrBundle {
			mesh		: meshes.add(Mesh::from(render_shape::Box::new(ground_size * 2.0, ground_height * 2.0, ground_size * 2.0))),
			material	: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
			transform	: Transform::from_xyz(0.0, -ground_height, 0.0),
			..default()
		})
		.insert			(Transform::from_xyz(0.0, -ground_height, 0.0))
		.insert			(GlobalTransform::default())
		.id				();
		
	println!			("ground Entity ID {:?}", ground);
}

pub struct WorldAxisDesc {
	pub min_dim : f32,
	pub max_dim : f32,
	pub offset	: f32,
}

impl Default for WorldAxisDesc {
	fn default() -> Self {
		Self {
			min_dim : 0.02,
			max_dim : 0.2,
			offset	: 0.1,
		}
	}
}

pub fn world_axis(
	transform_in	: Transform,
	world_axis_desc	: WorldAxisDesc,
	meshes			: &mut ResMut<Assets<Mesh>>,
	materials		: &mut ResMut<Assets<StandardMaterial>>,
	commands		: &mut Commands,
) {
	let min_dim		= world_axis_desc.min_dim;
	let max_dim		= world_axis_desc.max_dim;
	let offset		= world_axis_desc.offset;
	let min_color	= 0.1;
	let max_color	= 0.8;
	let offset_x	= Vec3::new(offset, 0.0, 0.0);
	let offset_y	= Vec3::new(0.0, offset, 0.0);
	let offset_z	= Vec3::new(0.0, 0.0, offset);

	let mut transform = transform_in.clone();

	// X
	transform.translation = transform_in.translation + offset_x;
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(max_dim, min_dim, min_dim))),
		material	: materials.add			(Color::rgb(max_color, min_color, min_color).into()),
		transform	: transform,
		..Default::default()
	});
	// Y
	transform.translation = transform_in.translation + offset_y;
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(min_dim, max_dim, min_dim))),
		material	: materials.add			(Color::rgb(min_color, max_color, min_color).into()),
		transform	: transform,
		..Default::default()
	});
	// Z
	transform.translation = transform_in.translation + offset_z;
	commands.spawn_bundle(PbrBundle {
		mesh		: meshes.add			(Mesh::from(render_shape::Box::new(min_dim, min_dim, max_dim))),
		material	: materials.add			(Color::rgb(min_color, min_color, max_color).into()),
		transform	: transform,
		..Default::default()
	});
}

pub fn infinite_grid(
	commands		: &mut Commands,
) {
	// commands.spawn_bundle(InfiniteGridBundle::default());
}

pub fn fixed_cube(
	pose				: Transform,
	hsize				: Vec3,
	color				: Color,
	meshes				: &mut ResMut<Assets<Mesh>>,
	materials			: &mut ResMut<Assets<StandardMaterial>>,
	commands			: &mut Commands
) {
	commands.spawn_bundle(PbrBundle {
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
	commands.spawn_bundle(PbrBundle {
		mesh			: meshes.add	(Mesh::from(render_shape::UVSphere { radius: radius, ..default() } )),
		material		: materials.add	(StandardMaterial { base_color: color, unlit: true, ..default() }),
		..default()
	})
	.insert				(pose)
	.insert				(GlobalTransform::default());
}


