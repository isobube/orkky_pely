use std::f32::consts::TAU;
use std::sync::mpsc;

use bevy::ecs::system::EntityCommands;
use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy_fps_controller::controller::FpsController;
use bevy_fps_controller::controller::FpsControllerInput;
use bevy_fps_controller::controller::LogicalPlayer;
use bevy_fps_controller::controller::RenderPlayer;
use bevy_rapier3d::prelude::*;

use crate::map::MapEntityPhysics;
use crate::map::MapLight;
use crate::map::MapEntityCollider;
use crate::map::MapChange;
use crate::map::MapEntity;
use crate::map::MapShape;
use crate::map::MapTemplate;
use crate::map_loader::MapChangesReceiver;
use crate::types::AddCollidingMesh;
use crate::types::AssetPacks;
use crate::types::GameEntity;
use crate::types::GltfRegister;
use crate::types::MapTemplates;
use crate::types::NeedsAsset;
use crate::types::NeedsTemplate;
use crate::types::PlayerIds;
use crate::types::UnloadedGltfAsset;
use crate::types::You;

fn handle_map_template(
	entity_commands: &mut EntityCommands,
	template: &MapTemplate,
	entity: &MapEntity,
	game_entity: &mut GameEntity,
) {
	game_entity.asset = template.asset.clone();

	match &template.asset {
		Some(asset) => {
			entity_commands.insert(NeedsAsset {
				asset: asset.clone(),
				add_colliding_mesh: template.automatic_collision_mesh.unwrap_or_default(),
				initial_transform: template.initial_transform.clone(),
				initial_rotation_y: template.initial_rotation_y.clone(),
			});

			// entity_commands.insert(StartAnimation {
			// 	asset: asset.clone(),
			// 	animation: "idle".to_string(),
			// });
		},
		None => {}
	}

	game_entity.iddle_animation = template.iddle_animation.clone();
	game_entity.walk_animation = template.walk_animation.clone();
	game_entity.run_animation = template.run_animation.clone();
	game_entity.reload_animation = template.reload_animation.clone();
	game_entity.shoot_animation = template.shoot_animation.clone();

	game_entity.weapons = template.weapons.clone();
	
	match &entity.camera {
		Some(camera_type) => {
			entity_commands.with_children(|parent| {
				let translation = if camera_type == "fps" {
					if let Some(translation) = template.fps_camera_location {
						Vec3::from_slice(&translation)
					} else {
						Vec3::default()
					}
				} else if camera_type == "third_person" {
					if let Some(translation) = template.third_person_camera_location {
						Vec3::from(translation)
					} else {
						Vec3::default()
					}
				} else {
					Vec3::default()
				};

				parent.spawn(
					Camera3dBundle {
						transform: Transform {
							translation: translation,
							..Default::default()
						},
						..Default::default()
					}
				);
			});
		},
		_ => {}
	}

	// Collider::cuboid(hx, hy, hz)

	match &template.collider {
		Some(collider) => {
			match collider {
				MapEntityCollider::AABB => {

				},
				MapEntityCollider::Capsule { a, b, radius } => {
					log::info!("spawning capsule collider: {:?} {:?} {:?}", a, b, radius);

					entity_commands.insert((
						RigidBody::Dynamic,
						Collider::capsule(Vec3::Y * *a, Vec3::Y * *b, *radius)
					));
				},
				MapEntityCollider::Cuboid { x, y, z } => {
					log::info!("spawning cuboid collider: {:?} {:?} {:?}", x, y, z);

					let half_x = *x / 2.0;
					let half_y = *y / 2.0;
					let half_z = *z / 2.0;

					entity_commands.insert((
						Restitution::coefficient(0.0),
						Collider::cuboid(*x, *y, *z)
					));
				},
				_ => {}
			}
		},
		None => {},
	}

	match &template.physics {
		Some(physics) => {
			match physics {
				MapEntityPhysics::Dynamic => {
					entity_commands.insert(RigidBody::Dynamic);
				},
				MapEntityPhysics::Static => {
					entity_commands.insert(RigidBody::Fixed);
				}
			}
		},
		None => {}
	}

	// if &template.automatic_collision_mesh {
	// 	commands.insert(
	// 		AddCollidingMesh {
	// 			glft: template.asset.clone(),
	// 		}
	// 	);
	// }

	// if let Some(mass) = template.mass {
	// 	commands.insert(AdditionalMassProperties::Mass(mass));
	// }
}

