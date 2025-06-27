use bevy::prelude::*;
use crate::core::enemies::components::Enemy;
use crate::app::GameEntity;
use crate::core::player::components::{Player, PlayerRotationTracker, PlayerParticle};

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
    mut query: Query<(&mut Enemy, &GlobalTransform, &Transform)>,
) {
    for (mut boss, global_transform, local_transform) in &mut query {
        if let Some(timer) = boss.fire_timer.as_mut() {
            timer.tick(time.delta());
        }

        if boss.fire_timer.as_ref().map_or(false, |t| t.just_finished()) {
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
    input: Res<ButtonInput<KeyCode>>,
) {
    if !input.pressed(KeyCode::Space) {
        return;
    }

    for (transform, mut tracker) in &mut query {
        let rotation_z = transform.rotation.to_euler(EulerRot::XYZ).2;

        // Normalisiere Rotation auf 0-360° in Radiant
        let angle = (rotation_z.rem_euclid(std::f32::consts::TAU)).to_degrees();

        // Snap auf die nächste 90°
        let index = (angle / 42.50).round() as i32 % 4;

        tracker.last_angle_index = index;

        // Schieße von 4 Ecken (relativ zur Würfelgröße)
        let offset = 16.0; // an Sprite-Größe anpassen
        let directions = [
            Vec2::new( offset,  offset),
            Vec2::new(-offset,  offset),
            Vec2::new(-offset, -offset),
            Vec2::new( offset, -offset),
        ];

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