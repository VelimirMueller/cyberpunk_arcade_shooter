use bevy::color::Color;
use bevy::math::Vec2;
use bevy::prelude::{default, Sprite, Transform, Commands};
use crate::core::world::barriers::components::Barrier;
use crate::env::{GROUND_Y, CEILING_Y};

pub(crate) fn spawn_barriers(
    mut commands: Commands,
) -> () {
    // Spawn upper Barrier
    commands.spawn((Sprite {
        color: Color::srgb(7.9, 0.2, 0.3),
        custom_size: Some(Vec2::new(3000.0, 10.0)),
        ..default()
    },
    Transform::from_xyz(0.0, GROUND_Y - 35.0, 0.0),
    Barrier));

    commands.spawn((Sprite {
        color: Color::srgb(7.9, 0.2, 0.3),
        custom_size: Some(Vec2::new(3000.0, 10.0)),
        ..default()
    },
    Transform::from_xyz(0.0, CEILING_Y + 35.0, 0.0),
    Barrier));
}