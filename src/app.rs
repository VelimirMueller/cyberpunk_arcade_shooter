use bevy::prelude::*;
use bevy::core_pipeline::core_2d::Camera2d;
use crate::core::player::systems::*;
use crate::core::player::components::Player;
use crate::core::world::barriers::systems::spawn_barriers;
use crate::core::enemies::systems::{create_enemies, enemy_movement_system, enemy_rotation};
use crate::systems::collision::detect_collisions;
use bevy::core_pipeline::{bloom::{Bloom}, tonemapping::{DebandDither, Tonemapping}};
use crate::core::enemies::components::Enemy;
use crate::systems::combat::{particle_movement_system, particle_cleanup_system, boss_shoot_system, player_shoot_system, player_particle_movement_system};
use crate::systems::game_over::{game_won_system, game_over_system, restart_listener, despawn_game_over_text};
use crate::data::game_state::GameState;

#[derive(Component)]
pub struct AnimatedText;


#[derive(Component)]
pub struct EnergyText;

#[derive(Component)]
pub struct GameEntity;



pub(crate) fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_state::<GameState>()
        .add_systems(Startup, setup)
        .add_systems(Startup, (spawn_player, spawn_barriers, create_enemies))
        .add_systems(Update, (despawn_game_over_text, player_movement, enemy_movement_system, enemy_rotation, detect_collisions, update_health_ui, update_enemy_health_ui, particle_movement_system, particle_cleanup_system, boss_shoot_system, player_shoot_system,player_particle_movement_system, update_energy_ui).run_if(in_state(GameState::Playing)))
        .add_systems(Update, (game_over_system, restart_listener).run_if(in_state(GameState::GameOver)))
        .add_systems(Update, (game_won_system, restart_listener).run_if(in_state(GameState::Won)))
        .run();
}

#[derive(Component)]
struct EnemyHpText;
fn setup(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
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
            left: Val::Px(10.0),
            ..default()
        },
        AnimatedText,
    ))
        .with_child((
            TextSpan::from("\n press [Space] to restart."),
            TextFont {
                font_size: 17.0,
                ..default()
            },
            TextColor(Color::WHITE),
            AnimatedText,
        ));

    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("Player Energy: "),
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
            top: Val::Px(35.0),
            left: Val::Px(10.0),
            ..default()
        },
        EnergyText,
    ))
        .with_child((
            TextSpan::from("\n press [Space] to restart."),
            TextFont {
                font_size: 17.0,
                ..default()
            },
            TextColor(Color::WHITE),
            EnergyText,
        ));

    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("Boss HP: "),
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
            right: Val::Px(5.0),
            ..default()
        },
        EnemyHpText
    ))
        .with_child((
            TextSpan::from("\n press [Space] to restart."),
            TextFont {
                font_size: 17.0,
                ..default()
            },
            TextColor(Color::WHITE),
            EnemyHpText
        ));
}

pub fn update_health_ui(
    player_query: Query<&Player>,
    mut span_query: Query<&mut TextSpan, With<AnimatedText>>,
) {
    if let Some(player) = player_query.iter().next() {
        for mut span in &mut span_query {
            **span = format!("{} %", player.current);
        }
    }
}

pub fn update_energy_ui(
    player_query: Query<&Player>,
    mut span_query: Query<&mut TextSpan, With<EnergyText>>,
) {
    if let Some(player) = player_query.iter().next() {
        for mut span in &mut span_query {
            **span = format!("{} %", player.energy);
        }
    }
}

pub fn update_enemy_health_ui(
    enemy_query: Query<&Enemy>,
    mut span_query: Query<&mut TextSpan, With<EnemyHpText>>,
    mut next_state: ResMut<NextState<GameState>>
) {
    let total_hp: u32 = enemy_query.iter().map(|enemy| enemy.current).sum();
    for mut span in &mut span_query {
        **span = format!("{} %", total_hp);

        if total_hp == 0 {
            next_state.set(GameState::Won);
        }
    }
}