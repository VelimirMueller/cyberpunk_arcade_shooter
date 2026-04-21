use crate::app::{GameEntity, ScreenShake};
use crate::core::boss::components::Boss;
use crate::core::boss::components::{BossProjectile, ChargeTelegraph, DashTrail, HazardZone};
use crate::core::player::components::Player;
use crate::systems::audio::{SoundEffect, SoundEvent};
use crate::systems::collision::collide;
use crate::systems::combat::EnemyParticle;
use crate::utils::config::ENTITY_SCALE;
use bevy::prelude::*;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpKind {
    Shockwave,
    Laser,
}

#[derive(Component)]
pub struct PowerUp {
    pub kind: PowerUpKind,
    pub lifetime: Timer,
}

pub const LASER_CHARGE_DURATION: f32 = 0.8;
pub const LASER_ACTIVE_DURATION: f32 = 5.2;
pub const LASER_FADE_DURATION: f32 = 0.8;
pub const LASER_TOTAL_DURATION: f32 =
    LASER_CHARGE_DURATION + LASER_ACTIVE_DURATION + LASER_FADE_DURATION;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaserPhase {
    Charging,
    Active,
    FadingOut,
}

pub fn laser_phase_from_elapsed(elapsed: f32) -> LaserPhase {
    if elapsed < LASER_CHARGE_DURATION {
        LaserPhase::Charging
    } else if elapsed < LASER_CHARGE_DURATION + LASER_ACTIVE_DURATION {
        LaserPhase::Active
    } else {
        LaserPhase::FadingOut
    }
}

#[derive(Component)]
pub struct LaserActive {
    pub timer: Timer,
    pub sound_timer: Timer,
    pub phase: LaserPhase,
    pub charge_timer: Timer,
}

#[derive(Component)]
pub struct LaserBeamCore;

#[derive(Component)]
pub struct LaserBeamShell {
    pub pulse_timer: f32,
}

#[derive(Component)]
pub struct LaserStreamParticle {
    pub lifetime: Timer,
    #[allow(dead_code)]
    pub drift_offset: f32,
    pub side: f32,
}

#[derive(Component)]
pub struct LaserImpact;

#[derive(Component)]
pub struct LaserMuzzle;

#[derive(Component)]
pub struct LaserChargeParticle {
    pub target: Vec2,
    pub speed: f32,
}

#[derive(Component)]
pub struct LaserChargeOrb {
    pub scale: f32,
}

#[derive(Resource)]
pub struct PowerUpTimer {
    pub timer: Timer,
}

#[derive(Component)]
pub struct PowerUpShockwave {
    pub timer: Timer,
}

pub fn setup_powerup_timer(mut commands: Commands) {
    let duration = 15.0 + rand::random::<f32>() * 5.0;
    commands.insert_resource(PowerUpTimer {
        timer: Timer::from_seconds(duration, TimerMode::Once),
    });
}

pub fn powerup_spawn_system(
    mut commands: Commands,
    time: Res<Time>,
    mut powerup_timer: ResMut<PowerUpTimer>,
    existing_powerups: Query<&PowerUp>,
) {
    powerup_timer.timer.tick(time.delta());
    if !powerup_timer.timer.finished() {
        return;
    }

    // Only 1 power-up on screen at a time
    if !existing_powerups.is_empty() {
        let duration = 15.0 + rand::random::<f32>() * 5.0;
        powerup_timer.timer = Timer::from_seconds(duration, TimerMode::Once);
        return;
    }

    // Random position within play bounds
    let x = (rand::random::<f32>() - 0.5) * 1000.0;
    let y = (rand::random::<f32>() - 0.5) * 400.0;

    // Random 50/50 choice
    let kind = if rand::random::<bool>() {
        PowerUpKind::Shockwave
    } else {
        PowerUpKind::Laser
    };

    let color = match kind {
        PowerUpKind::Shockwave => Color::srgb(0.0, 8.0, 8.0),
        PowerUpKind::Laser => Color::srgb(8.0, 0.0, 8.0),
    };

    commands.spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(16.0 * ENTITY_SCALE, 16.0 * ENTITY_SCALE)),
            ..default()
        },
        Transform::from_xyz(x, y, 0.5)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
        PowerUp {
            kind,
            lifetime: Timer::from_seconds(10.0, TimerMode::Once),
        },
        GameEntity,
    ));

    // Reset timer for next spawn
    let duration = 15.0 + rand::random::<f32>() * 5.0;
    powerup_timer.timer = Timer::from_seconds(duration, TimerMode::Once);
}

