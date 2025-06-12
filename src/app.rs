use bevy::prelude::*;
use bevy::core_pipeline::core_2d::Camera2d;
use crate::core::player::systems::*;
use crate::core::player::components::Player;
use crate::core::world::barriers::systems::spawn_barriers;
use crate::core::enemies::systems::{create_enemies, enemy_rotation};
use crate::systems::collision::detect_collisions;
use bevy::core_pipeline::{bloom::{Bloom}, tonemapping::{DebandDither, Tonemapping}};

#[derive(Component)]
struct AnimatedText;

pub(crate) fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (setup, spawn_player, spawn_barriers, create_enemies))
        .add_systems(Update, (player_movement, enemy_rotation, detect_collisions, update_health_ui))
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

    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("Player HP: "),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            font_size: 17.0,
            ..default()
        },
        TextShadow::default(),
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(15.0),
            left: Val::Px(5.0),
            ..default()
        },
        AnimatedText,
    ))
        .with_child((
            TextSpan::from("100"),
            TextFont {
                font_size: 17.0,
                ..default()
            },
            TextColor(Color::WHITE),
            AnimatedText,
        ));;
}

pub fn update_health_ui(
    player_query: Query<&Player>,
    mut span_query: Query<&mut TextSpan, With<AnimatedText>>,
) {
    if let Some(player) = player_query.iter().next() {
        for mut span in &mut span_query {
            **span = format!("{} %", player.max);
        }
    }
}