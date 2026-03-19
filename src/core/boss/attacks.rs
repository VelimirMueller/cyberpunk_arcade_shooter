// Per-boss attack patterns — implemented in Task 4-8

use bevy::prelude::*;
use crate::app::{GameEntity, ScreenShake};
use crate::core::boss::components::*;
use crate::systems::audio::{SoundEvent, SoundEffect};

/// Grid Phantom attack pattern: dash + telegraph + trail
/// State machine: Idle → WindUp → Dashing → Recovery (with optional chain dash in Phase 3)
pub fn phantom_attack(
    boss: &mut Boss,
    boss_transform: &Transform,
    player_transform: &Transform,
    commands: &mut Commands,
    delta: f32,
    sound_events: &mut EventWriter<SoundEvent>,
) {
    let boss_pos = boss_transform.translation.truncate();
    let player_pos = player_transform.translation.truncate();

    match &mut boss.attack_state {
        AttackState::Idle => {
            boss.primary_timer.tick(std::time::Duration::from_secs_f32(delta));

            if boss.primary_timer.just_finished() {
                // Start wind-up
                let windup_duration = if boss.combo_count > 0 { 0.3 } else { 1.0 };
                boss.attack_state = AttackState::WindUp(
                    Timer::from_seconds(windup_duration, TimerMode::Once),
                );
                boss.combo_count = 0;
                sound_events.write(SoundEvent(SoundEffect::DashTelegraph));

                // Spawn telegraph line from boss to player
                let direction = (player_pos - boss_pos).normalize_or_zero();
                let telegraph_end = boss_pos + direction * 900.0;

                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.0, 8.0, 8.0, 0.4),
                        custom_size: Some(Vec2::new(2.0, 900.0)),
                        ..default()
                    },
                    Transform::from_translation(boss_pos.extend(0.0))
                        .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2)),
                    ChargeTelegraph {
                        start: boss_pos,
                        end: telegraph_end,
                        lifetime: Timer::from_seconds(windup_duration, TimerMode::Once),
                    },
                    GameEntity,
                ));
            }
        }
        AttackState::WindUp(timer) => {
            timer.tick(std::time::Duration::from_secs_f32(delta));
            if timer.finished() {
                // Transition to dashing toward player's current position
                boss.attack_state = AttackState::Dashing {
                    target: player_pos,
                    speed: 800.0,
                };
            }
        }
        AttackState::Dashing { target, speed: _ } => {
            let target_pos = *target;
            let dist = boss_pos.distance(target_pos);

            if dist < 10.0 {
                // Reached target
                // Phase 2+: spawn DashTrail
                if boss.phase != BossPhase::Phase1 {
                    let trail_lifetime = match boss.phase {
                        BossPhase::Phase2 => 2.0,
                        BossPhase::Phase3 => 4.0,
                        _ => 2.0,
                    };

                    commands.spawn((
                        Sprite {
                            color: Color::srgba(0.0, 8.0, 8.0, 0.8),
                            custom_size: Some(Vec2::new(20.0, 20.0)),
                            ..default()
                        },
                        Transform::from_translation(boss_pos.extend(0.0)),
                        DashTrail {
                            lifetime: Timer::from_seconds(trail_lifetime, TimerMode::Once),
                            damage: 5,
                        },
                        GameEntity,
                    ));
                }

                // Phase 3: chain a second dash if combo_count < 1
                if boss.phase == BossPhase::Phase3 && boss.combo_count < 1 {
                    boss.combo_count += 1;
                    // Short wind-up before chain dash
                    boss.attack_state = AttackState::WindUp(
                        Timer::from_seconds(0.3, TimerMode::Once),
                    );

                    // Spawn telegraph for chain dash
                    let direction = (player_pos - boss_pos).normalize_or_zero();
                    let telegraph_end = boss_pos + direction * 900.0;

                    commands.spawn((
                        Sprite {
                            color: Color::srgba(0.0, 8.0, 8.0, 0.4),
                            custom_size: Some(Vec2::new(2.0, 900.0)),
                            ..default()
                        },
                        Transform::from_translation(boss_pos.extend(0.0))
                            .with_rotation(Quat::from_rotation_z(direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2)),
                        ChargeTelegraph {
                            start: boss_pos,
                            end: telegraph_end,
                            lifetime: Timer::from_seconds(0.3, TimerMode::Once),
                        },
                        GameEntity,
                    ));
                } else {
                    // Enter recovery
                    let recovery_time = match boss.phase {
                        BossPhase::Phase1 => 3.0,
                        BossPhase::Phase2 => 2.0,
                        BossPhase::Phase3 => 1.0,
                    };
                    boss.attack_state = AttackState::Recovery(
                        Timer::from_seconds(recovery_time, TimerMode::Once),
                    );
                    boss.combo_count = 0;

                    // Fire 1-2 slow homing projectiles during recovery
                    let num_projectiles = if boss.phase == BossPhase::Phase1 { 1 } else { 2 };
                    for i in 0..num_projectiles {
                        let angle_offset = if num_projectiles == 1 {
                            0.0
                        } else {
                            (i as f32 - 0.5) * 0.3
                        };
                        let dir = (player_pos - boss_pos).normalize_or_zero();
                        let rotated_dir = Vec2::new(
                            dir.x * angle_offset.cos() - dir.y * angle_offset.sin(),
                            dir.x * angle_offset.sin() + dir.y * angle_offset.cos(),
                        );
                        let velocity = rotated_dir * 80.0;

                        commands.spawn((
                            Sprite {
                                color: Color::srgb(0.0, 6.0, 6.0),
                                custom_size: Some(Vec2::new(8.0, 8.0)),
                                ..default()
                            },
                            Transform::from_translation(boss_pos.extend(0.0)),
                            BossProjectile {
                                velocity,
                                damage: 5,
                            },
                            GameEntity,
                        ));
                    }
                }
            }
            // Movement is handled by boss_idle_movement (Dashing branch)
        }
        AttackState::Recovery(timer) => {
            timer.tick(std::time::Duration::from_secs_f32(delta));
            if timer.finished() {
                boss.attack_state = AttackState::Idle;
            }
        }
        AttackState::Charging { .. } | AttackState::Attacking => {
            // Not used by Grid Phantom
        }
    }
}