fn spaw_map_entity(
	commands: &mut Commands,
	entity: &MapEntity,
	player_ids: &mut ResMut<PlayerIds>,
) {
	log::info!("Spawning map entity: {}", entity.template);

	let mut game_entity = GameEntity {
		entity_id: entity.entity_id.clone(),
		..Default::default()
	};

	let mut new_component = commands.spawn((
		SpatialBundle {
			..Default::default()
		},
		game_entity
		// MapEntityId(entity.entity_id.clone())
	));

	let scale = match entity.scale {
		Some(scale) => Vec3::splat(scale),
		None => Vec3::splat(1.0)
	};

	let translation = match entity.initial_position {
		Some(translation) => Vec3::from_slice(&translation),
		None => Vec3::default()
	};

	let entity_transform = Transform {
		scale: scale,
		translation: translation,
		..Default::default()
	};

	new_component.insert(entity_transform.clone());

	new_component.insert(
		NeedsTemplate {
			template: entity.template.clone(),
			map_enitity: entity.clone()
		}
	);

	if let Some(true) = entity.player {
		let player_id = player_ids.provide_player_id(&entity.entity_id);

		log::info!("[{}] entity is player {}", entity.entity_id, player_id);

		new_component.insert((
			RenderPlayer(player_id),
			You
		));

		commands.spawn((
			Collider::capsule(Vec3::Y * 0.5, Vec3::Y * 1.5, 0.5),
			LogicalPlayer(player_id),
			FpsControllerInput {
				pitch: -TAU / 12.0,
				yaw: TAU * 5.0 / 8.0,
				..default()
			},
			FpsController { ..default() },
			ActiveEvents::COLLISION_EVENTS,
			Velocity::zero(),
			RigidBody::Dynamic,
			Sleeping::disabled(),
			// LockedAxes::ROTATION_LOCKED,
			AdditionalMassProperties::Mass(1.0),
			GravityScale(0.0),
			Ccd { enabled: true }, // Prevent clipping when going fast
			TransformBundle::from_transform(entity_transform),
		));
	}
}

pub fn handle_needs_template(
	mut commands: Commands,
	template_map: Res<MapTemplates>,
	mut query: Query<(Entity, &NeedsTemplate, &mut GameEntity)>,
) {
	for (entity, needs_template, mut game_entity) in query.iter_mut() {
		match template_map.templates.get(&needs_template.template) {
			Some(template) => {
				let mut entity_commands = commands.entity(entity);
				
				handle_map_template(&mut entity_commands, template, &needs_template.map_enitity, &mut game_entity);

				entity_commands.remove::<NeedsTemplate>();
			},
			None => {}
		}
	}
}

fn spawn_light(
	commands: &mut Commands,
	light: MapLight
) {
	match light {
		MapLight::Point(point) => {
			log::info!("Spawning point light: {:?}", point);

			// commands.spawn(PointLightBundle {
			// 	point_light: PointLight {
			// 		intensity: 15000.0,
			// 		shadows_enabled: true,
			// 		..default()
			// 	},
			// 	transform: Transform::from_xyz(4.0, 8.0, 4.0),
			// 	..default()
			// });

			let mut light_bundle = PointLightBundle {
				point_light: PointLight {
					color: Color::hex(point.color).unwrap(),
					..Default::default()
				},
				..Default::default()
			};

			if let Some(intensity) = point.intensity {
				light_bundle.point_light.intensity = intensity;
			}

			if let Some(range) = point.range {
				light_bundle.point_light.range = range;
			}

			if let Some(radius) = point.radius {
				light_bundle.point_light.radius = radius;
			}

			if let Some(shadows_enabled) = point.shadows_enabled {
				light_bundle.point_light.shadows_enabled = shadows_enabled;
			}

			if let Some(location) = point.location {
				light_bundle.transform = Transform::from_xyz(location[0], location[1], location[2]);
			}

			commands.spawn(light_bundle);
		}
	}
}

fn spawn_shape(
	commands: &mut Commands,
	meshes: &mut ResMut<Assets<Mesh>>,
	materials: &mut ResMut<Assets<StandardMaterial>>,
	shape: MapShape
) {
	log::info!("spawning shape: {:?}", shape);

	match shape {
		MapShape::Cube(cube) => {
			commands.spawn(
				PbrBundle {
					mesh: meshes.add(Mesh::from(shape::Cube { size: cube.size })),
					..Default::default()
				}
			);
		},
		MapShape::Plane(plane) => {
			log::info!("spawning plane {:?}", plane);
			
			let mut plane_bundle = PbrBundle {
				mesh: meshes.add(Mesh::from(shape::Plane { size: plane.size })),
				material: materials.add(
					StandardMaterial {
						base_color: Color::rgb(0.3, 0.5, 0.3),
						..Default::default()
					}
				),
				..Default::default()
			};

			// if let Some(material) = plane.material {
			// 	plane_bundle.material = materials.add(Color::hex(material).unwrap().into());
			// }

			if let Some(location) = plane.location {
				plane_bundle.transform = Transform::from_xyz(location[0], location[1], location[2]);
			}

			commands.spawn(plane_bundle);
		},
		MapShape::Quad(quad) => {
			commands.spawn(
				PbrBundle {
					mesh: meshes.add(Mesh::from(shape::Quad { 
						size: Vec2::from_slice(&quad.size),
						..Default::default() 
					})),
					..Default::default()
				}
			);
		},
		MapShape::Circle(circle) => {
			commands.spawn(
				PbrBundle {
					mesh: meshes.add(Mesh::from(shape::Circle {
						radius: circle.radius,
						vertices: match circle.vertices {
							Some(vertices) => vertices,
							None => 32
						},
						..Default::default()
					})),
					..Default::default()
				}
			);
		},
		MapShape::Box(box_shape) => {
			let mut entity_commands = commands.spawn(
				PbrBundle {
					mesh: meshes.add(Mesh::from(shape::Box {
						min_x: box_shape.min_x,
						min_y: box_shape.min_y,
						min_z: box_shape.min_z,
						max_x: box_shape.max_x,
						max_y: box_shape.max_y,
						max_z: box_shape.max_z,
						..Default::default()
					})),
					material: materials.add(StandardMaterial {
						base_color: Color::hex("E6EED6").unwrap(),
						..default()
					}),
					..Default::default()
				}
			);

			if let Some(true) = box_shape.collider {
				let hx = (box_shape.max_x - box_shape.min_x) / 2.0;
				let hy = (box_shape.max_y - box_shape.min_y) / 2.0;
				let hz = (box_shape.max_z - box_shape.min_z) / 2.0;

				log::info!("spawning box collider: {:?}", (hx, hy, hz));

				entity_commands.insert((
					Collider::cuboid(hx, hy, hz),
					RigidBody::Fixed
				));
			}
		},
	}
}