pub fn powerup_lifetime_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PowerUp, &mut Sprite)>,
) {
    let t = time.elapsed_secs();
    for (entity, mut powerup, mut sprite) in query.iter_mut() {
        powerup.lifetime.tick(time.delta());

        // Gentle pulse animation
        let pulse = 0.6 + 0.4 * (t * 4.0).sin();
        let base = match powerup.kind {
            PowerUpKind::Shockwave => Color::srgba(0.0, 8.0, 8.0, pulse),
            PowerUpKind::Laser => Color::srgba(8.0, 0.0, 8.0, pulse),
        };
        sprite.color = base;

        if powerup.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn powerup_pickup_system(
    mut commands: Commands,
    player_query: Query<(&Transform, &Sprite, Entity), With<Player>>,
    powerup_query: Query<(Entity, &Transform, &Sprite, &PowerUp)>,
    mut boss_query: Query<&mut Boss>,
    enemy_particle_query: Query<Entity, With<EnemyParticle>>,
    boss_projectile_query: Query<Entity, With<BossProjectile>>,
    dash_trail_query: Query<Entity, With<DashTrail>>,
    hazard_zone_query: Query<Entity, With<HazardZone>>,
    telegraph_query: Query<Entity, With<ChargeTelegraph>>,
    mut screen_shake: ResMut<ScreenShake>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    let Ok((player_transform, player_sprite, player_entity)) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation;
    let player_size = player_sprite.custom_size.unwrap_or(Vec2::ONE);

    for (powerup_entity, powerup_transform, powerup_sprite, powerup) in &powerup_query {
        let powerup_pos = powerup_transform.translation;
        let powerup_size = powerup_sprite.custom_size.unwrap_or(Vec2::new(16.0, 16.0));

        if !collide(player_pos, player_size, powerup_pos, powerup_size) {
            continue;
        }

        // Consume power-up
        commands.entity(powerup_entity).despawn();

        match powerup.kind {
            PowerUpKind::Shockwave => {
                // 1. Despawn all projectiles/hazards
                for entity in enemy_particle_query.iter() {
                    commands.entity(entity).despawn();
                }
                for entity in boss_projectile_query.iter() {
                    commands.entity(entity).despawn();
                }
                for entity in dash_trail_query.iter() {
                    commands.entity(entity).despawn();
                }
                for entity in hazard_zone_query.iter() {
                    commands.entity(entity).despawn();
                }
                for entity in telegraph_query.iter() {
                    commands.entity(entity).despawn();
                }

                // 2. Deal 20 damage to boss
                for mut boss in boss_query.iter_mut() {
                    if boss.current_hp > 0 {
                        boss.current_hp = boss.current_hp.saturating_sub(20);
                        sound_events.write(SoundEvent(SoundEffect::EnemyHit));
                    }
                }

                // 3. Screen shake
                screen_shake.intensity = 2.0;
                screen_shake.duration = 0.5;
                screen_shake.timer = 0.5;

                // 4. Spawn shockwave ring visual
                commands.spawn((
                    Sprite {
                        color: Color::srgba(8.0, 8.0, 8.0, 0.9),
                        custom_size: Some(Vec2::new(20.0, 20.0)),
                        ..default()
                    },
                    Transform::from_translation(player_pos),
                    PowerUpShockwave {
                        timer: Timer::from_seconds(0.5, TimerMode::Once),
                    },
                    GameEntity,
                ));

                // 5. Sound
                sound_events.write(SoundEvent(SoundEffect::ShockwavePowerUp));
            }
            PowerUpKind::Laser => {
                // Add LaserActive to player (charge phase)
                commands.entity(player_entity).insert(LaserActive {
                    timer: Timer::from_seconds(LASER_TOTAL_DURATION, TimerMode::Once),
                    sound_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                    phase: LaserPhase::Charging,
                    charge_timer: Timer::from_seconds(LASER_CHARGE_DURATION, TimerMode::Once),
                });

                // Spawn charge orb at player
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.4, 1.0, 0.4, 0.9),
                        custom_size: Some(Vec2::new(8.0, 8.0)),
                        ..default()
                    },
                    Transform::from_translation(player_pos.with_z(0.4)),
                    LaserChargeOrb { scale: 1.0 },
                    GameEntity,
                ));

                // Spawn 8 converging charge particles from random screen positions
                let mut rng = rand::thread_rng();
                for _ in 0..8 {
                    let px = (rng.gen_range(-1.0_f32..1.0_f32)) * 600.0;
                    let py = (rng.gen_range(-1.0_f32..1.0_f32)) * 350.0;
                    let speed = rng.gen_range(200.0_f32..400.0_f32);
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(0.2, 1.0, 0.3, 0.8),
                            custom_size: Some(Vec2::new(4.0, 4.0)),
                            ..default()
                        },
                        Transform::from_xyz(px, py, 0.35),
                        LaserChargeParticle {
                            target: player_pos.truncate(),
                            speed,
                        },
                        GameEntity,
                    ));
                }

                // Subtle screen shake
                screen_shake.intensity = 0.2;
                screen_shake.duration = 0.8;
                screen_shake.timer = 0.8;

                // Play charge sound
                sound_events.write(SoundEvent(SoundEffect::LaserCharge));
            }
        }
    }
}

