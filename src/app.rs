use bevy::prelude::*;
use bevy::core_pipeline::core_2d::Camera2d;
use crate::core::player::systems::*;
use crate::core::player::components::{Player, PlayerRotationTracker};
use crate::core::world::barriers::systems::spawn_barriers;
use crate::systems::collision::detect_collisions;
use bevy::core_pipeline::{bloom::{Bloom}, tonemapping::{DebandDither, Tonemapping}};
use crate::core::boss::components::Boss;
use crate::systems::combat::{particle_movement_system, particle_cleanup_system, player_shoot_system, player_particle_movement_system};
use crate::core::boss::systems::{boss_phase_system, boss_idle_movement, boss_attack_system, hazard_lifetime_system, boss_projectile_system};
use crate::systems::game_over::{game_won_system, game_over_system, restart_listener, despawn_game_over_text};
use crate::data::game_state::GameState;
use crate::systems::round::{start_round_announce, round_announce_system, boss_defeated_check, score_tally_system, despawn_round_clear};
use crate::ui::announcement::{spawn_announcement_ui, update_announcement_ui, despawn_announcement_ui};
use crate::systems::audio::{toggle_sound, SoundEvent};
use crate::systems::background::{spawn_background_stars, animate_stars, draw_background_grid};
use crate::systems::collision::DeathEvent;
use crate::systems::particles::{
    setup_shockwave_assets, handle_death_events,
    animate_shatter, animate_shockwave,
    AfterimageTimer, spawn_afterimages, animate_afterimages,
    AmbientParticleTimer, spawn_ambient_particles, animate_ambient_particles,
};
use crate::systems::post_processing::{CrtPostProcessPlugin, CrtSettings};

#[derive(Resource)]
pub struct GameData {
    pub score: u32,
    pub round: u32,
    pub high_score: u32,
    pub total_play_time: f32,
    pub enemies_killed: u32,
    pub total_enemies: u32,
    pub total_rounds: u32,
}

impl Default for GameData {
    fn default() -> Self {
        Self {
            score: 0,
            round: 1,
            high_score: 0,
            total_play_time: 0.0,
            enemies_killed: 0,
            total_enemies: 1,
            total_rounds: 5,
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
        .add_plugins((DefaultPlugins, CrtPostProcessPlugin))
        .init_state::<GameState>()
        .init_resource::<GameData>()
        .init_resource::<ScreenShake>()
        .add_event::<SoundEvent>()
        .init_resource::<AfterimageTimer>()
        .init_resource::<AmbientParticleTimer>()
        .add_event::<DeathEvent>()
        .add_systems(Startup, (setup, setup_menu, spawn_background_stars, setup_shockwave_assets))
        .add_systems(Startup, crate::systems::audio::setup_synth_audio)
        .add_systems(Update, (animate_stars, draw_background_grid, crate::systems::audio::play_sounds))
        .add_systems(Update, menu_input_system.run_if(in_state(GameState::Menu)))
        .add_systems(OnEnter(GameState::RoundAnnounce), (start_round_announce, spawn_announcement_ui))
        .add_systems(Update, (round_announce_system, update_announcement_ui).run_if(in_state(GameState::RoundAnnounce)))
        .add_systems(OnExit(GameState::RoundAnnounce), despawn_announcement_ui)
        .add_systems(Update, pause_toggle_system.run_if(in_state(GameState::RoundActive)))
        .add_systems(Update, (despawn_game_over_text, player_movement, detect_collisions, update_health_ui, update_enemy_health_ui, particle_movement_system, particle_cleanup_system, boss_attack_system, player_shoot_system, player_particle_movement_system, update_energy_ui, screen_shake_system, damage_flash_system, update_game_data, update_score_ui, boss_phase_system, boss_idle_movement, hazard_lifetime_system, boss_projectile_system).run_if(in_state(GameState::RoundActive)))
        .add_systems(Update, handle_death_events.after(detect_collisions).run_if(in_state(GameState::RoundActive)))
        .add_systems(Update, (animate_shatter, animate_shockwave).run_if(in_state(GameState::RoundActive)))
        .add_systems(Update, (spawn_afterimages, animate_afterimages, spawn_ambient_particles, animate_ambient_particles).run_if(in_state(GameState::RoundActive)))
        .add_systems(Update, (boss_defeated_check, score_tally_system).run_if(in_state(GameState::RoundActive)))
        .add_systems(OnExit(GameState::RoundActive), despawn_round_clear)
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
        CrtSettings::default(),
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
    boss_query: Query<&Boss>,
    mut span_query: Query<&mut TextSpan, With<EnemyHpText>>,
) {
    let total_hp: u32 = boss_query.iter().map(|boss| boss.current_hp).sum();
    for mut span in &mut span_query {
        **span = format!("{} %", total_hp);
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
            Text::new("Round: 1"),
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

        next_state.set(GameState::RoundAnnounce);
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
    _game_data: ResMut<GameData>,
    mut audio: NonSendMut<crate::systems::audio::SynthAudio>,
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
        next_state.set(GameState::RoundActive);
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
        text.0 = format!("Round: {}", game_data.round);
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
