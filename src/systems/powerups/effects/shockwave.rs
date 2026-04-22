use crate::app::{GameEntity, ScreenShake};
use crate::core::boss::components::Boss;
use crate::core::boss::components::{BossProjectile, ChargeTelegraph, DashTrail, HazardZone};
use crate::systems::audio::{SoundEffect, SoundEvent};
use crate::systems::combat::EnemyParticle;
use bevy::prelude::*;

#[derive(Component)]
pub struct PowerUpShockwave {
    pub timer: Timer,
}

/// Apply shockwave effect: clear projectiles/hazards, damage boss, screen shake, spawn ring, play sound.
#[allow(clippy::too_many_arguments)]
pub fn apply_shockwave(
    commands: &mut Commands,
    player_pos: Vec3,
    boss_query: &mut Query<&mut Boss>,
    enemy_particle_query: &Query<Entity, With<EnemyParticle>>,
    boss_projectile_query: &Query<Entity, With<BossProjectile>>,
    dash_trail_query: &Query<Entity, With<DashTrail>>,
    hazard_zone_query: &Query<Entity, With<HazardZone>>,
    telegraph_query: &Query<Entity, With<ChargeTelegraph>>,
    screen_shake: &mut ScreenShake,
    sound_events: &mut EventWriter<SoundEvent>,
) {
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

pub fn powerup_shockwave_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PowerUpShockwave, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut shockwave, mut transform, mut sprite) in query.iter_mut() {
        shockwave.timer.tick(time.delta());
        let progress = shockwave.timer.fraction();
        let scale = 1.0 + progress * 20.0;
        transform.scale = Vec3::splat(scale);
        let alpha = (1.0 - progress) * 0.9;
        sprite.color = Color::srgba(8.0, 8.0, 8.0, alpha);
        if shockwave.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}