pub fn handle_map_changes(
	mut commands: Commands,
	chnages_receiver: Res<MapChangesReceiver>,
	mut map_templates: ResMut<MapTemplates>, 
	mut gltf_register: ResMut<GltfRegister>,
	mut done: Local<bool>,
	asset_server: Res<AssetServer>,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
	mut player_ids: ResMut<PlayerIds>,
) {
	if *done {
		return;
	}

	let chnages_receiver = chnages_receiver.rx.lock().unwrap();

	loop {
		match chnages_receiver.try_recv() {
			Ok(change) => {
				log::info!("mapchange {:?}", change);

				match change {
					MapChange::NewMapEntity(entity) => {
						spaw_map_entity(
							&mut commands, 
							&entity,
							&mut player_ids
						);
					},
        			MapChange::NewMapTemplate(template) => {
						match &template.asset {
							Some(asset_path) => {
								let asset: Handle<Gltf> = asset_server.load(asset_path);

								let unloaded_asset = UnloadedGltfAsset {
									asset: asset_path.clone(),
									gltf: asset
								};

								gltf_register.unloaded.push(unloaded_asset);
							},
							None => todo!(),
						}

						map_templates.templates.insert(template.name.clone(), template);
					},
					MapChange::NewMapShape(shape) => {
						spawn_shape(&mut commands, &mut meshes, &mut materials, shape);
					},
					MapChange::NewLight(ligth) => {
						spawn_light(&mut commands, ligth);
					},
					MapChange::NewAmbientLight(args) => {
						commands.insert_resource(AmbientLight {
							brightness: args.brightness,
							color: Color::hex(args.color).unwrap(),
						});
					},
        			MapChange::NewCameraEntity(id) => {
						// commands.spawn(
						// 	Camera3dBundle {
						// 		transform: Transform::from_xyz(0.0, 0.0, 0.0),
						// 		..Default::default()
						// 	}
						// );
					},					
				}
			},
			Err(err) => {
				match err {
					mpsc::TryRecvError::Empty => {
						break;
					},
					mpsc::TryRecvError::Disconnected => {
						log::info!("changes disconnected");
	
						*done = true;
	
						return;
					},
				}
			}
		};
	}
}

pub fn give_assets(
	mut commands: Commands,
	query: Query<(Entity, &GameEntity, &NeedsAsset)>,
	asset_packs: Res<AssetPacks>
) {
	for (entity, game_entity, needs_asset) in query.iter() {
		match asset_packs.asset_packs.get(&needs_asset.asset) {
			Some(asset_pack) => {
				log::info!("[{}] giving asset: {:?}", game_entity.entity_id, needs_asset.asset);

				let mut entity_commands = commands.entity(entity);
				entity_commands.remove::<NeedsAsset>();

				let scene = if asset_pack.scenes.len() > 0 {
					asset_pack.scenes[0].clone()
				} else {
					log::info!("[{}] no scene found", game_entity.entity_id);
					
					continue;
				};	

				if needs_asset.add_colliding_mesh {
					log::info!("[{}] adding collision mesh", game_entity.entity_id);

					entity_commands.insert(
						AddCollidingMesh {
							glft: asset_pack.gltf.clone(),
						}
					);
				}

				entity_commands.with_children(|parent| {
					log::info!("[{}] assign scene", game_entity.entity_id);

					let mut bundle = SceneBundle {
						scene: scene,
						..Default::default()
					};

					if let Some(transform) = needs_asset.initial_transform {
						log::info!("[{}] initial transform {:?}", game_entity.entity_id, transform);

						bundle.transform.translation = Vec3::new(transform[0], transform[1], transform[2]);
					}

					if let Some(rotation) = needs_asset.initial_rotation_y {
						log::info!("[{}] initial rotation {:?}", game_entity.entity_id, rotation);

						bundle.transform.rotation = Quat::from_rotation_y(
							rotation.to_radians()
						);
					}

					parent.spawn(bundle);
				});
			},
			None => {
				log::info!("[{}] no asset {:?}", game_entity.entity_id, needs_asset.asset);
			}
		}
	}
}
