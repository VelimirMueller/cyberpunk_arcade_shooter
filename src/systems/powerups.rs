use bevy::prelude::*;
use crate::app::{GameEntity, ScreenShake};
use crate::core::boss::components::Boss;
use crate::core::player::components::Player;
use crate::systems::audio::{SoundEvent, SoundEffect};
use crate::systems::collision::collide;
use crate::systems::combat::EnemyParticle;
use crate::core::boss::components::{BossProjectile, DashTrail, HazardZone, ChargeTelegraph};

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

#[derive(Component)]
pub struct LaserActive {
    pub timer: Timer,
    pub sound_timer: Timer,
}

#[derive(Component)]
pub struct LaserBeam;

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
            custom_size: Some(Vec2::new(16.0, 16.0)),
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
                    boss.current_hp = boss.current_hp.saturating_sub(20);
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
                // Add LaserActive to player
                commands.entity(player_entity).insert(LaserActive {
                    timer: Timer::from_seconds(6.0, TimerMode::Once),
                    sound_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
                });

                // Spawn laser beam entity
                commands.spawn((
                    Sprite {
                        color: Color::srgb(1.0, 8.0, 0.7),
                        custom_size: Some(Vec2::new(10.0, 600.0)),
                        ..default()
                    },
                    Transform::from_translation(player_pos + Vec3::new(0.0, 300.0, 0.3)),
                    LaserBeam,
                    GameEntity,
                ));

                sound_events.write(SoundEvent(SoundEffect::LaserHum));
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
