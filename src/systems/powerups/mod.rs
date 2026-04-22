use crate::app::{GameEntity, ScreenShake};
use crate::core::boss::components::Boss;
use crate::core::boss::components::{BossProjectile, ChargeTelegraph, DashTrail, HazardZone};
use crate::core::player::components::Player;
use crate::systems::audio::SoundEvent;
use crate::systems::collision::collide;
use crate::systems::combat::EnemyParticle;
use crate::utils::config::ENTITY_SCALE;
use bevy::prelude::*;

pub mod effects;
#[allow(unused_imports)]
pub use effects::laser::{
    LASER_ACTIVE_DURATION, LASER_CHARGE_DURATION, LASER_FADE_DURATION, LASER_TOTAL_DURATION,
    LaserActive, LaserBeamCore, LaserBeamShell, LaserChargeOrb, LaserChargeParticle, LaserImpact,
    LaserMuzzle, LaserPhase, LaserStreamParticle, laser_charge_orb_system,
    laser_charge_particle_system, laser_impact_system, laser_phase_from_elapsed,
    laser_stream_particle_system, laser_system,
};
pub use effects::shockwave::{PowerUpShockwave, powerup_shockwave_system};

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

#[derive(Resource)]
pub struct PowerUpTimer {
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
                crate::systems::powerups::effects::shockwave::apply_shockwave(
                    &mut commands,
                    player_pos,
                    &mut boss_query,
                    &enemy_particle_query,
                    &boss_projectile_query,
                    &dash_trail_query,
                    &hazard_zone_query,
                    &telegraph_query,
                    &mut screen_shake,
                    &mut sound_events,
                );
            }
            PowerUpKind::Laser => {
                crate::systems::powerups::effects::laser::apply_laser_pickup(
                    &mut commands,
                    player_entity,
                    player_pos,
                    &mut screen_shake,
                    &mut sound_events,
                );
            }
        }
    }
}
