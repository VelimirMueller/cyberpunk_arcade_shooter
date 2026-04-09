use bevy::prelude::*;
use cyberpunk_rpg::core::boss::components::*;
use cyberpunk_rpg::core::player::components::Player;
use cyberpunk_rpg::app::GameEntity;

pub fn spawn_test_boss(app: &mut App, boss_type: BossType, hp: u32) -> Entity {
    let color = Color::srgb(1.0, 1.0, 1.0);
    app.world_mut().spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 150.0, 0.0),
        Boss {
            boss_type,
            phase: BossPhase::Phase1,
            current_hp: hp,
            max_hp: hp,
            phase_thresholds: (0.60, 0.30, 0.10),
            transition_style: TransitionStyle::Stagger,
            primary_timer: Timer::from_seconds(3.0, TimerMode::Repeating),
            secondary_timer: None,
            attack_state: AttackState::Idle,
            base_color: color,
            last_hit_time: None,
            last_laser_hit_time: None,
            combo_count: 0,
            max_combo: 1,
            cycle_index: 0,
            is_invulnerable: false,
        },
        GameEntity,
    )).id()
}

pub fn spawn_test_player(app: &mut App, position: Vec2) -> Entity {
    app.world_mut().spawn((
        Sprite {
            color: Color::srgb(1.0, 1.0, 1.0),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, 0.0),
        Player {
            current: 100,
            max: 100,
            energy: 100,
            last_collision_time: None,
            last_shot_time: None,
        },
        GameEntity,
    )).id()
}