/// Apex Protocol attack pattern: cycles through all previous boss attacks
/// Uses cycle_index to track which sub-attack type is active
pub fn apex_attack(
    boss: &mut Boss,
    boss_transform: &mut Transform,
    player_transform: &Transform,
    commands: &mut Commands,
    delta: f32,
    screen_shake: &mut ScreenShake,
    hazard_count: usize,
    sound_events: &mut EventWriter<SoundEvent>,
) {
    let cycle_len = match boss.phase {
        BossPhase::Phase1 => 2,
        BossPhase::Phase2 => 3,
        BossPhase::Phase3 => 4,
    };
    let current_attack = boss.cycle_index % cycle_len;

    // Track if boss was idle before the sub-attack runs
    let was_idle = matches!(boss.attack_state, AttackState::Idle);

    match current_attack {
        0 => phantom_attack(boss, boss_transform, player_transform, commands, delta, sound_events),
        1 => sentinel_attack(boss, boss_transform, player_transform, commands, delta, sound_events),
        2 => berserker_attack(boss, boss_transform, player_transform, commands, delta, screen_shake, sound_events),
        3 => weaver_attack(boss, boss_transform, player_transform, commands, delta, hazard_count, sound_events),
        _ => {}
    }

    // When sub-attack completes (returns to Idle), advance cycle
    if !was_idle && matches!(boss.attack_state, AttackState::Idle) {
        boss.cycle_index += 1;
    }
}

