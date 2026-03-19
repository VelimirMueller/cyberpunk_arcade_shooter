use bevy::prelude::*;
use crate::app::{GameEntity, ScreenShake};
use crate::core::boss::components::Boss;
use crate::core::player::components::Player;
use crate::systems::audio::{SoundEvent, SoundEffect};
use crate::systems::collision::collide;
use crate::systems::combat::EnemyParticle;
use crate::core::boss::components::{BossProjectile, DashTrail, HazardZone, ChargeTelegraph};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpKind {
    Shockwave,
    Laser,
}

#[derive(Component)]
pub struct PowerUp {
    pub kind: PowerUpKind,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct LaserActive {
    pub timer: Timer,
    pub sound_timer: Timer,
}

#[derive(Component)]
pub struct LaserBeam;

#[derive(Resource)]
pub struct PowerUpTimer {
    pub timer: Timer,
}

#[derive(Component)]
pub struct PowerUpShockwave {
    pub timer: Timer,
}
