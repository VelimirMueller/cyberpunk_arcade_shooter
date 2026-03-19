use bevy::prelude::*;
use crate::app::GameEntity;
use crate::core::boss::components::*;
use crate::core::boss::attacks;
use crate::core::player::components::Player;
use crate::app::ScreenShake;
use crate::systems::audio::{SoundEvent, SoundEffect};

pub fn score_multiplier(round: u32) -> f32 {
    match round {
        1 => 1.0,
        2 => 1.5,
        3 => 2.0,
        4 => 2.5,
        5 => 3.0,
        _ => 3.0,
    }
}

#[derive(Component)]
pub struct PhaseShiftText {
    pub timer: Timer,
}

#[derive(Component)]
pub struct PhaseFlashEffect {
    pub timer: Timer,
}

pub fn boss_type_for_round(round: u32) -> BossType {
    match round {
        1 => BossType::GridPhantom,
        2 => BossType::NeonSentinel,
        3 => BossType::ChromeBerserker,
        4 => BossType::VoidWeaver,
        5 => BossType::ApexProtocol,
        _ => BossType::ApexProtocol,
    }
}

fn boss_config(boss_type: BossType) -> (u32, TransitionStyle, Color, f32) {
    match boss_type {
        BossType::GridPhantom => (150, TransitionStyle::Stagger, Color::srgb(0.0, 8.0, 8.0), 1.0),
        BossType::NeonSentinel => (200, TransitionStyle::Stagger, Color::srgb(8.0, 0.0, 8.0), 1.2),
        BossType::ChromeBerserker => (250, TransitionStyle::RageBurst, Color::srgb(8.0, 4.0, 0.0), 1.4),
        BossType::VoidWeaver => (300, TransitionStyle::Stagger, Color::srgb(4.0, 0.0, 8.0), 1.1),
        BossType::ApexProtocol => (400, TransitionStyle::RageBurst, Color::srgb(8.0, 8.0, 8.0), 1.6),
    }
}

pub fn spawn_boss(commands: &mut Commands, round: u32) {
    let boss_type = boss_type_for_round(round);
    let (max_hp, transition_style, color, size_mult) = boss_config(boss_type);
    let base_size = 50.0;
    let size = base_size * size_mult;

    let primary_timer = match boss_type {
        BossType::GridPhantom => Timer::from_seconds(3.0, TimerMode::Repeating),
        BossType::NeonSentinel => Timer::from_seconds(4.0, TimerMode::Repeating),
        BossType::ChromeBerserker => Timer::from_seconds(2.8, TimerMode::Repeating),
        BossType::VoidWeaver => Timer::from_seconds(5.0, TimerMode::Repeating),
        BossType::ApexProtocol => Timer::from_seconds(3.0, TimerMode::Repeating),
    };

    // Scale attack speed by round: 0.9^(round-1) makes bosses attack faster each round
    let speed_scale = 0.9_f32.powi((round as i32) - 1);
    let scaled_duration = primary_timer.duration().as_secs_f32() * speed_scale;
    let primary_timer = Timer::from_seconds(scaled_duration, TimerMode::Repeating);

    commands.spawn((
        Sprite { color, custom_size: Some(Vec2::new(size, size)), ..default() },
        Transform::from_xyz(0.0, 150.0, 0.0),
        Boss {
            boss_type,
            phase: BossPhase::Phase1,
            current_hp: max_hp,
            max_hp,
            phase_thresholds: (0.50, 0.20),
            transition_style,
            primary_timer,
            secondary_timer: None,
            attack_state: AttackState::Idle,
            combo_count: 0,
            max_combo: 1,
            cycle_index: 0,
            is_invulnerable: false,
        },
        GameEntity,
    ));
}

