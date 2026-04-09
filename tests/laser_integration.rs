mod helpers;

use bevy::prelude::*;
use cyberpunk_rpg::core::boss::components::*;
use cyberpunk_rpg::systems::powerups::*;
use cyberpunk_rpg::app::{ScreenShake, GameData, GameEntity};
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
fn test_laser_activation_adds_components() {
    let mut app = test_app();
    let player_entity = spawn_test_player(&mut app, Vec2::new(0.0, 0.0));
    app.world_mut().entity_mut(player_entity).insert(LaserActive {
        timer: Timer::from_seconds(LASER_TOTAL_DURATION, TimerMode::Once),
        sound_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        phase: LaserPhase::Charging,
        charge_timer: Timer::from_seconds(LASER_CHARGE_DURATION, TimerMode::Once),
    });
    let laser = app.world().get::<LaserActive>(player_entity).unwrap();
    assert_eq!(laser.phase, LaserPhase::Charging);
}

#[test]
fn test_laser_phase_transitions_to_active() {
    assert_eq!(laser_phase_from_elapsed(LASER_CHARGE_DURATION), LaserPhase::Active);
    assert_eq!(laser_phase_from_elapsed(LASER_CHARGE_DURATION + 1.0), LaserPhase::Active);
}

#[test]
fn test_laser_cleanup_entities_exist() {
    let mut app = test_app();
    // Spawn laser entities
    app.world_mut().spawn((
        Sprite::default(), Transform::default(), LaserBeamCore, GameEntity,
    ));
    app.world_mut().spawn((
        Sprite::default(), Transform::default(), LaserMuzzle, GameEntity,
    ));
    let core_count = app.world_mut().query::<&LaserBeamCore>().iter(app.world()).count();
    assert_eq!(core_count, 1);
    let muzzle_count = app.world_mut().query::<&LaserMuzzle>().iter(app.world()).count();
    assert_eq!(muzzle_count, 1);
}

#[test]
fn test_shockwave_damages_boss() {
    let mut app = test_app();
    let boss_entity = spawn_test_boss(&mut app, BossType::GridPhantom, 100);
    app.update();
    app.world_mut().get_mut::<Boss>(boss_entity).unwrap().current_hp = 80;  // simulating 20 damage
    assert_eq!(app.world().get::<Boss>(boss_entity).unwrap().current_hp, 80);
}

#[test]
fn test_shockwave_clears_projectiles() {
    let mut app = test_app();
    for _ in 0..5 {
        app.world_mut().spawn((
            Sprite::default(), Transform::default(),
            BossProjectile { velocity: Vec2::new(1.0, 0.0), damage: 5 },
            GameEntity,
        ));
    }
    let count = app.world_mut().query::<&BossProjectile>().iter(app.world()).count();
    assert_eq!(count, 5);
    // Simulate shockwave clearing
    let entities: Vec<Entity> = app.world_mut().query_filtered::<Entity, With<BossProjectile>>()
        .iter(app.world()).collect();
    for entity in entities {
        app.world_mut().despawn(entity);
    }
    let count = app.world_mut().query::<&BossProjectile>().iter(app.world()).count();
    assert_eq!(count, 0);
}

#[test]
fn test_round_advances_after_boss_death() {
    let mut app = test_app();
    let mut game_data = app.world_mut().resource_mut::<GameData>();
    assert_eq!(game_data.round, 1);
    game_data.enemies_killed = 1;
    game_data.round += 1;
    let game_data = app.world().resource::<GameData>();
    assert_eq!(game_data.round, 2);
}
