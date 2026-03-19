use bevy::prelude::*;
use crate::core::boss::components::Boss;
use crate::app::GameEntity;
use crate::core::player::components::{Player, PlayerRotationTracker, PlayerParticle};
use crate::systems::audio::{SoundEvent, SoundEffect};
use crate::systems::powerups::LaserActive;

#[derive(Component)]
pub struct EnemyParticle;

#[derive(Component)]
pub struct Velocity(pub Vec2);

pub(crate) fn spawn_enemy_particle_sprite(mut commands: Commands, position: Vec3, velocity: Vec2) {
    commands.spawn((
        Sprite {
            color: Color::srgb(5.2, 1.8, 5.2),
            custom_size: Some(Vec2::new(2.0, 2.0)),
            ..default()
        },
        Transform::from_translation(position),
        Velocity(velocity),
        EnemyParticle,
        GameEntity
    ));
}

pub fn particle_movement_system(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform), With<EnemyParticle>>,
) {
    let dt = time.delta().as_secs_f32();

    for (velocity, mut transform) in &mut query {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;
    }
}

pub fn particle_cleanup_system(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<EnemyParticle>>,
    player_query: Query<(Entity, &Transform), With<PlayerParticle>>
) {
    const SCREEN_BOUNDS: f32 = 600.0; // adjust to your camera view

    for (entity, transform) in &query {
        let pos = transform.translation;
        if pos.x.abs() > SCREEN_BOUNDS || pos.y.abs() > SCREEN_BOUNDS {
            commands.entity(entity).despawn();
        }
    }

    for (entity, transform) in &player_query {
        let pos = transform.translation;
        if pos.x.abs() > SCREEN_BOUNDS || pos.y.abs() > SCREEN_BOUNDS {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn boss_shoot_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&mut Boss, &GlobalTransform, &Transform)>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    for (mut boss, global_transform, local_transform) in &mut query {
        boss.primary_timer.tick(time.delta());

        if boss.primary_timer.just_finished() {
            sound_events.write(SoundEvent(SoundEffect::EnemyShoot));
            let scale = local_transform.scale.xy(); // assume uniform scale for cube
            let half_width = 0.5 * scale.x;
            let half_height = 0.5 * scale.y;

            let corners = [
                Vec2::new(half_width, half_height),
                Vec2::new(-half_width, half_height),
                Vec2::new(half_width, -half_height),
                Vec2::new(-half_width, -half_height),
            ];

            for corner in corners {
                let corner_world = global_transform.transform_point(Vec3::new(corner.x, corner.y, 0.0));
                let velocity = corner.normalize_or_zero() * 120.0;

                spawn_enemy_particle_sprite(commands.reborrow(), corner_world, velocity);
            }
        }
    }
}

pub fn player_particle_movement_system(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Transform), With<PlayerParticle>>,
) {
    let dt = time.delta().as_secs_f32();

    for (velocity, mut transform) in &mut query {
        transform.translation.x += velocity.0.x * dt;
        transform.translation.y += velocity.0.y * dt;
    }
}

pub(crate) fn player_shoot_system(
    mut commands: Commands,
    mut query: Query<(&Transform, &mut PlayerRotationTracker)>,
    mut player: Query<(&mut Player, &Transform, &Sprite, Option<&LaserActive>)>,
    input: Res<ButtonInput<KeyCode>>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    if !input.pressed(KeyCode::Space) {
        return;
    }

    const SHOT_COOLDOWN: f32 = 0.15; // Seconds between shots
    const ENERGY_COST_PER_CORNER: u32 = 1; // Reduced from 3
    const MIN_ENERGY: u32 = 4; // Need at least this much to shoot (4 corners)

    for (transform, mut tracker) in &mut query {
        let rotation_z = transform.rotation.to_euler(EulerRot::XYZ).2;

        // Normalisiere Rotation auf 0-360° in Radiant
        let angle = (rotation_z.rem_euclid(std::f32::consts::TAU)).to_degrees();

        // Snap auf die nächste 90°
        let index = (angle / 42.50).round() as i32 % 4;

        tracker.last_angle_index = index;

        if let Ok((mut player_data, _transform, _sprite, laser_active)) = player.single_mut() {
            if laser_active.is_some() {
                return; // Suppress normal shooting during laser mode
            }
            // Check cooldown
            let can_shoot = player_data.last_shot_time.map_or(true, |t| t.elapsed().as_secs_f32() >= SHOT_COOLDOWN);

            if can_shoot && player_data.energy >= MIN_ENERGY {
                player_data.last_shot_time = Some(std::time::Instant::now());

                // Schieße von 4 Ecken (relativ zur Würfelgröße)
                let offset = 16.0; // an Sprite-Größe anpassen
                let directions = [
                    Vec2::new( offset,  offset),
                    Vec2::new(-offset,  offset),
                    Vec2::new(-offset, -offset),
                    Vec2::new( offset, -offset),
                ];

                let total_cost = ENERGY_COST_PER_CORNER * directions.len() as u32;
                if let Some(energy) = player_data.energy.checked_sub(total_cost) {
                    player_data.energy = energy;
                    sound_events.write(SoundEvent(SoundEffect::PlayerShoot));

                    for dir in directions {
                        // Drehe Ecken-Offset mit Spielerrotation
                        let rotated = transform.rotation * dir.extend(0.0);
                        let pos = transform.translation + rotated;

                        commands.spawn((
                            Sprite {
                                color: Color::srgb(1.0, 7.3, 0.7),
                                custom_size: Some(Vec2::splat(3.0)),
                                ..default()
                            },
                            Transform::from_translation(pos),
                            Velocity(rotated.truncate().normalize() * 500.0),
                            PlayerParticle,
                        ));
                    }
                }
            }
        }
    }
}