/// Neon Sentinel attack pattern: stationary rotating beam sweeps
/// Spawns beam projectile segments in lines that rotate around the boss
pub fn sentinel_attack(
    boss: &mut Boss,
    boss_transform: &mut Transform,
    _player_transform: &Transform,
    commands: &mut Commands,
    delta: f32,
    sound_events: &mut EventWriter<SoundEvent>,
) {
    let boss_pos = boss_transform.translation.truncate();

    // Determine rotation speed based on phase
    let rotation_speed = match boss.phase {
        BossPhase::Phase1 => 1.5,
        BossPhase::Phase2 => 2.5,
        BossPhase::Phase3 => 3.0,
    };

    // Rotate the boss each frame
    boss_transform.rotate_z(rotation_speed * delta);

    match &mut boss.attack_state {
        AttackState::Idle => {
            boss.primary_timer.tick(std::time::Duration::from_secs_f32(delta));

            if boss.primary_timer.just_finished() {
                sound_events.write(SoundEvent(SoundEffect::BeamSweep));
                // Determine number of beam directions based on phase
                let num_beams = match boss.phase {
                    BossPhase::Phase1 => 1,
                    BossPhase::Phase2 => 2,
                    BossPhase::Phase3 => 2,
                };

                let base_angle = boss_transform.rotation.to_euler(EulerRot::ZYX).0;
                let spread_count = if boss.phase == BossPhase::Phase3 { 3 } else { 1 };

                for beam_idx in 0..num_beams {
                    let beam_angle = base_angle + (beam_idx as f32) * std::f32::consts::PI / num_beams as f32;

                    // Phase 3: randomize direction slightly
                    let angle_jitter = if boss.phase == BossPhase::Phase3 {
                        (rand::random::<f32>() - 0.5) * 0.4
                    } else {
                        0.0
                    };

                    for spread_idx in 0..spread_count {
                        let spread_offset = if spread_count > 1 {
                            (spread_idx as f32 - 1.0) * 0.15
                        } else {
                            0.0
                        };

                        let final_angle = beam_angle + angle_jitter + spread_offset;
                        let direction = Vec2::new(final_angle.cos(), final_angle.sin());

                        // Spawn ~12 segments in a line
                        for seg in 0..12 {
                            let offset = direction * (30.0 + seg as f32 * 40.0);
                            let seg_pos = boss_pos + offset;

                            commands.spawn((
                                Sprite {
                                    color: Color::srgb(8.0, 0.0, 8.0),
                                    custom_size: Some(Vec2::new(4.0, 4.0)),
                                    ..default()
                                },
                                Transform::from_translation(seg_pos.extend(0.0)),
                                BossProjectile {
                                    velocity: direction * 200.0,
                                    damage: 5,
                                },
                                GameEntity,
                            ));
                        }
                    }
                }
            }
        }
        AttackState::Recovery(timer) => {
            timer.tick(std::time::Duration::from_secs_f32(delta));
            if timer.finished() {
                boss.attack_state = AttackState::Idle;
            }
        }
        _ => {
            // Sentinel doesn't use other states
        }
    }

    // Keep sentinel stationary at spawn position
    boss_transform.translation.x = 0.0;
    boss_transform.translation.y = 150.0;
}

/// Chrome Berserker attack pattern: charges with combos
/// State machine: Idle → WindUp → Charging → (combo or Recovery) → Idle
pub fn berserker_attack(
    boss: &mut Boss,
    boss_transform: &Transform,
    player_transform: &Transform,
    commands: &mut Commands,
    delta: f32,
    screen_shake: &mut ScreenShake,
    sound_events: &mut EventWriter<SoundEvent>,
) {
    let boss_pos = boss_transform.translation.truncate();
    let player_pos = player_transform.translation.truncate();

    match &mut boss.attack_state {
        AttackState::Idle => {
            boss.primary_timer.tick(std::time::Duration::from_secs_f32(delta));

            if boss.primary_timer.just_finished() {
                // Start wind-up
                boss.attack_state = AttackState::WindUp(
                    Timer::from_seconds(0.8, TimerMode::Once),
                );
                sound_events.write(SoundEvent(SoundEffect::ChargeWindUp));
                // Screen shake during wind-up
                screen_shake.intensity = 0.3;
                screen_shake.duration = 0.8;
                screen_shake.timer = 0.8;
            }
        }
        AttackState::WindUp(timer) => {
            timer.tick(std::time::Duration::from_secs_f32(delta));
            if timer.finished() {
                // Charge at player at 1000px/s
                boss.attack_state = AttackState::Charging {
                    target: player_pos,
                    speed: 1000.0,
                };
            }
        }
        AttackState::Charging { target, speed: _ } => {
            let target_pos = *target;
            let dist = boss_pos.distance(target_pos);

            if dist < 15.0 {
                // Arrived at target
                // Phase 3: spawn shockwave DashTrail
                if boss.phase == BossPhase::Phase3 {
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(8.0, 4.0, 0.0, 0.8),
                            custom_size: Some(Vec2::new(80.0, 80.0)),
                            ..default()
                        },
                        Transform::from_translation(boss_pos.extend(0.0)),
                        DashTrail {
                            lifetime: Timer::from_seconds(0.5, TimerMode::Once),
                            damage: 10,
                        },
                        GameEntity,
                    ));
                    screen_shake.intensity = 1.0;
                    screen_shake.duration = 0.3;
                    screen_shake.timer = 0.3;
                }

                // Combo logic: chain another charge if combo_count < max_combo
                let max_combo = match boss.phase {
                    BossPhase::Phase1 => 1,
                    BossPhase::Phase2 => 3,
                    BossPhase::Phase3 => 3,
                };

                if boss.combo_count < max_combo {
                    boss.combo_count += 1;
                    // Short wind-up before next charge
                    boss.attack_state = AttackState::WindUp(
                        Timer::from_seconds(0.3, TimerMode::Once),
                    );
                    screen_shake.intensity = 0.2;
                    screen_shake.duration = 0.3;
                    screen_shake.timer = 0.3;
                } else {
                    // Enter recovery
                    let recovery_time = match boss.phase {
                        BossPhase::Phase1 => 2.0,
                        BossPhase::Phase2 | BossPhase::Phase3 => 1.0,
                    };
                    boss.attack_state = AttackState::Recovery(
                        Timer::from_seconds(recovery_time, TimerMode::Once),
                    );
                    boss.combo_count = 0;
                }
            }
            // Movement handled by boss_idle_movement (Charging branch)
        }
        AttackState::Recovery(timer) => {
            timer.tick(std::time::Duration::from_secs_f32(delta));
            if timer.finished() {
                boss.attack_state = AttackState::Idle;
                boss.combo_count = 0;
            }
        }
        AttackState::Dashing { .. } | AttackState::Attacking => {
            // Not used by Chrome Berserker
        }
    }
}

