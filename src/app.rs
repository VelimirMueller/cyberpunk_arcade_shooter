use bevy::prelude::*;
use bevy::core_pipeline::core_2d::Camera2d;
use crate::core::player::systems::*;
use crate::core::player::components::{Player, PlayerRotationTracker};
use crate::core::world::barriers::systems::spawn_barriers;
use crate::core::enemies::systems::{create_enemies, enemy_movement_system, enemy_rotation};
use crate::systems::collision::detect_collisions;
use bevy::core_pipeline::{bloom::{Bloom}, tonemapping::{DebandDither, Tonemapping}};
use crate::core::enemies::components::Enemy;
use crate::systems::combat::{particle_movement_system, particle_cleanup_system, boss_shoot_system, player_shoot_system, player_particle_movement_system};
use crate::systems::game_over::{game_won_system, game_over_system, restart_listener, despawn_game_over_text};
use crate::data::game_state::GameState;
use crate::systems::audio::toggle_sound;

#[derive(Resource)]
pub struct GameData {
    pub score: u32,
    pub wave: u32,
    pub high_score: u32,
    pub total_play_time: f32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            wave: 1,
            high_score: 0,
            total_play_time: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: f32,
    pub timer: f32,
}

impl Default for ScreenShake {
    fn default() -> Self {
        Self {
            intensity: 0.0,
            duration: 0.0,
            timer: 0.0,
        }
    }
}

#[derive(Component)]
pub struct DamageFlash {
    pub timer: f32,
    pub duration: f32,
}

#[derive(Component)]
pub struct MenuEntity;

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
        .init_resource::<GameData>()
        .init_resource::<ScreenShake>()
        .init_resource::<crate::systems::audio::AudioManager>()
        .add_systems(Startup, (setup, setup_menu, crate::systems::audio::setup_audio))
        .add_systems(Update, menu_input_system.run_if(in_state(GameState::Menu)))
        .add_systems(Update, pause_toggle_system.run_if(in_state(GameState::Playing)))
        .add_systems(Update, (despawn_game_over_text, player_movement, enemy_movement_system, enemy_rotation, detect_collisions, update_health_ui, update_enemy_health_ui, particle_movement_system, particle_cleanup_system, boss_shoot_system, player_shoot_system,player_particle_movement_system, update_energy_ui, screen_shake_system, damage_flash_system, update_game_data, update_score_ui).run_if(in_state(GameState::Playing)))
        .add_systems(Update, (game_over_system, restart_listener).run_if(in_state(GameState::GameOver)))
        .add_systems(Update, (game_won_system, restart_listener).run_if(in_state(GameState::Won)))
        .add_systems(Update, pause_menu_system.run_if(in_state(GameState::Paused)))
        .run();
}

#[derive(Component)]
struct EnemyHpText;
fn setup(mut commands: Commands, _next_state: ResMut<NextState<GameState>>) {
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

// ============ NEW GAME LOOP SYSTEMS ============

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct WaveText;

#[derive(Component)]
pub struct PauseText;

pub fn setup_menu(mut commands: Commands, game_data: Res<GameData>) {
    commands.spawn((
        Text::new("CYBERPUNK BLOOM CUBE"),
        TextFont {
            font_size: 60.0,
            ..default()
        },
        TextShadow::default(),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(22.0),
            top: Val::Percent(30.0),
            ..default()
        },
        MenuEntity,
    ));

    commands.spawn((
        Text::new("Press ENTER to Start"),
        TextFont {
            font_size: 30.0,
            ..default()
        },
        TextShadow::default(),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(28.0),
            top: Val::Percent(50.0),
            ..default()
        },
        MenuEntity,
    ));

    commands.spawn((
        Text::new(&format!("High Score: {}", game_data.high_score)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextShadow::default(),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(38.0),
            top: Val::Percent(65.0),
            ..default()
        },
        MenuEntity,
    ));
}

pub fn menu_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    menu_query: Query<Entity, With<MenuEntity>>,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        // Clear menu entities
        for entity in &menu_query {
            commands.entity(entity).despawn();
        }
        // Start game
        commands.spawn((Player { current: 100, max: 100, last_collision_time: None, energy: 100, last_shot_time: None }, PlayerRotationTracker { last_angle_index: 0 }, GameEntity, Transform::from_xyz(-250.0, 0.0, 0.0), GlobalTransform::default(), Sprite { color: Color::srgb(1.2, 2.8, 1.2), custom_size: Some(Vec2::new(50.0, 50.0)), ..default() }));
        spawn_barriers(commands.reborrow());
        create_enemies(commands.reborrow());

        // Add score UI
        commands.spawn((
            Text::new("Score: 0"),
            TextFont { font_size: 17.0, ..default() },
            TextShadow::default(),
            TextLayout::new_with_justify(JustifyText::Center),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(55.0),
                left: Val::Px(10.0),
                ..default()
            },
            ScoreText,
        ));

        commands.spawn((
            Text::new("Wave: 1"),
            TextFont { font_size: 17.0, ..default() },
            TextShadow::default(),
            TextLayout::new_with_justify(JustifyText::Center),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(75.0),
                left: Val::Px(10.0),
                ..default()
            },
            WaveText,
        ));

        next_state.set(GameState::Playing);
    }
}

