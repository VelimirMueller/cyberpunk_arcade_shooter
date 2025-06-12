use bevy::prelude::*;
use crate::core::enemies::components::Enemy;
use crate::env::{ROTATE_SPEED, TIME_STEP};

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

    for color in enemy_colors {
        commands.spawn((
            Enemy { current: 100, max: 100 },
            Sprite {
                color,
                custom_size: Some(Vec2::new(50.0, 50.0 * (count+0.85)/1.25)),
                ..default()
            },
            Transform::from_xyz((22.0) * count/2.0, (10.0 ) * count, (4.0) * count),
            GlobalTransform::default(),
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
            transform.rotate_z(ROTATE_SPEED * TIME_STEP);
        } else {
            transform.rotate_z(-ROTATE_SPEED * TIME_STEP/2.0);
        }
    }
}