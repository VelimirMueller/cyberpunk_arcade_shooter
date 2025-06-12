use bevy::prelude::*;
use crate::core::enemies::components::{Enemy, EnemyMovement};
use crate::env::{ROTATE_SPEED, TIME_STEP};
use crate::app::GameEntity;

pub(crate) fn create_enemies(
    commands: Commands,
) {
    let enemy_colors = [
        Color::srgba(4.5, 1.2, 2.2, 0.7),
        Color::srgba(2.5, 1.2, 4.2, 0.4),
        Color::srgba(1.5, 1.2, 4.2, 0.23)
    ];
    spawn_enemy(commands, enemy_colors);
}
fn spawn_enemy(
    mut commands: Commands,
    enemy_colors: [Color; 3]) {
    let mut count = 0.0;

    let arena_size = Vec2::new(900.0, 400.0); // Example bounds
    for color in enemy_colors {
        commands.spawn((
            Enemy { current: 100, max: 100, fire_timer: if count == 0.0 {
                Some(Timer::from_seconds(0.42, TimerMode::Repeating))
            } else {
                None
            }, },
            Sprite {
                color,
                custom_size: Some(Vec2::new(50.0, 50.0 * (count+0.85)/1.25)),
                ..default()
            },
            GameEntity,
            Transform::from_xyz((22.0) * count/2.0, (10.0 ) * count, (4.0) * count),
            GlobalTransform::default(),
            EnemyMovement {
                corners: vec![
                    Vec3::new(-arena_size.x / 2.0,  arena_size.y / 2.0, 0.0), // Top-left
                    Vec3::new(-arena_size.x / 2.5,  arena_size.y / 2.5, 0.0),
                    Vec3::new(-arena_size.x / 2.9,  arena_size.y / 4.5, 0.0),
                    Vec3::new( arena_size.x / 2.0, -arena_size.y / 2.0, 0.0), // Bottom-right
                    Vec3::new( arena_size.x / 2.0,  arena_size.y / 2.0, 0.0), // Top-right
                    Vec3::new(-arena_size.x / 2.5,  arena_size.y / 2.5, 0.0),
                    Vec3::new(-arena_size.x / 7.5,  arena_size.y / 3.5, 0.0),
                    Vec3::new(-arena_size.x / 2.0, -arena_size.y / 2.0, 0.0), // Bottom-left
                    Vec3::new(-arena_size.x / 2.5,  arena_size.y / 5.5, 0.0),
                    Vec3::new(-arena_size.x / 8.5,  arena_size.y / 3.5, 0.0),
                    Vec3::new(-arena_size.x / 4.9,  arena_size.y / 1.99, 0.0),
                    Vec3::new(-arena_size.x / 2.2,  arena_size.y / 5.9, 0.0),
                ],
                current_target: 1,
                speed: 150.0,
            },
        ));

        count = count + 1.0;
    }
}

pub(crate) fn enemy_rotation(
    mut query: Query<&mut Transform, With<Enemy>>
) {
    for (i, mut transform) in query.iter_mut().enumerate() {
        if i % 3 == 1 {
            transform.rotate_z(ROTATE_SPEED * TIME_STEP/1.1);
        }
        if i % 2 == 1 {
            transform.rotate_z((-ROTATE_SPEED * 3.0)* TIME_STEP);
        } else {
            transform.rotate_z(-ROTATE_SPEED * TIME_STEP/2.0);
        }
    }
}

pub(crate) fn enemy_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut EnemyMovement), With<Enemy>>,
) {
    for (mut transform, mut movement) in &mut query {
        let target = movement.corners[movement.current_target];
        let direction = (target - transform.translation).normalize();
        let distance = transform.translation.distance(target);

        let move_step = direction * (movement.speed * 2.0) * time.delta().as_secs_f32();

        if distance < move_step.length() {
            transform.translation = target;
            movement.current_target = (movement.current_target + 1) % movement.corners.len();
        } else {
            transform.translation += move_step;
        }
    }
}