use bevy::prelude::*;
use bevy::core_pipeline::core_2d::Camera2d;
use crate::core::player::systems::*;
use crate::core::world::barriers::systems::spawn_barriers;
use bevy::core_pipeline::{bloom::{Bloom}, tonemapping::{DebandDither, Tonemapping}};

pub(crate) fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, spawn_player, spawn_barriers))
        .add_systems(Update, player_movement)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform::default(),
        GlobalTransform::default(),
        Camera {
            hdr: true, // 1. HDR is required for bloom
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
        Bloom::default(),           // 3. Enable bloom for the camera
        DebandDither::Enabled,
    ));
}