pub fn boss_phase_system(
    mut commands: Commands,
    mut boss_query: Query<(&mut Boss, &Transform)>,
    mut screen_shake: ResMut<ScreenShake>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    for (mut boss, boss_transform) in boss_query.iter_mut() {
        let hp_pct = boss.current_hp as f32 / boss.max_hp as f32;
        let (threshold_2, threshold_3) = boss.phase_thresholds;

        let new_phase = if hp_pct <= threshold_3 {
            BossPhase::Phase3
        } else if hp_pct <= threshold_2 {
            BossPhase::Phase2
        } else {
            BossPhase::Phase1
        };

        if new_phase != boss.phase {
            boss.phase = new_phase;
            match boss.transition_style {
                TransitionStyle::Stagger => {
                    boss.attack_state = AttackState::Recovery(
                        Timer::from_seconds(1.5, TimerMode::Once)
                    );
                    sound_events.write(SoundEvent(SoundEffect::PhaseShift));

                    // Spawn "PHASE SHIFT" text centered on screen
                    commands.spawn((
                        Text::new("PHASE SHIFT"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgba(1.0, 0.2, 0.2, 1.0)),
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Percent(50.0),
                            top: Val::Percent(40.0),
                            ..default()
                        },
                        PhaseShiftText {
                            timer: Timer::from_seconds(1.0, TimerMode::Once),
                        },
                        GameEntity,
                    ));
                },
                TransitionStyle::RageBurst => {
                    screen_shake.intensity = 1.5;
                    screen_shake.duration = 0.5;
                    screen_shake.timer = 0.5;
                    sound_events.write(SoundEvent(SoundEffect::RageBurst));

                    // Spawn a white flash sprite at boss position
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(8.0, 8.0, 8.0, 0.9),
                            custom_size: Some(Vec2::new(20.0, 20.0)),
                            ..default()
                        },
                        Transform::from_translation(boss_transform.translation),
                        PhaseFlashEffect {
                            timer: Timer::from_seconds(0.4, TimerMode::Once),
                        },
                        GameEntity,
                    ));
                },
            }
            if boss.boss_type == BossType::ChromeBerserker {
                boss.max_combo = match new_phase {
                    BossPhase::Phase1 => 1,
                    BossPhase::Phase2 => 3,
                    BossPhase::Phase3 => 3,
                };
            }
        }
    }
}

pub fn boss_idle_movement(
    time: Res<Time>,
    mut boss_query: Query<(&Boss, &mut Transform)>,
) {
    for (boss, mut transform) in boss_query.iter_mut() {
        // Sentinel and Weaver handle their own positioning
        if boss.boss_type == BossType::NeonSentinel || boss.boss_type == BossType::VoidWeaver {
            // Sentinel stays stationary (position set in attack fn)
            // Weaver teleports (position set in attack fn)
            // Still allow charging/dashing movement if somehow in that state
            if let AttackState::Dashing { target, speed } | AttackState::Charging { target, speed } = &boss.attack_state {
                let direction = (*target - transform.translation.truncate()).normalize_or_zero();
                transform.translation += (direction * *speed * time.delta_secs()).extend(0.0);
            }
            continue;
        }

        match &boss.attack_state {
            AttackState::Idle => {
                let t = time.elapsed_secs();
                transform.translation.y = 150.0 + (t * 1.5).sin() * 30.0;
            }
            AttackState::Dashing { target, speed } | AttackState::Charging { target, speed } => {
                let direction = (*target - transform.translation.truncate()).normalize_or_zero();
                transform.translation += (direction * *speed * time.delta_secs()).extend(0.0);
            }
            _ => {}
        }
    }
}

pub fn boss_attack_system(
    time: Res<Time>,
    mut commands: Commands,
    mut boss_query: Query<(&mut Boss, &mut Transform)>,
    player_query: Query<&Transform, (With<Player>, Without<Boss>)>,
    mut screen_shake: ResMut<ScreenShake>,
    hazard_query: Query<&HazardZone>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    let Ok(player_transform) = player_query.single() else { return };
    let hazard_count = hazard_query.iter().count();
    for (mut boss, mut boss_transform) in boss_query.iter_mut() {
        let delta = time.delta_secs();
        match boss.boss_type {
            BossType::GridPhantom => {
                attacks::phantom_attack(&mut boss, &boss_transform, player_transform, &mut commands, delta, &mut sound_events);
            }
            BossType::NeonSentinel => {
                attacks::sentinel_attack(&mut boss, &mut boss_transform, player_transform, &mut commands, delta, &mut sound_events);
            }
            BossType::ChromeBerserker => {
                attacks::berserker_attack(&mut boss, &boss_transform, player_transform, &mut commands, delta, &mut screen_shake, &mut sound_events);
            }
            BossType::VoidWeaver => {
                attacks::weaver_attack(&mut boss, &mut boss_transform, player_transform, &mut commands, delta, hazard_count, &mut sound_events);
            }
            BossType::ApexProtocol => {
                attacks::apex_attack(&mut boss, &mut boss_transform, player_transform, &mut commands, delta, &mut screen_shake, hazard_count, &mut sound_events);
            }
        }
    }
}

