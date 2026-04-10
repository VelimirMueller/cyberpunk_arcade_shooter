use crate::utils::time_compat::Instant;
use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub current: u32,
    pub max: u32,
    pub energy: u32,
    pub last_collision_time: Option<Instant>,
    pub last_shot_time: Option<Instant>,
}

#[derive(Component)]
pub struct PlayerRotationTracker {
    #[allow(dead_code)]
    pub last_angle_index: i32, // 0, 1, 2, 3 -> representing 0°, 90°, 180°, 270°
}

#[derive(Component)]
pub struct PlayerParticle;
