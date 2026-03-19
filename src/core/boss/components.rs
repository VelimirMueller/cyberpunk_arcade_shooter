use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BossType {
    GridPhantom,
    NeonSentinel,
    ChromeBerserker,
    VoidWeaver,
    ApexProtocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BossPhase {
    #[default]
    Phase1,
    Phase2,
    Phase3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionStyle {
    Stagger,
    RageBurst,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum AttackState {
    #[default]
    Idle,
    WindUp(Timer),
    Attacking,
    Recovery(Timer),
    Dashing { target: Vec2, speed: f32 },
    Charging { target: Vec2, speed: f32 },
}

#[derive(Component)]
pub struct Boss {
    pub boss_type: BossType,
    pub phase: BossPhase,
    pub current_hp: u32,
    pub max_hp: u32,
    pub phase_thresholds: (f32, f32),
    pub transition_style: TransitionStyle,
    pub primary_timer: Timer,
    pub secondary_timer: Option<Timer>,
    pub attack_state: AttackState,
    pub combo_count: u32,
    pub max_combo: u32,
    pub cycle_index: u32,
    pub is_invulnerable: bool,
}

#[derive(Component)]
pub struct DashTrail {
    pub lifetime: Timer,
    pub damage: u32,
}

#[derive(Component)]
pub struct BeamSweep {
    pub angle: f32,
    pub arc_width: f32,
    pub rotation_speed: f32,
    pub damage: u32,
}

#[derive(Component)]
pub struct HazardZone {
    pub radius: f32,
    pub lifetime: Timer,
    pub drift_velocity: Option<Vec2>,
    pub explodes: bool,
    pub explosion_timer: Option<Timer>,
    pub damage: u32,
}

#[derive(Component)]
pub struct ChargeTelegraph {
    pub start: Vec2,
    pub end: Vec2,
    pub lifetime: Timer,
}

#[derive(Component)]
pub struct BossProjectile {
    pub velocity: Vec2,
    pub damage: u32,
}

#[derive(Component)]
pub struct PhaseTransitionEffect {
    pub timer: Timer,
    pub style: TransitionStyle,
}
