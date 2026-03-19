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