pub fn powerup_shockwave_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PowerUpShockwave, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut shockwave, mut transform, mut sprite) in query.iter_mut() {
        shockwave.timer.tick(time.delta());
        let progress = shockwave.timer.fraction();
        // Expand rapidly
        let scale = 1.0 + progress * 20.0;
        transform.scale = Vec3::splat(scale);
        // Fade out
        let alpha = (1.0 - progress) * 0.9;
        sprite.color = Color::srgba(8.0, 8.0, 8.0, alpha);
        if shockwave.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn laser_system(
    time: Res<Time>,
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, Option<&mut LaserActive>), With<Player>>,
    mut core_query: Query<
        (Entity, &mut Transform, &mut Sprite),
        (
            With<LaserBeamCore>,
            Without<Player>,
            Without<Boss>,
            Without<LaserBeamShell>,
        ),
    >,
    mut shell_query: Query<
        (Entity, &mut Transform, &mut Sprite, &mut LaserBeamShell),
        (Without<Player>, Without<Boss>, Without<LaserBeamCore>),
    >,
    muzzle_query: Query<Entity, With<LaserMuzzle>>,
    impact_query: Query<Entity, With<LaserImpact>>,
    mut boss_query: Query<
        (&mut Boss, &Transform, &Sprite),
        (
            Without<Player>,
            Without<LaserBeamCore>,
            Without<LaserBeamShell>,
        ),
    >,
    mut screen_shake: ResMut<ScreenShake>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    let Ok((player_entity, player_transform, laser_active)) = player_query.single_mut() else {
        return;
    };

    let Some(mut laser) = laser_active else {
        return;
    };

    let dt = time.delta();
    laser.timer.tick(dt);
    laser.sound_timer.tick(dt);
    laser.charge_timer.tick(dt);

    let elapsed = laser.timer.elapsed_secs();
    let new_phase = laser_phase_from_elapsed(elapsed);

    // --- Phase transitions ---
    if laser.phase != new_phase {
        match new_phase {
            LaserPhase::Active => {
                // Transition: Charging -> Active
                screen_shake.intensity = 1.0;
                screen_shake.duration = 0.3;
                screen_shake.timer = 0.3;
                sound_events.write(SoundEvent(SoundEffect::LaserFire));

                let player_pos = player_transform.translation;
                let player_rotation = player_transform.rotation;
                let forward = player_rotation * Vec3::Y * 300.0;

                // Spawn core beam (bright white/cyan, 6px wide)
                commands.spawn((
                    Sprite {
                        color: Color::srgba(4.0, 8.0, 6.0, 1.0),
                        custom_size: Some(Vec2::new(6.0 * ENTITY_SCALE, 600.0)),
                        ..default()
                    },
                    Transform::from_translation((player_pos + forward).with_z(0.5))
                        .with_rotation(player_rotation),
                    LaserBeamCore,
                    GameEntity,
                ));

                // Spawn shell beam (translucent green, 32px wide)
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.0, 4.0, 0.5, 0.55),
                        custom_size: Some(Vec2::new(32.0 * ENTITY_SCALE, 600.0)),
                        ..default()
                    },
                    Transform::from_translation((player_pos + forward).with_z(0.4))
                        .with_rotation(player_rotation),
                    LaserBeamShell { pulse_timer: 0.0 },
                    GameEntity,
                ));

                // Spawn muzzle flash (40x20 at player)
                commands.spawn((
                    Sprite {
                        color: Color::srgba(2.0, 8.0, 4.0, 0.8),
                        custom_size: Some(Vec2::new(40.0 * ENTITY_SCALE, 20.0 * ENTITY_SCALE)),
                        ..default()
                    },
                    Transform::from_translation(player_pos.with_z(0.6)),
                    LaserMuzzle,
                    GameEntity,
                ));
            }
            LaserPhase::FadingOut => {
                // Transition: Active -> FadingOut
                sound_events.write(SoundEvent(SoundEffect::LaserFadeOut));
            }
            LaserPhase::Charging => {}
        }
        laser.phase = new_phase;
    }

    let player_pos = player_transform.translation;
    let player_rotation = player_transform.rotation;
    let forward_dir = player_rotation * Vec3::Y;
    let beam_center = player_pos + forward_dir * 300.0;

    match laser.phase {
        LaserPhase::Charging => {
            // Nothing — charge particles are handled by laser_charge_particle_system
        }
        LaserPhase::Active => {
            // Update core position and rotation
            for (_entity, mut t, _sprite) in core_query.iter_mut() {
                t.translation = (player_pos + forward_dir * 300.0).with_z(0.5);
                t.rotation = player_rotation;
            }

            // Update shell position, rotation, and pulse width
            for (_entity, mut t, mut sprite, mut shell) in shell_query.iter_mut() {
                t.translation = (player_pos + forward_dir * 300.0).with_z(0.4);
                t.rotation = player_rotation;
                shell.pulse_timer += dt.as_secs_f32();
                let pulse = (30.0 + 6.0 * (shell.pulse_timer * std::f32::consts::TAU * 1.7).sin())
                    * ENTITY_SCALE;
                if let Some(size) = sprite.custom_size.as_mut() {
                    size.x = pulse;
                }
            }

            // Update muzzle position
            for entity in muzzle_query.iter() {
                commands
                    .entity(entity)
                    .insert(Transform::from_translation(player_pos.with_z(0.6)));
            }

            // Periodically spawn stream particles on sound_timer ticks
            if laser.sound_timer.just_finished() {
                sound_events.write(SoundEvent(SoundEffect::LaserHum));
                let mut rng = rand::thread_rng();
                // Spawn a stream particle along beam direction
                let offset_along = rng.gen_range(0.0_f32..580.0_f32);
                let drift = rng.gen_range(-18.0_f32..18.0_f32);
                let side: f32 = if rng.gen_bool(0.5) { 1.0 } else { -1.0 };
                let right = player_rotation * Vec3::X;
                let spawn_pos = player_pos + forward_dir * offset_along + right * drift;
                commands.spawn((
                    Sprite {
                        color: Color::srgba(0.0, 6.0, 2.0, 0.7),
                        custom_size: Some(Vec2::new(3.0, 6.0)),
                        ..default()
                    },
                    Transform::from_translation(spawn_pos.with_z(0.45))
                        .with_rotation(player_rotation),
                    LaserStreamParticle {
                        lifetime: Timer::from_seconds(0.3, TimerMode::Once),
                        drift_offset: drift,
                        side,
                    },
                    GameEntity,
                ));
            }

            // Beam vs Boss collision
            let right = player_rotation * Vec3::X;
            let beam_half_width = 16.0; // half of 32px shell
            let beam_half_length = 300.0;
            let extent_x = (right.x.abs() * beam_half_width
                + forward_dir.x.abs() * beam_half_length)
                .max(beam_half_width);
            let extent_y = (right.y.abs() * beam_half_width
                + forward_dir.y.abs() * beam_half_length)
                .max(beam_half_width);
            let beam_aabb_size = Vec2::new(extent_x * 2.0, extent_y * 2.0);

            for (mut boss, boss_transform, boss_sprite) in boss_query.iter_mut() {
                if boss.current_hp == 0 || boss.is_invulnerable {
                    continue;
                }
                let boss_size = boss_sprite.custom_size.unwrap_or(Vec2::ONE);
                if collide(
                    beam_center,
                    beam_aabb_size,
                    boss_transform.translation,
                    boss_size,
                ) && boss
                    .last_laser_hit_time
                    .is_none_or(|t| t.elapsed().as_secs_f32() > 0.075)
                {
                    boss.current_hp = boss.current_hp.saturating_sub(1);
                    boss.last_laser_hit_time = Some(crate::utils::time_compat::Instant::now());
                    sound_events.write(SoundEvent(SoundEffect::EnemyHit));

                    // Spawn/update impact at boss position
                    // Despawn old impact and respawn fresh
                    for entity in impact_query.iter() {
                        commands.entity(entity).despawn();
                    }
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(2.0, 8.0, 2.0, 0.9),
                            custom_size: Some(Vec2::new(20.0, 20.0)),
                            ..default()
                        },
                        Transform::from_translation(boss_transform.translation.with_z(0.6)),
                        LaserImpact,
                        GameEntity,
                    ));
                }
            }
        }
        LaserPhase::FadingOut => {
            let fade_elapsed = elapsed - (LASER_CHARGE_DURATION + LASER_ACTIVE_DURATION);
            let fade_progress = (fade_elapsed / LASER_FADE_DURATION).clamp(0.0, 1.0);
            let alpha = (1.0 - fade_progress).max(0.0);

            // Narrow and fade core
            for (_entity, mut t, mut sprite) in core_query.iter_mut() {
                t.translation = (player_pos + forward_dir * 300.0).with_z(0.5);
                t.rotation = player_rotation;
                let width = 6.0 * (1.0 - fade_progress).max(0.01);
                sprite.custom_size = Some(Vec2::new(width, 600.0));
                sprite.color = Color::srgba(4.0, 8.0, 6.0, alpha);
            }

            // Fade shell
            for (_entity, mut t, mut sprite, _shell) in shell_query.iter_mut() {
                t.translation = (player_pos + forward_dir * 300.0).with_z(0.4);
                t.rotation = player_rotation;
                sprite.color = Color::srgba(0.0, 4.0, 0.5, 0.55 * alpha);
            }
        }
    }

    // Expire: despawn everything and remove LaserActive
    if laser.timer.finished() {
        commands.entity(player_entity).remove::<LaserActive>();
        for (entity, _, _) in core_query.iter() {
            commands.entity(entity).despawn();
        }
        for (entity, _, _, _) in shell_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in muzzle_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in impact_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

/// Move charge particles toward the player (their target), fade as they approach, despawn when close.
pub fn laser_charge_particle_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut Sprite,
        &mut LaserChargeParticle,
    )>,
    player_query: Query<&Transform, (With<Player>, Without<LaserChargeParticle>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();

    let dt = time.delta().as_secs_f32();
    for (entity, mut transform, mut sprite, mut particle) in query.iter_mut() {
        // Update target each frame so particles track player
        particle.target = player_pos;

        let pos = transform.translation.truncate();
        let to_target = particle.target - pos;
        let dist = to_target.length();

        if dist < 8.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let direction = to_target.normalize();
        let move_dist = (particle.speed * dt).min(dist);
        transform.translation.x += direction.x * move_dist;
        transform.translation.y += direction.y * move_dist;

        // Fade based on proximity (full opacity from far, fades to 0 when close)
        let alpha = (dist / 400.0).clamp(0.0, 1.0) * 0.8;
        sprite.color = Color::srgba(0.2, 1.0, 0.3, alpha);
    }
}

