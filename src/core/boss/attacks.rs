// Per-boss attack patterns — implemented in Task 4-8

use bevy::prelude::*;
use crate::app::GameEntity;
use crate::core::boss::components::*;

/// Grid Phantom attack pattern: dash + telegraph + trail
/// State machine: Idle → WindUp → Dashing → Recovery (with optional chain dash in Phase 3)
pub fn phantom_attack(
    boss: &mut Boss,
    boss_transform: &Transform,
    player_transform: &Transform,
    commands: &mut Commands,
    delta: f32,
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
