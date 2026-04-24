//! Reproducer: simulates round-1 boss death + round-2 boss spawn + round-2 combat systems.
//! Intent: find which system panics after the first boss is killed.
mod helpers;

use bevy::prelude::*;
use cyberpunk_rpg::app::{GameData, GameEntity, ScreenShake};
use cyberpunk_rpg::core::boss::components::*;
use cyberpunk_rpg::core::boss::systems::{
    boss_attack_system, boss_death_check_system, boss_death_system, boss_idle_movement,
    boss_phase_system, boss_projectile_system, boss_visual_system, death_explosion_system,
    eliminated_text_system, hazard_lifetime_system, hazard_zone_system, phase_flash_system,
    phase_name_text_system, phase_shift_text_system, phase_transition_system, spawn_boss,
};
use cyberpunk_rpg::core::player::components::{Player, PlayerRotationTracker};
use cyberpunk_rpg::systems::audio::SoundEvent;
use cyberpunk_rpg::systems::collision::DeathEvent;
use cyberpunk_rpg::systems::particles::{
    AfterimageTimer, AmbientParticleTimer, animate_shatter, animate_shockwave, handle_death_events,
    setup_shockwave_assets,
};
use cyberpunk_rpg::systems::powerups::{
    laser_charge_orb_system, laser_charge_particle_system, laser_impact_system,
    laser_stream_particle_system, laser_system, powerup_shockwave_system,
};

fn repro_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin::default());

    app.init_resource::<GameData>()
        .init_resource::<ScreenShake>()
        .init_resource::<AfterimageTimer>()
        .init_resource::<AmbientParticleTimer>()
        .add_event::<SoundEvent>()
        .add_event::<DeathEvent>();

    app.init_asset::<Mesh>().init_asset::<ColorMaterial>();

    app.add_systems(Startup, setup_shockwave_assets);

    // Bulk of RoundActive systems, no state gating.
    app.add_systems(
        Update,
        (
            boss_attack_system,
            boss_phase_system,
            boss_idle_movement,
            hazard_lifetime_system,
            boss_projectile_system,
            hazard_zone_system,
        ),
    );
    app.add_systems(
        Update,
        (
            boss_death_check_system,
            boss_death_system,
            handle_death_events,
            death_explosion_system,
            eliminated_text_system,
            phase_name_text_system,
            phase_flash_system,
            phase_shift_text_system,
            boss_visual_system,
            phase_transition_system,
            animate_shatter,
            animate_shockwave,
        ),
    );
    app.add_systems(
        Update,
        (
            laser_system,
            powerup_shockwave_system,
            laser_charge_particle_system,
            laser_charge_orb_system,
            laser_stream_particle_system,
            laser_impact_system,
        ),
    );

    app
}

fn spawn_player(app: &mut App) {
    app.world_mut().spawn((
        Player {
            current: 100,
            max: 100,
            last_collision_time: None,
            energy: 100,
            last_shot_time: None,
        },
        PlayerRotationTracker {
            last_angle_index: 0,
        },
        GameEntity,
        Transform::from_xyz(-250.0, 0.0, 0.0),
        GlobalTransform::default(),
        Sprite {
            color: Color::srgb(1.2, 2.8, 1.2),
            custom_size: Some(Vec2::new(50.0, 50.0)),
            ..default()
        },
    ));
}

#[test]
fn repro_round1_death_then_round2_boss() {
    let mut app = repro_app();
    app.update(); // startup

    spawn_player(&mut app);
    let boss1 = helpers::spawn_test_boss(&mut app, BossType::GridPhantom, 1);
    app.update();

    // Kill round 1 boss
    app.world_mut().get_mut::<Boss>(boss1).unwrap().current_hp = 0;

    for _ in 0..120 {
        std::thread::sleep(std::time::Duration::from_millis(8));
        app.update();
    }

    // Now spawn round-2 boss directly via the real helper
    app.world_mut()
        .commands()
        .queue(|world: &mut World| {
            let mut commands_state =
                bevy::ecs::system::SystemState::<Commands>::new(world);
            let mut commands = commands_state.get_mut(world);
            spawn_boss(&mut commands, 2);
            commands_state.apply(world);
        });
    app.update();

    // Run round 2 combat for 5 seconds of wall clock
    for i in 0..600 {
        std::thread::sleep(std::time::Duration::from_millis(8));
        app.update();
        if i % 100 == 0 {
            let bosses: Vec<(u32, u32, bool)> = app
                .world_mut()
                .query::<&Boss>()
                .iter(app.world())
                .map(|b| (b.current_hp, b.max_hp, b.is_invulnerable))
                .collect();
            println!("tick {i}: bosses = {:?}", bosses);
        }
    }
}
