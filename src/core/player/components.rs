use bevy::prelude::*;
#[derive(Component)]
pub struct Player {
    pub current: u32,
    pub max: u32,
    pub last_collision_time: Option<std::time::Instant>,
}