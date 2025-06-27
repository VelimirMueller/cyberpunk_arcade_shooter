use bevy::prelude::*;
use crate::app::GameEntity;
use crate::core::player::components::{Player, PlayerParticle, PlayerRotationTracker};
use crate::env::{
    GROUND_Y,
    CEILING_Y,
    LEFT_BOUND,
    RIGHT_BOUND,
    MOVE_SPEED,
    ROTATE_SPEED,
    TIME_STEP
};

fn add_energy(player: &mut Player) {
    player.energy += 1;
}
pub(crate) fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform), With<Player>>,
) {
    for (mut player, mut transform) in &mut query {
        let direction = get_input_direction(&keyboard_input);

        if direction != Vec3::ZERO {
            // Add energy if moving
            add_energy(&mut player); // or any value you prefer

            apply_movement(&mut transform, direction);
            apply_rotation(&mut transform, &keyboard_input);

            if keyboard_input.pressed(KeyCode::Space) {
                println!("Keyboard input: Space");
            }
        }
    }
}

fn get_input_direction(input: &ButtonInput<KeyCode>) -> Vec3 {
    let mut direction = Vec3::ZERO;

    if input.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    direction
}

fn apply_movement(transform: &mut Transform, direction: Vec3) -> () {
    with_boundaries(transform, direction);
}

fn with_boundaries(transform: &mut Transform, direction: Vec3) -> () {

    let mut new_translation = transform.translation + direction.normalize_or_zero() * MOVE_SPEED * TIME_STEP;
    if new_translation.y < GROUND_Y {
        new_translation.y = GROUND_Y;
    }

    if new_translation.y > CEILING_Y {
        new_translation.y = CEILING_Y;
    }

    if new_translation.x < LEFT_BOUND {
        new_translation.x = LEFT_BOUND;
    }

    if new_translation.x > RIGHT_BOUND {
        new_translation.x = RIGHT_BOUND;
    }

    transform.translation = new_translation;
}

fn apply_rotation(transform: &mut Transform, input: &ButtonInput<KeyCode>) -> () {
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::KeyA) {
        transform.rotate_z(ROTATE_SPEED * TIME_STEP);
    }

    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::KeyS) {
        transform.rotate_z(-ROTATE_SPEED * TIME_STEP);
    }
}

pub(crate) fn spawn_player(
    mut commands: Commands,
) -> () {
    commands.spawn((
        Player { current: 100, max: 100, last_collision_time: None, energy: 100 },
        PlayerRotationTracker { last_angle_index: 0 },
        GameEntity,
        Transform::from_xyz(-250.0, 0.0, 0.0),
        GlobalTransform::default(),
        Sprite {
            color: Color::srgb(1.2, 2.8, 1.2),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
    ));
}