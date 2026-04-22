use crate::core::boss::systems::{
    boss_attack_system, boss_death_check_system, boss_death_system, boss_idle_movement,
    boss_phase_system, boss_projectile_system, boss_visual_system, death_explosion_system,
    desperation_ambient_shake, eliminated_text_system, hazard_lifetime_system, hazard_zone_system,
    phase_flash_system, phase_name_text_system, phase_shift_text_system, phase_transition_system,
};
use crate::core::player::components::{Player, PlayerRotationTracker};
use crate::core::player::systems::*;
use crate::core::world::barriers::systems::spawn_barriers;
use crate::data::game_state::GameState;
use crate::systems::audio::SoundEvent;
use crate::systems::background::{animate_stars, draw_background_grid, spawn_background_stars};
use crate::systems::collision::DeathEvent;
use crate::systems::collision::detect_collisions;
use crate::systems::combat::{
    particle_cleanup_system, particle_movement_system, player_particle_movement_system,
    player_shoot_system,
};
use crate::systems::game_over::restart_listener;
use crate::systems::particles::{
    AfterimageTimer, AmbientParticleTimer, animate_afterimages, animate_ambient_particles,
    animate_shatter, animate_shockwave, handle_death_events, setup_shockwave_assets,
    spawn_afterimages, spawn_ambient_particles,
};
use crate::systems::post_processing::{CrtPostProcessPlugin, CrtSettings};
use crate::systems::powerups::{
    laser_charge_orb_system, laser_charge_particle_system, laser_impact_system,
    laser_stream_particle_system, laser_system, powerup_lifetime_system, powerup_pickup_system,
    powerup_shockwave_system, powerup_spawn_system, setup_powerup_timer,
};
use crate::systems::round::{
    boss_defeated_check, despawn_round_clear, round_announce_system, score_tally_system,
    start_round_announce,
};
use crate::ui::announcement::{
    despawn_announcement_ui, spawn_announcement_ui, update_announcement_ui,
};
use crate::ui::hud::{spawn_hud, update_boss_hud, update_player_hud, update_score_hud};
use crate::ui::menus::{
    PauseEntity, despawn_game_over_screen, despawn_game_won_screen, despawn_pause_menu,
    despawn_title_menu, spawn_game_over_screen, spawn_game_won_screen, spawn_pause_menu,
    spawn_title_menu,
};
use crate::utils::config::{ENTITY_SCALE, QualityTier};
use bevy::core_pipeline::core_2d::Camera2d;
use bevy::core_pipeline::{
    bloom::Bloom,
    tonemapping::{DebandDither, Tonemapping},
};
use bevy::prelude::*;
use bevy::window::WindowPlugin;

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
pub struct AnimatedText;

#[derive(Component)]
pub struct EnergyText;

#[derive(Component)]
pub struct GameEntity;

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Cyberpunk: The Incredible Bloom Cube".to_string(),
                    canvas: Some("#game-canvas".to_string()),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: true,
                    ..default()
                }),
                ..default()
            }),
            CrtPostProcessPlugin,
        ))
        .init_state::<GameState>()
        .init_resource::<QualityTier>()
        .init_resource::<GameData>()
        .init_resource::<ScreenShake>()
        .add_event::<SoundEvent>()
        .init_resource::<AfterimageTimer>()
        .init_resource::<AmbientParticleTimer>()
        .add_event::<DeathEvent>()
        .add_systems(
            Startup,
            (setup, spawn_background_stars, setup_shockwave_assets),
        )
        .add_systems(Startup, crate::systems::audio::setup_audio)
        .add_systems(
            Update,
            (
                animate_stars,
                draw_background_grid,
                crate::systems::audio::play_sounds,
            ),
        )
        .add_systems(OnEnter(GameState::Menu), spawn_title_menu)
        .add_systems(OnExit(GameState::Menu), despawn_title_menu)
        .add_systems(Update, menu_input_system.run_if(in_state(GameState::Menu)))
        .add_systems(
            OnEnter(GameState::RoundAnnounce),
            (
                start_round_announce,
                spawn_announcement_ui,
                spawn_hud,
                setup_powerup_timer,
            ),
        )
        .add_systems(
            Update,
            (round_announce_system, update_announcement_ui)
                .run_if(in_state(GameState::RoundAnnounce)),
        )
        .add_systems(OnExit(GameState::RoundAnnounce), despawn_announcement_ui)
        .add_systems(
            Update,
            pause_toggle_system.run_if(in_state(GameState::RoundActive)),
        )
        .add_systems(
            Update,
            (
                player_movement,
                detect_collisions,
                particle_movement_system,
                particle_cleanup_system,
                boss_attack_system,
                player_shoot_system,
                player_particle_movement_system,
                screen_shake_system,
                damage_flash_system,
                update_game_data,
                boss_phase_system,
                boss_idle_movement,
                hazard_lifetime_system,
                boss_projectile_system,
                hazard_zone_system,
                update_boss_hud,
                update_player_hud,
                update_score_hud,
            )
                .run_if(in_state(GameState::RoundActive)),
        )
        .add_systems(
            Update,
            boss_death_check_system
                .after(detect_collisions)
                .run_if(in_state(GameState::RoundActive)),
        )
        .add_systems(
            Update,
            handle_death_events
                .after(detect_collisions)
                .run_if(in_state(GameState::RoundActive)),
        )
        .add_systems(
            Update,
            (animate_shatter, animate_shockwave).run_if(in_state(GameState::RoundActive)),
        )
        .add_systems(
            Update,
            (
                spawn_afterimages,
                animate_afterimages,
                spawn_ambient_particles,
                animate_ambient_particles,
                phase_shift_text_system,
                phase_flash_system,
                boss_visual_system,
                phase_transition_system,
                phase_name_text_system,
                boss_death_system,
                death_explosion_system,
                eliminated_text_system,
                desperation_ambient_shake,
            )
                .run_if(in_state(GameState::RoundActive)),
        )
        .add_systems(
            Update,
            (
                powerup_spawn_system,
                powerup_lifetime_system,
                powerup_pickup_system,
                laser_system,
                powerup_shockwave_system,
                laser_charge_particle_system,
                laser_charge_orb_system,
                laser_stream_particle_system,
                laser_impact_system,
                crate::systems::powerups::effects::phase_shift::phase_shift_tick_system,
            )
                .run_if(in_state(GameState::RoundActive)),
        )
        .add_systems(
            Update,
            crate::systems::powerups::effects::blink::blink_particle_system
                .run_if(in_state(GameState::RoundActive)),
        )
        .add_systems(
            Update,
            (boss_defeated_check, score_tally_system).run_if(in_state(GameState::RoundActive)),
        )
        .add_systems(
            OnExit(GameState::RoundActive),
            (
                despawn_round_clear,
                crate::systems::powerups::cleanup_player_buffs_on_round_exit,
            ),
        )
        .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
        .add_systems(OnExit(GameState::GameOver), despawn_game_over_screen)
        .add_systems(
            Update,
            restart_listener.run_if(in_state(GameState::GameOver)),
        )
        .add_systems(OnEnter(GameState::Won), spawn_game_won_screen)
        .add_systems(OnExit(GameState::Won), despawn_game_won_screen)
        .add_systems(Update, restart_listener.run_if(in_state(GameState::Won)))
        .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
        .add_systems(OnExit(GameState::Paused), despawn_pause_menu)
        .add_systems(
            Update,
            pause_menu_system.run_if(in_state(GameState::Paused)),
        )
        .run();
}