/// Void Weaver attack pattern: hazard zones + teleport
/// Spawns HazardZone entities and teleports around the arena
pub fn weaver_attack(
    boss: &mut Boss,
    boss_transform: &mut Transform,
    player_transform: &Transform,
    commands: &mut Commands,
    delta: f32,
    hazard_count: usize,
    sound_events: &mut EventWriter<SoundEvent>,
) {
    let player_pos = player_transform.translation.truncate();

    match &mut boss.attack_state {
        AttackState::Idle => {
            boss.primary_timer.tick(std::time::Duration::from_secs_f32(delta));

            if boss.primary_timer.just_finished() {
                let max_hazards = match boss.phase {
                    BossPhase::Phase1 => 3,
                    BossPhase::Phase2 | BossPhase::Phase3 => 4,
                };

                if hazard_count < max_hazards {
                    sound_events.write(SoundEvent(SoundEffect::HazardSpawn));
                    // Spawn a HazardZone at random position
                    let x = (rand::random::<f32>() - 0.5) * 1000.0; // ±500
                    let y = (rand::random::<f32>() - 0.5) * 400.0;  // ±200

                    let drift_velocity = if boss.phase != BossPhase::Phase1 {
                        let dir = (player_pos - Vec2::new(x, y)).normalize_or_zero();
                        Some(dir * 30.0)
                    } else {
                        None
                    };

                    let explodes = boss.phase == BossPhase::Phase3;
                    let explosion_timer = if explodes {
                        Some(Timer::from_seconds(3.0, TimerMode::Once))
                    } else {
                        None
                    };

                    commands.spawn((
                        Sprite {
                            color: Color::srgba(0.5, 0.0, 1.0, 0.3),
                            custom_size: Some(Vec2::new(60.0, 60.0)),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(x, y, 0.0)),
                        HazardZone {
                            radius: 30.0,
                            lifetime: Timer::from_seconds(8.0, TimerMode::Once),
                            drift_velocity,
                            explodes,
                            explosion_timer,
                            damage: 5,
                        },
                        GameEntity,
                    ));
                }

                // Enter recovery (teleport cooldown)
                let teleport_cd = match boss.phase {
                    BossPhase::Phase1 => 3.0,
                    BossPhase::Phase2 => 2.5,
                    BossPhase::Phase3 => 1.5,
                };
                boss.attack_state = AttackState::Recovery(
                    Timer::from_seconds(teleport_cd, TimerMode::Once),
                );
            }
        }
        AttackState::Recovery(timer) => {
            timer.tick(std::time::Duration::from_secs_f32(delta));
            if timer.finished() {
                // Teleport boss to random position
                let x = (rand::random::<f32>() - 0.5) * 1000.0;
                let y = (rand::random::<f32>() - 0.5) * 400.0;
                boss_transform.translation.x = x;
                boss_transform.translation.y = y;
                boss.attack_state = AttackState::Idle;
            }
        }
        _ => {
            // Void Weaver doesn't use other states
        }
    }
}
