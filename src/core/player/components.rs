use crate::utils::time_compat::Instant;
use bevy::prelude::*;

#[derive(Component)]
pub struct Player {
    pub current: u32,
    pub max: u32,
    pub energy: u32,
    pub max_energy: u32,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_default_shape_compiles() {
        // Sanity that all fields present and default-constructible via struct literal
        let p = Player {
            current: 100,
            max: 100,
            energy: 50,
            max_energy: 100,
            last_collision_time: None,
            last_shot_time: None,
        };
        assert_eq!(p.max_energy, 100);
        assert!(p.energy < p.max_energy);
    }
}
