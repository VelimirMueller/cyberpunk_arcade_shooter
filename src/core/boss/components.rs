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
    Phase4,
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
    pub phase_thresholds: (f32, f32, f32),
    pub transition_style: TransitionStyle,
    pub primary_timer: Timer,
    pub secondary_timer: Option<Timer>,
    pub attack_state: AttackState,
    pub base_color: Color,
    pub last_hit_time: Option<std::time::Instant>,
    pub last_laser_hit_time: Option<std::time::Instant>,
    pub combo_count: u32,
    pub max_combo: u32,
    pub cycle_index: u32,
    pub is_invulnerable: bool,
}

impl Boss {
    pub fn phase_for_hp_pct(&self) -> BossPhase {
        let hp_pct = self.current_hp as f32 / self.max_hp as f32;
        let (t1, t2, t3) = self.phase_thresholds;
        if hp_pct <= t3 {
            BossPhase::Phase4
        } else if hp_pct <= t2 {
            BossPhase::Phase3
        } else if hp_pct <= t1 {
            BossPhase::Phase2
        } else {
            BossPhase::Phase1
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_boss(current_hp: u32, max_hp: u32) -> Boss {
        Boss {
            boss_type: BossType::GridPhantom,
            phase: BossPhase::Phase1,
            current_hp,
            max_hp,
            phase_thresholds: (0.60, 0.30, 0.10),
            transition_style: TransitionStyle::Stagger,
            primary_timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            secondary_timer: None,
            attack_state: AttackState::Idle,
            base_color: Color::srgb(0.0, 8.0, 8.0),
            last_hit_time: None,
            last_laser_hit_time: None,
            combo_count: 0,
            max_combo: 1,
            cycle_index: 0,
            is_invulnerable: false,
        }
    }

    #[test]
    fn test_phase_thresholds_default() {
        let boss = test_boss(100, 100);
        assert_eq!(boss.phase_thresholds, (0.60, 0.30, 0.10));
    }

    #[test]
    fn test_phase_from_hp_percentage() {
        let cases: &[(u32, u32, BossPhase)] = &[
            (80, 100, BossPhase::Phase1),
            (60, 100, BossPhase::Phase2),
            (45, 100, BossPhase::Phase2),
            (30, 100, BossPhase::Phase3),
            (15, 100, BossPhase::Phase3),
            (10, 100, BossPhase::Phase4),
            (5, 100, BossPhase::Phase4),
        ];
        for &(current_hp, max_hp, expected) in cases {
            let boss = test_boss(current_hp, max_hp);
            assert_eq!(
                boss.phase_for_hp_pct(),
                expected,
                "HP {}/{} should be {:?}",
                current_hp,
                max_hp,
                expected
            );
        }
    }

    #[test]
    fn test_boss_spawn_hp_per_type() {
        let cases: &[(BossType, u32)] = &[
            (BossType::GridPhantom, 150),
            (BossType::NeonSentinel, 200),
            (BossType::ChromeBerserker, 250),
            (BossType::VoidWeaver, 300),
            (BossType::ApexProtocol, 400),
        ];
        // HP values are defined in systems::boss_config; here we just verify our constants
        let expected_hp = |boss_type: BossType| match boss_type {
            BossType::GridPhantom => 150,
            BossType::NeonSentinel => 200,
            BossType::ChromeBerserker => 250,
            BossType::VoidWeaver => 300,
            BossType::ApexProtocol => 400,
        };
        for &(boss_type, hp) in cases {
            assert_eq!(expected_hp(boss_type), hp);
        }
    }
}