pub fn hazard_lifetime_system(
    time: Res<Time>,
    mut commands: Commands,
    mut trail_query: Query<(Entity, &mut DashTrail, &mut Sprite)>,
    mut telegraph_query: Query<(Entity, &mut ChargeTelegraph)>,
) {
    for (entity, mut trail, mut sprite) in trail_query.iter_mut() {
        trail.lifetime.tick(time.delta());
        let alpha = 1.0 - trail.lifetime.fraction();
        sprite.color = Color::srgba(0.0, 8.0, 8.0, 0.8 * alpha);
        if trail.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }

    for (entity, mut telegraph) in telegraph_query.iter_mut() {
        telegraph.lifetime.tick(time.delta());
        if telegraph.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn boss_projectile_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut BossProjectile, &mut Transform)>,
    player_query: Query<&Transform, (With<Player>, Without<BossProjectile>)>,
) {
    let player_pos = player_query.single().map(|t| t.translation.truncate()).ok();

    for (entity, mut projectile, mut transform) in query.iter_mut() {
        // Slow homing: steer toward player
        if let Some(player_pos) = player_pos {
            let current_pos = transform.translation.truncate();
            let desired_dir = (player_pos - current_pos).normalize_or_zero();
            let current_dir = projectile.velocity.normalize_or_zero();
            let new_dir = (current_dir + desired_dir * 0.02).normalize_or_zero();
            let speed = projectile.velocity.length();
            projectile.velocity = new_dir * speed;
        }

        transform.translation += (projectile.velocity * time.delta_secs()).extend(0.0);

        // Despawn if off screen
        let pos = transform.translation;
        if pos.x.abs() > 700.0 || pos.y.abs() > 400.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub fn hazard_zone_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut HazardZone, &mut Transform, &mut Sprite)>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    for (entity, mut hazard, mut transform, mut sprite) in query.iter_mut() {
        hazard.lifetime.tick(time.delta());

        // Drift toward player if velocity set
        if let Some(drift) = hazard.drift_velocity {
            transform.translation += (drift * time.delta_secs()).extend(0.0);
        }

        // Handle explosion timer
        if hazard.explodes {
            if let Some(ref mut exp_timer) = hazard.explosion_timer {
                exp_timer.tick(time.delta());
                if exp_timer.just_finished() {
                    sound_events.write(SoundEvent(SoundEffect::HazardExplode));
                    // Explode: expand to 120x120 magenta, set radius to 60, short lifetime
                    sprite.custom_size = Some(Vec2::new(120.0, 120.0));
                    sprite.color = Color::srgba(8.0, 0.0, 8.0, 0.8);
                    hazard.radius = 60.0;
                    hazard.lifetime = Timer::from_seconds(0.3, TimerMode::Once);
                    hazard.explodes = false; // Don't re-explode
                    hazard.explosion_timer = None;
                }
            }
        }

        // Fade alpha over lifetime
        let alpha = 1.0 - hazard.lifetime.fraction();
        let base_alpha = if hazard.radius >= 60.0 { 0.8 } else { 0.3 };
        let current = sprite.color.to_srgba();
        sprite.color = Color::srgba(current.red, current.green, current.blue, base_alpha * (1.0 - alpha));

        // Despawn when lifetime done
        if hazard.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn phase_shift_text_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PhaseShiftText, &mut TextColor)>,
) {
    for (entity, mut text, mut color) in query.iter_mut() {
        text.timer.tick(time.delta());
        let alpha = 1.0 - text.timer.fraction();
        color.0 = Color::srgba(1.0, 0.2, 0.2, alpha);
        if text.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn phase_flash_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PhaseFlashEffect, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut flash, mut transform, mut sprite) in query.iter_mut() {
        flash.timer.tick(time.delta());
        let progress = flash.timer.fraction();
        // Expand rapidly
        let scale = 1.0 + progress * 15.0;
        transform.scale = Vec3::splat(scale);
        // Fade out
        let alpha = (1.0 - progress) * 0.9;
        sprite.color = Color::srgba(8.0, 8.0, 8.0, alpha);
        if flash.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub fn boss_visual_system(
    time: Res<Time>,
    mut boss_query: Query<(&Boss, &mut Sprite)>,
) {
    let t = time.elapsed_secs();

    for (boss, mut sprite) in boss_query.iter_mut() {
        let (pulse_alpha, color_mult) = match boss.phase {
            BossPhase::Phase1 => (1.0_f32, 1.0_f32),
            BossPhase::Phase2 => {
                // Slow sine pulse, period ~1s
                let pulse = 0.7 + 0.3 * (t * std::f32::consts::TAU).sin();
                (pulse, 1.3)
            }
            BossPhase::Phase3 => {
                // Rapid pulse, period ~0.3s
                let pulse = 0.6 + 0.4 * (t * std::f32::consts::TAU / 0.3).sin();
                (pulse, 1.6)
            }
        };

        let base = sprite.color.to_srgba();
        sprite.color = Color::srgba(
            base.red * color_mult,
            base.green * color_mult,
            base.blue * color_mult,
            pulse_alpha,
        );
    }
}