/// Position the charge orb at the player and manage its scale through the charge and early active phase.
#[allow(clippy::type_complexity)]
pub fn laser_charge_orb_system(
    _time: Res<Time>,
    mut commands: Commands,
    mut orb_query: Query<(Entity, &mut Transform, &mut Sprite, &mut LaserChargeOrb)>,
    player_query: Query<
        (&Transform, Option<&LaserActive>),
        (With<Player>, Without<LaserChargeOrb>),
    >,
) {
    let Ok((player_transform, laser_active)) = player_query.single() else {
        return;
    };

    for (entity, mut transform, mut sprite, mut orb) in orb_query.iter_mut() {
        transform.translation = player_transform.translation.with_z(0.4);

        match laser_active {
            None => {
                // LaserActive removed — despawn orb
                commands.entity(entity).despawn();
            }
            Some(laser) => {
                let elapsed = laser.timer.elapsed_secs();
                match laser.phase {
                    LaserPhase::Charging => {
                        // Scale up from 1 to 3 during charge phase
                        let progress = (elapsed / LASER_CHARGE_DURATION).clamp(0.0, 1.0);
                        orb.scale = 1.0 + 2.0 * progress;
                        transform.scale = Vec3::splat(orb.scale);
                        sprite.color = Color::srgba(0.4, 1.0, 0.4, 0.9);
                    }
                    LaserPhase::Active => {
                        // Shrink during first 0.3s of active phase
                        let active_elapsed = elapsed - LASER_CHARGE_DURATION;
                        let shrink_progress = (active_elapsed / 0.3).clamp(0.0, 1.0);
                        let scale = 3.0 * (1.0 - shrink_progress);
                        if scale <= 0.01 {
                            commands.entity(entity).despawn();
                        } else {
                            orb.scale = scale;
                            transform.scale = Vec3::splat(scale);
                            sprite.color =
                                Color::srgba(0.4, 1.0, 0.4, 0.9 * (1.0 - shrink_progress));
                        }
                    }
                    LaserPhase::FadingOut => {
                        commands.entity(entity).despawn();
                    }
                }
            }
        }
    }
}

