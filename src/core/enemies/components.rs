use bevy::prelude::*;
#[derive(Component)]
pub struct Enemy {
    pub current: u32,
    pub max: u32,
}

