use crate::env::{CEILING_Y, GROUND_Y, LEFT_BOUND, RIGHT_BOUND};
use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
pub struct BackgroundStar {
    pub velocity: Vec2,
}

pub fn spawn_background_stars(mut commands: Commands) {
    let mut rng = rand::thread_rng();
    let colors = [
        Color::srgba(1.0, 1.0, 1.0, 0.3),
        Color::srgba(0.5, 1.5, 0.5, 0.2),
        Color::srgba(1.5, 0.5, 1.0, 0.15),
    ];

    for _ in 0..40 {
        let x = rng.gen_range(LEFT_BOUND..RIGHT_BOUND);
        let y = rng.gen_range(GROUND_Y..CEILING_Y);
        let size = rng.gen_range(1.0..3.0);
        let speed = rng.gen_range(5.0..15.0);
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let color = colors[rng.gen_range(0..colors.len())];

        commands.spawn((
            Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(x, y, -90.0),
            BackgroundStar {
                velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
            },
        ));
    }
}

pub fn animate_stars(time: Res<Time>, mut query: Query<(&BackgroundStar, &mut Transform)>) {
    let dt = time.delta().as_secs_f32();
    for (star, mut transform) in &mut query {
        transform.translation.x += star.velocity.x * dt;
        transform.translation.y += star.velocity.y * dt;

        // Wrap at arena bounds
        if transform.translation.x > RIGHT_BOUND {
            transform.translation.x = LEFT_BOUND;
        } else if transform.translation.x < LEFT_BOUND {
            transform.translation.x = RIGHT_BOUND;
        }
        if transform.translation.y > CEILING_Y {
            transform.translation.y = GROUND_Y;
        } else if transform.translation.y < GROUND_Y {
            transform.translation.y = CEILING_Y;
        }
    }
}

pub fn draw_background_grid(mut gizmos: Gizmos) {
    let grid_color = Color::srgba(0.0, 1.0, 0.25, 0.04);
    let cell_size = 40.0;

    // Vertical lines
    let mut x = LEFT_BOUND;
    while x <= RIGHT_BOUND {
        gizmos.line_2d(Vec2::new(x, GROUND_Y), Vec2::new(x, CEILING_Y), grid_color);
        x += cell_size;
    }

    // Horizontal lines
    let mut y = GROUND_Y;
    while y <= CEILING_Y {
        gizmos.line_2d(
            Vec2::new(LEFT_BOUND, y),
            Vec2::new(RIGHT_BOUND, y),
            grid_color,
        );
        y += cell_size;
    }
}
