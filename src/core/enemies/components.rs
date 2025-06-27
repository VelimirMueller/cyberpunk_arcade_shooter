use bevy::prelude::*;
#[derive(Component)]
pub struct Enemy {
    pub current: u32,
    pub max: u32,
    pub fire_timer:  Option<Timer>,
    pub last_collision_time: Option<std::time::Instant>,
}

#[derive(Component)]
pub struct EnemyMovement {
    pub corners: Vec<Vec3>,    // The 4 corner positions
    pub current_target: usize, // Index of the next target
    pub speed: f32,
}