pub fn pause_toggle_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::Paused);
    }
}

pub fn pause_menu_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut game_data: ResMut<GameData>,
    mut audio: ResMut<crate::systems::audio::AudioManager>,
    pause_query: Query<Entity, With<PauseText>>,
) {
    // Spawn pause menu if not exists
    if pause_query.is_empty() {
        let sound_status = if audio.sound_enabled { "ON" } else { "OFF" };
        commands.spawn((
            Text::new(format!("PAUSED\n\nPress ESC to Resume\nPress Q to Return to Menu\nPress M to Toggle Sound ({})", sound_status)),
            TextFont {
                font_size: 40.0,
                ..default()
            },
            TextShadow::default(),
            TextLayout::new_with_justify(JustifyText::Center),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(10.0),
                top: Val::Percent(30.0),
                ..default()
            },
            PauseText,
        ));
    }

    if keyboard_input.just_pressed(KeyCode::Escape) {
        // Clear pause text and resume
        for entity in &pause_query {
            commands.entity(entity).despawn();
        }
        next_state.set(GameState::Playing);
    }

    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        // Return to menu
        for entity in &pause_query {
            commands.entity(entity).despawn();
        }
        next_state.set(GameState::Menu);
    }

    if keyboard_input.just_pressed(KeyCode::KeyM) {
        // Toggle sound
        toggle_sound(&mut audio);
        // Respawn pause menu with updated sound status
        for entity in &pause_query {
            commands.entity(entity).despawn();
        }
    }
}

pub fn update_game_data(
    time: Res<Time>,
    mut game_data: ResMut<GameData>,
) {
    game_data.total_play_time += time.delta().as_secs_f32();
}

pub fn update_score_ui(
    game_data: Res<GameData>,
    mut score_query: Query<&mut Text, With<ScoreText>>,
    mut wave_query: Query<&mut Text, (With<WaveText>, Without<ScoreText>)>,
) {
    for mut text in &mut score_query {
        text.0 = format!("Score: {}", game_data.score);
    }
    for mut text in &mut wave_query {
        text.0 = format!("Wave: {}", game_data.wave);
    }
}

pub fn screen_shake_system(
    time: Res<Time>,
    mut screen_shake: ResMut<ScreenShake>,
    mut camera_query: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
) {
    if screen_shake.intensity > 0.0 {
        screen_shake.timer -= time.delta().as_secs_f32();

        if screen_shake.timer <= 0.0 {
            screen_shake.intensity = 0.0;
        } else {
            let shake_amount = screen_shake.intensity * (screen_shake.timer / screen_shake.duration);
            if let Ok(mut transform) = camera_query.single_mut() {
                transform.translation.x = (rand::random::<f32>() - 0.5) * shake_amount * 10.0;
                transform.translation.y = (rand::random::<f32>() - 0.5) * shake_amount * 10.0;
            }
        }
    } else {
        // Reset camera position
        if let Ok(mut transform) = camera_query.single_mut() {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
        }
    }
}

pub fn damage_flash_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DamageFlash, &mut Sprite)>,
) {
    for (entity, mut flash, mut sprite) in &mut query {
        flash.timer -= time.delta().as_secs_f32();

        if flash.timer <= 0.0 {
            commands.entity(entity).remove::<DamageFlash>();
            sprite.color = Color::WHITE;
        } else {
            let alpha = (flash.timer / flash.duration).min(1.0);
            sprite.color = Color::srgba(1.0, 0.0, 0.0, alpha);
        }
    }
}

pub fn trigger_screen_shake(screen_shake: &mut ScreenShake) {
    screen_shake.intensity = 15.0;
    screen_shake.duration = 0.3;
    screen_shake.timer = 0.3;
}

pub fn trigger_damage_flash(entity: Entity, mut commands: Commands) {
    commands.entity(entity).insert(DamageFlash {
        timer: 0.2,
        duration: 0.2,
    });
}