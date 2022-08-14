use bevy				:: prelude :: { * };
use bevy_rapier3d		:: prelude :: { * };
use bevy_mod_picking	:: { * };
use bevy_polyline		:: { prelude :: * };
use bevy_mod_gizmos		:: { * };

use bevy::render::mesh::shape as render_shape;
use std::f32::consts	:: { * };
use std::io::{self, Write};

use super				:: { * };
use super				:: { utils :: * };
use crate				:: { bevy_spline };

pub fn brick_road(
	transform			: &Transform,
	config_in			: &Herringbone2Config,
	debug				: bool,

	polylines			: &mut ResMut<Assets<Polyline>>,
	polyline_materials 	: &mut ResMut<Assets<PolylineMaterial>>,

	mut sargs			: &mut SpawnArguments,
) -> Entity {
	let mut config = config_in.clone();

	let root_e = bevy_spline::spawn::new(
		transform,
		config.length,
		120.0,
		Color::rgb(0.2, 0.2, 0.2),
		polylines,
		polyline_materials,
		sargs
	);

	config.root_entity	= root_e;

	let tile_size		= config.hsize * 2.0;

	config.mesh = sargs.meshes.add(
	Mesh::from(
		render_shape::Box::new(
			tile_size.x, tile_size.y, tile_size.z
		)
	));

	config.material	=
	sargs.materials.add(
	StandardMaterial { 
		base_color : Color::ALICE_BLUE,
		..default()
	});

	config.material_dbg	=
	sargs.materials.add(
	StandardMaterial { 
		base_color : Color::PINK,
		..default()
	});

	sargs.commands.entity(root_e)
		.insert			(RoadWidth::W(config.width))
		.insert			(config)
		.insert			(HerringboneControl::default())
		.insert			(BrickRoadProgressState::default())
		;

	root_e
}

