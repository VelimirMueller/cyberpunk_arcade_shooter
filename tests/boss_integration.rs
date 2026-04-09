mod helpers;

use bevy::prelude::*;
use cyberpunk_rpg::app::{GameData, ScreenShake};
use cyberpunk_rpg::core::boss::components::*;
use cyberpunk_rpg::core::boss::systems::boss_phase_system;
use cyberpunk_rpg::systems::audio::SoundEvent;
use cyberpunk_rpg::systems::collision::DeathEvent;
use helpers::*;

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ScreenShake>();
    app.init_resource::<GameData>();
    app.add_event::<SoundEvent>();
    app.add_event::<DeathEvent>();
    app
}

#[test]
fn test_boss_takes_damage() {
    let mut app = test_app();
    let boss_entity = spawn_test_boss(&mut app, BossType::GridPhantom, 100);
    app.update();
    app.world_mut()
        .get_mut::<Boss>(boss_entity)
        .unwrap()
        .current_hp = 90;
    assert_eq!(app.world().get::<Boss>(boss_entity).unwrap().current_hp, 90);
}

#[test]
fn test_boss_phase_transition_at_60_percent() {
    let mut app = test_app();
    app.add_systems(Update, boss_phase_system);
    let boss_entity = spawn_test_boss(&mut app, BossType::GridPhantom, 100);
    app.update();
    app.world_mut()
        .get_mut::<Boss>(boss_entity)
        .unwrap()
        .current_hp = 60;
    app.update();
    assert!(
        app.world()
            .get::<PhaseTransitionSequence>(boss_entity)
            .is_some()
    );
    let transition = app
        .world()
        .get::<PhaseTransitionSequence>(boss_entity)
        .unwrap();
    assert_eq!(transition.target_phase, BossPhase::Phase2);
}

#[test]
fn test_boss_phase_transition_at_30_percent() {
    let mut app = test_app();
    app.add_systems(Update, boss_phase_system);
    let boss_entity = spawn_test_boss(&mut app, BossType::GridPhantom, 100);
    app.update();
    app.world_mut()
        .get_mut::<Boss>(boss_entity)
        .unwrap()
        .current_hp = 30;
    app.update();
    let transition = app
        .world()
        .get::<PhaseTransitionSequence>(boss_entity)
        .unwrap();
    assert_eq!(transition.target_phase, BossPhase::Phase3);
}

#[test]
fn test_boss_enters_desperation_at_10_percent() {
    let mut app = test_app();
    app.add_systems(Update, boss_phase_system);
    let boss_entity = spawn_test_boss(&mut app, BossType::GridPhantom, 100);
    app.update();
    app.world_mut()
        .get_mut::<Boss>(boss_entity)
        .unwrap()
        .current_hp = 10;
    app.update();
    let transition = app
        .world()
        .get::<PhaseTransitionSequence>(boss_entity)
        .unwrap();
    assert_eq!(transition.target_phase, BossPhase::Phase4);
}

#[test]
fn test_boss_invulnerable_during_transition() {
    let mut app = test_app();
    app.add_systems(Update, boss_phase_system);
    let boss_entity = spawn_test_boss(&mut app, BossType::GridPhantom, 100);
    app.update();
    app.world_mut()
        .get_mut::<Boss>(boss_entity)
        .unwrap()
        .current_hp = 60;
    app.update();
    let boss = app.world().get::<Boss>(boss_entity).unwrap();
    assert!(boss.is_invulnerable);
}

#[test]
fn test_boss_attack_state_resets_on_transition() {
    let mut app = test_app();
    app.add_systems(Update, boss_phase_system);
    let boss_entity = spawn_test_boss(&mut app, BossType::GridPhantom, 100);
    app.update();
    {
        let mut boss = app.world_mut().get_mut::<Boss>(boss_entity).unwrap();
        boss.attack_state = AttackState::Attacking;
        boss.current_hp = 60;
    }
    app.update();
    let boss = app.world().get::<Boss>(boss_entity).unwrap();
    assert_eq!(boss.attack_state, AttackState::Idle);
}
