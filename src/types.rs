use std::time::Duration;

use bevy::gltf::Gltf;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy::utils::HashMap;

use crate::map::MapEntity;
use crate::map::MapTemplate;
use crate::map::Weapon;

#[derive(Clone, Component)]
pub struct You;

#[derive(Clone, Component)]
pub struct AddCollidingMesh {
	pub glft: Handle<Gltf>,
}

#[derive(Clone, Component)]
pub struct NeedsTemplate {
	pub template: String,
	pub map_enitity: MapEntity
}

#[derive(Clone, Component)]
pub struct NeedsAsset {
	pub asset: String,
	pub add_colliding_mesh: bool,
	pub initial_transform: Option<[f32; 3]>,
	pub initial_rotation_y: Option<f32>,
}

#[derive(Clone, Component, Default)]
pub struct StartAnimation {
	pub asset: String,
	pub animation: String,
	pub repeat: bool,
}

#[derive(Clone, Component)]
pub struct StopAnimation;

#[derive(Clone, Resource, Default)]
pub struct MapTemplates {
	pub templates: HashMap<String, MapTemplate>
}

#[derive(Clone, Default)]
pub struct UnloadedGltfAsset {
	pub asset: String,
	pub gltf: Handle<Gltf>,
}

#[derive(Clone, Resource, Default)]
pub struct GltfRegister {
	pub unloaded: Vec<UnloadedGltfAsset>,
}

#[derive(Clone, Default)]
pub struct AssetPack {
	pub gltf: Handle<Gltf>,
	pub scenes: Vec<Handle<Scene>>,
	pub named_scenes: HashMap<String, Handle<Scene>>,
	pub animations: Vec<Handle<AnimationClip>>,
	pub named_animations: HashMap<String, Handle<AnimationClip>>
}

#[derive(Clone, Resource, Default)]
pub struct AssetPacks {
	pub asset_packs: HashMap<String, AssetPack>
}

#[derive(Clone, Component, Default)]
pub struct GameEntity {
	pub entity_id: String,
	pub current_weapon: usize,
	pub weapons: Vec<Weapon>,
	pub asset: Option<String>,
	pub iddle_animation: Option<String>,
	pub walk_animation: Option<String>,
	pub run_animation: Option<String>,
	pub reload_animation: Option<String>,
	pub shoot_animation: Option<String>,
}

#[derive(Clone, Resource, Default)]
pub struct EntityStore {
	pub entities: HashMap<String, GameEntity>
}

// #[derive(Clone, Component)]
// pub struct MapEntityId(pub String);


#[derive(Clone, Resource, Default)]
pub struct PlayerIds {
	player_ids: HashMap<String, u8>
}

impl PlayerIds {
	pub fn provide_player_id(&mut self, entity_id: &str) -> u8 {
		if let Some(player_id) = self.player_ids.get(entity_id) {
			return *player_id;
		}

		let player_id = self.player_ids.len() as u8;
		self.player_ids.insert(entity_id.to_string(), player_id);

		player_id
	}
}

#[derive(Clone, Component)]
pub struct StartAttack;

#[derive(Clone, Component)]
pub struct StopAttack;


#[derive(Clone, Component)]
pub struct MeleeHitbox {
	pub delay: f32,
	pub dur: f32,
	pub radius: f32,
	pub start_angle: f32,
	pub end_angle: f32,
}