/// Move stream particles along beam direction, drift outward, fade over lifetime.
pub fn laser_stream_particle_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut Transform,
        &mut Sprite,
        &mut LaserStreamParticle,
    )>,
    player_query: Query<&Transform, (With<Player>, Without<LaserStreamParticle>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return;
    };
    let forward_dir = player_transform.rotation * Vec3::Y;
    let right_dir = player_transform.rotation * Vec3::X;
    let dt = time.delta().as_secs_f32();

    for (entity, mut transform, mut sprite, mut particle) in query.iter_mut() {
        particle.lifetime.tick(time.delta());

        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let progress = particle.lifetime.fraction();
        // Move along beam direction
        transform.translation += forward_dir * 80.0 * dt;
        // Drift outward
        transform.translation += right_dir * particle.side * 20.0 * dt;

        let alpha = (1.0 - progress) * 0.7;
        sprite.color = Color::srgba(0.0, 6.0, 2.0, alpha);
    }
}

/// Position the laser impact at boss position, pulse scale/opacity, hide when laser not active.
pub fn laser_impact_system(
    time: Res<Time>,
    mut commands: Commands,
    mut impact_query: Query<(Entity, &mut Transform, &mut Sprite), With<LaserImpact>>,
    player_query: Query<Option<&LaserActive>, With<Player>>,
    boss_query: Query<&Transform, (With<Boss>, Without<LaserImpact>)>,
) {
    let Ok(laser_active) = player_query.single() else {
        return;
    };

    if laser_active.is_none_or(|l| l.phase != LaserPhase::Active) {
        // Laser not active — despawn any impact entities
        for (entity, _, _) in impact_query.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }

    let t = time.elapsed_secs();
    if let Ok(boss_transform) = boss_query.single() {
        for (_entity, mut transform, mut sprite) in impact_query.iter_mut() {
            transform.translation = boss_transform.translation.with_z(0.6);
            let pulse = 0.7 + 0.3 * (t * 8.0).sin();
            let scale = 1.0 + 0.4 * (t * 6.0).sin();
            transform.scale = Vec3::splat(scale);
            sprite.color = Color::srgba(2.0, 8.0, 2.0, 0.9 * pulse);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_laser_total_duration() {
        assert!((LASER_TOTAL_DURATION - 6.8).abs() < f32::EPSILON);
    }

    #[test]
    fn test_laser_phase_from_elapsed() {
        assert_eq!(laser_phase_from_elapsed(0.0), LaserPhase::Charging);
        assert_eq!(laser_phase_from_elapsed(0.5), LaserPhase::Charging);
        assert_eq!(laser_phase_from_elapsed(0.8), LaserPhase::Active);
        assert_eq!(laser_phase_from_elapsed(3.0), LaserPhase::Active);
        assert_eq!(laser_phase_from_elapsed(6.0), LaserPhase::FadingOut);
        assert_eq!(laser_phase_from_elapsed(6.5), LaserPhase::FadingOut);
    }
}