#[allow(dead_code)]
fn setup(
    mut commands: Commands,
    _next_state: ResMut<NextState<GameState>>,
    quality: Res<QualityTier>,
) {
    let bloom = match *quality {
        QualityTier::Desktop => Bloom::default(),
        QualityTier::Mobile => Bloom {
            intensity: 0.2,
            low_frequency_boost: 0.5,
            ..default()
        },
    };

    let crt = match *quality {
        QualityTier::Desktop => CrtSettings::default(),
        QualityTier::Mobile => CrtSettings {
            scanline_intensity: 0.10,
            scanline_count: 150.0,
            vignette_intensity: 0.3,
            vignette_radius: 0.75,
            curvature_amount: 0.0,
        },
    };

    let mut camera = commands.spawn((
        Camera2d,
        Transform::default(),
        GlobalTransform::default(),
        Camera {
            hdr: true,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        bloom,
        DebandDither::Enabled,
        crt,
    ));

    if *quality == QualityTier::Mobile {
        camera.insert(Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::render::camera::ScalingMode::AutoMin {
                min_width: 1500.0,
                min_height: 620.0,
            },
            ..OrthographicProjection::default_2d()
        }));
    }
}

// ============ NEW GAME LOOP SYSTEMS ============

#[derive(Component)]
pub struct ScoreText;

#[derive(Component)]
pub struct WaveText;

pub fn menu_input_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) {
    if keyboard_input.just_pressed(KeyCode::Enter) {
        // Start game — spawn player & barriers
        commands.spawn((
            Player {
                current: 100,
                max: 100,
                last_collision_time: None,
                energy: 100,
                max_energy: 100,
                last_shot_time: None,
            },
            PlayerRotationTracker {
                last_angle_index: 0,
            },
            GameEntity,
            Transform::from_xyz(-250.0, 0.0, 0.0),
            GlobalTransform::default(),
            Sprite {
                color: Color::srgb(1.2, 2.8, 1.2),
                custom_size: Some(Vec2::new(50.0 * ENTITY_SCALE, 50.0 * ENTITY_SCALE)),
                ..default()
            },
        ));
        spawn_barriers(commands.reborrow());

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
    mut library: ResMut<crate::systems::audio::SoundLibrary>,
    pause_query: Query<Entity, With<PauseEntity>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        next_state.set(GameState::RoundActive);
    }

    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        next_state.set(GameState::Menu);
    }

    if keyboard_input.just_pressed(KeyCode::KeyM) {
        crate::systems::audio::toggle_sound(&mut library);
        // Respawn pause menu with updated sound status
        for entity in &pause_query {
            commands.entity(entity).despawn();
        }
        let sound_status = if library.sound_enabled { "ON" } else { "OFF" };
        commands
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    row_gap: Val::Px(16.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
                PauseEntity,
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("PAUSED"),
                    TextFont {
                        font_size: 36.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.0, 1.0, 1.0)),
                ));
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(Val::Px(24.0)),
                            row_gap: Val::Px(12.0),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        BorderColor(Color::srgb(0.15, 0.15, 0.15)),
                    ))
                    .with_children(|container| {
                        let gray = Color::srgb(0.33, 0.33, 0.33);
                        container.spawn((
                            Text::new("Press ESC to Resume"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(gray),
                        ));
                        container.spawn((
                            Text::new("Press Q to Return to Menu"),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(gray),
                        ));
                        container.spawn((
                            Text::new(format!("Press M to Toggle Sound ({})", sound_status)),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(gray),
                        ));
                    });
            });
    }
}

pub fn update_game_data(time: Res<Time>, mut game_data: ResMut<GameData>) {
    game_data.total_play_time += time.delta().as_secs_f32();
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
            let shake_amount =
                screen_shake.intensity * (screen_shake.timer / screen_shake.duration);
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