pub fn brick_road_iter(
		spline 			: &Spline,
	mut state			: &mut BrickRoadProgressState,
		config			: &Herringbone2Config,
		_ass			: &Res<AssetServer>,
		control			: &HerringboneControl,
		sargs			: &mut SpawnArguments,
) {
	// a little logging helper lambda
	let iter 			= state.iter;
	let mut log_holder	= LogHolder::default();
	let mut log 		= |str_in : String| {
		let str_final 	= format!("[{}] {}\n", iter, str_in);
		if control.verbose {
			let _res =
			io::stdout().write(str_final.as_bytes());
		}
		log_holder.data.push_str(str_final.as_str());
	};

	//
	//
	// Calculating new/next tile position that fits on tile

	// on a straight line tile position's z works as "t" (parameter for spline sampling). Later on t gets adjusted in find_t_on_spline to road limits for current tile
	// z is used explicitely here because we don't want to deal with 2 dimensions in spline sampling and offset by x will be added later
	log(format!("new brick_road_iter! t: {:.3}", state.t));

	let mut next_pos	= Vec3::ZERO;
	if !calc_next_tile_pos_on_road(&mut next_pos, state, spline, config, &mut log) {
		state.finished = true;
		return;
	}
	let prev_pos		= state.pos;
	let tile_pos_delta 	= next_pos - prev_pos;

	//
	//
	// Find closes point on spline to figure out borders

	let t = find_t_on_spline(next_pos, state.pos, state.t, spline, &mut log);

	log(format!("t after spline fitting: {:.3}", t));
	log(format!("t: {:.3} next_pos:[{:.3} {:.3} {:.3}] prev_pos: [{:.3} {:.3} {:.3}] tile_pos_delta: [{:.3} {:.3} {:.3}]", t, next_pos.x, next_pos.y, next_pos.z, state.pos.x, state.pos.y, state.pos.z, tile_pos_delta.x, tile_pos_delta.y, tile_pos_delta.z));

	// in herringbone pattern every next tile is rotated +-45 degrees from 0
	let pattern_angle	= herringbone_angle(state.pattern_iter);
	let pattern_rotation = Quat::from_rotation_y(pattern_angle);

	// 
	//
	// Final pose
	let mut tile_pose 	= Transform::identity();
	tile_pose.translation = next_pos;
	tile_pose.rotation	= pattern_rotation;

	// 
	//
	// Filtering + Spawning

	// calculating info for filtering
	let spline_p 	= spline.calc_position(t);
	let spline_r	= spline.calc_rotation_wpos(t, spline_p);

	let hwidth_rotated = spline_r.mul_vec3(Vec3::X * calc_max_distance_to_spline(config));

	let tile2spline	= tile_pose.translation - spline_p;
	let tile_pos_rotated =
	if tile2spline.length() > 0.001 {
		let tile_azimuth	= Quat::from_rotation_arc(Vec3::Z, tile2spline.normalize());
		spline_p + (tile_azimuth).mul_vec3(Vec3::Z * tile2spline.length())
	} else {
		tile_pose.translation
	};

	let filter_info	= Herringbone2TileFilterInfo {
		pos						: tile_pos_rotated,
		t						: t,
		left_border				: spline_p - hwidth_rotated,
		right_border			: spline_p + hwidth_rotated,
		road_halfwidth_rotated	: hwidth_rotated,
		spline_p				: spline_p
	};

	// now we got to spawning
	let tile_entity_id =
	if !control.dry_run {
		let fi		= &filter_info;
		let left	= fi.left_border;
		let right	= fi.right_border;
		let in_range = ((left.x <= fi.pos.x) && (fi.pos.x <= right.x)) || ((right.x <= fi.pos.x) && (fi.pos.x <= left.x))
						|| ((left.z <= fi.pos.z) && (fi.pos.z <= right.z)) || ((right.z <= fi.pos.z) && (fi.pos.z <= left.z));

		spawn_tile(tile_pose, !in_range, state, config, control, sargs, &mut log)
	} else {
		Entity::from_raw(0)
	};

	//
	//
	// cheat/debug: end on certain column/row id to avoid long logs etc
	let debug			= false;
	if state.iter == 12 && debug {
		log(format!("DEBUG FULL STOP"));
		state.finished = true;
		return;
	}

	//
	//
	// Iteration ended
	state.iter			+= 1;
	state.t		 		= t;
	state.pos	 		= next_pos;

	log(format!("----------------------------"));

	// id == 0 means no tile was spawned
	if tile_entity_id.id() != 0 {
		sargs.commands.entity(tile_entity_id)
		.insert(filter_info)
		.insert(log_holder);
	}
}

fn herringbone_angle(pattern_iter : usize) -> f32 {
	if pattern_iter % 2 == 0 {
		-FRAC_PI_4
	} else {
		FRAC_PI_4
	}
}

fn spawn_tile(
	pose	: Transform,
	filtered_out : bool,
	state	: &mut BrickRoadProgressState,
	config	: &Herringbone2Config,
	control	: &HerringboneControl,
	sargs	: &mut SpawnArguments,
	mut log	: impl FnMut(String)
) -> Entity {
    let (me, mut ma) = (config.mesh.clone_weak(), config.material.clone_weak());

	if filtered_out {
		if control.visual_debug {
			ma = config.material_dbg.clone_weak();
			log(format!("tile was filtered out!"));
		} else {
			return Entity::from_raw(0);
		}
	}

    let tile_to_spawn = PbrBundle{ mesh: me, material: ma, ..default() };
	let mut tile_entity_id	= Entity::from_raw(0);
    sargs.commands.entity(config.root_entity).with_children(|road_root| {
		tile_entity_id = road_root.spawn_bundle(tile_to_spawn)
			.insert			(config.body_type)
			.insert			(pose)
			.insert			(GlobalTransform::default())
			.insert			(Collider::cuboid(config.hsize.x, config.hsize.y, config.hsize.z))
			.insert_bundle	(PickableBundle::default())
//			.insert			(Draggable::default())
			.insert			(Herringbone2)
			.insert			(Tile)
			.insert			(state.clone())
			.insert			(config.clone())
			.id				()
			;
	});

	tile_entity_id
}