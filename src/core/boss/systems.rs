use bevy::prelude::*;
use crate::app::GameEntity;
use crate::core::boss::components::*;

pub fn boss_type_for_round(round: u32) -> BossType {
    match round {
        1 => BossType::GridPhantom,
        2 => BossType::NeonSentinel,
        3 => BossType::ChromeBerserker,
        4 => BossType::VoidWeaver,
        5 => BossType::ApexProtocol,
        _ => BossType::ApexProtocol,
    }
}

fn boss_config(boss_type: BossType) -> (u32, TransitionStyle, Color, f32) {
    match boss_type {
        BossType::GridPhantom => (150, TransitionStyle::Stagger, Color::srgb(0.0, 8.0, 8.0), 1.0),
        BossType::NeonSentinel => (200, TransitionStyle::Stagger, Color::srgb(8.0, 0.0, 8.0), 1.2),
        BossType::ChromeBerserker => (250, TransitionStyle::RageBurst, Color::srgb(8.0, 4.0, 0.0), 1.4),
        BossType::VoidWeaver => (300, TransitionStyle::Stagger, Color::srgb(4.0, 0.0, 8.0), 1.1),
        BossType::ApexProtocol => (400, TransitionStyle::RageBurst, Color::srgb(8.0, 8.0, 8.0), 1.6),
    }
}

pub fn spawn_boss(commands: &mut Commands, round: u32) {
    let boss_type = boss_type_for_round(round);
    let (max_hp, transition_style, color, size_mult) = boss_config(boss_type);
    let base_size = 50.0;
    let size = base_size * size_mult;

    let primary_timer = match boss_type {
        BossType::GridPhantom => Timer::from_seconds(3.0, TimerMode::Repeating),
        BossType::NeonSentinel => Timer::from_seconds(4.0, TimerMode::Repeating),
        BossType::ChromeBerserker => Timer::from_seconds(2.8, TimerMode::Repeating),
        BossType::VoidWeaver => Timer::from_seconds(5.0, TimerMode::Repeating),
        BossType::ApexProtocol => Timer::from_seconds(3.0, TimerMode::Repeating),
    };

    commands.spawn((
        Sprite { color, custom_size: Some(Vec2::new(size, size)), ..default() },
        Transform::from_xyz(0.0, 150.0, 0.0),
        Boss {
            boss_type,
            phase: BossPhase::Phase1,
            current_hp: max_hp,
            max_hp,
            phase_thresholds: (0.50, 0.20),
            transition_style,
            primary_timer,
            secondary_timer: None,
            attack_state: AttackState::Idle,
            combo_count: 0,
            max_combo: 1,
            cycle_index: 0,
            is_invulnerable: false,
        },
        GameEntity,
    ));
}

pub fn boss_phase_system(
    mut boss_query: Query<&mut Boss>,
    mut screen_shake: ResMut<crate::app::ScreenShake>,
) {
    for mut boss in boss_query.iter_mut() {
        let hp_pct = boss.current_hp as f32 / boss.max_hp as f32;
        let (threshold_2, threshold_3) = boss.phase_thresholds;

        let new_phase = if hp_pct <= threshold_3 {
            BossPhase::Phase3
        } else if hp_pct <= threshold_2 {
            BossPhase::Phase2
        } else {
            BossPhase::Phase1
        };

        if new_phase != boss.phase {
            boss.phase = new_phase;
            match boss.transition_style {
                TransitionStyle::Stagger => {
                    boss.attack_state = AttackState::Recovery(
                        Timer::from_seconds(1.5, TimerMode::Once)
                    );
                },
                TransitionStyle::RageBurst => {
                    screen_shake.intensity = 1.5;
                    screen_shake.duration = 0.5;
                    screen_shake.timer = 0.5;
                },
            }
            if boss.boss_type == BossType::ChromeBerserker {
                boss.max_combo = match new_phase {
                    BossPhase::Phase1 => 1,
                    BossPhase::Phase2 => 3,
                    BossPhase::Phase3 => 3,
                };
            }
        }
    }
}

pub fn boss_idle_movement(
    time: Res<Time>,
    mut boss_query: Query<(&Boss, &mut Transform)>,
) {
    for (boss, mut transform) in boss_query.iter_mut() {
        if matches!(boss.attack_state, AttackState::Idle) {
            let t = time.elapsed_secs();
            transform.translation.y = 150.0 + (t * 1.5).sin() * 30.0;
        }
    }
}
