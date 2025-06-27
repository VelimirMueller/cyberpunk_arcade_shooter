use bevy::prelude::*;
#[derive(Component)]
pub struct Player {
    pub current: u32,
    pub max: u32,
    pub energy: u32,
    pub last_collision_time: Option<std::time::Instant>,
}

#[derive(Component)]
pub(crate) struct PlayerRotationTracker {
    pub(crate) last_angle_index: i32, // 0, 1, 2, 3 -> representing 0°, 90°, 180°, 270°
}

#[derive(Component)]
pub(crate) struct PlayerParticle;