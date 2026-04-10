use crate::app::GameEntity;
use crate::core::boss::components::{
    BeamSweep, Boss, BossProjectile, ChargeTelegraph, DashTrail, HazardZone,
};
use crate::core::player::components::{Player, PlayerParticle};
use crate::core::player::systems::spawn_player;
use crate::core::world::barriers::components::Barrier;
use crate::core::world::barriers::systems::spawn_barriers;
use crate::data::game_state::GameState;
use crate::systems::combat::EnemyParticle;
use bevy::prelude::*;

#[allow(clippy::too_many_arguments)]
pub fn restart_listener(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
    game_entity_query: Query<Entity, With<GameEntity>>,
    boss_query: Query<Entity, With<Boss>>,
    player_query: Query<Entity, With<Player>>,
    enemy_particle_query: Query<Entity, With<EnemyParticle>>,
    player_particle_query: Query<Entity, With<PlayerParticle>>,
    barrier_query: Query<Entity, With<Barrier>>,
    dash_trail_query: Query<Entity, With<DashTrail>>,
    hazard_zone_query: Query<Entity, With<HazardZone>>,
    beam_sweep_query: Query<Entity, With<BeamSweep>>,
    charge_telegraph_query: Query<Entity, With<ChargeTelegraph>>,
    boss_projectile_query: Query<Entity, With<BossProjectile>>,
    mut commands: Commands,
    mut game_data: ResMut<crate::app::GameData>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        // Update high score if current score is higher
        if game_data.score > game_data.high_score {
            game_data.high_score = game_data.score;
        }

        // Reset game data for new game
        game_data.score = 0;
        game_data.round = 1;
        game_data.total_play_time = 0.0;
        game_data.enemies_killed = 0;

        // Despawn all game entities
        for entity in &game_entity_query {
            commands.entity(entity).despawn();
        }
        for entity in &boss_query {
            commands.entity(entity).despawn();
        }
        for entity in &player_query {
            commands.entity(entity).despawn();
        }
        for entity in &enemy_particle_query {
            commands.entity(entity).despawn();
        }
        for entity in &player_particle_query {
            commands.entity(entity).despawn();
        }
        for entity in &barrier_query {
            commands.entity(entity).despawn();
        }
        for entity in &dash_trail_query {
            commands.entity(entity).despawn();
        }
        for entity in &hazard_zone_query {
            commands.entity(entity).despawn();
        }
        for entity in &beam_sweep_query {
            commands.entity(entity).despawn();
        }
        for entity in &charge_telegraph_query {
            commands.entity(entity).despawn();
        }
        for entity in &boss_projectile_query {
            commands.entity(entity).despawn();
        }

        // Spawn fresh game entities
        spawn_player(commands.reborrow());
        spawn_barriers(commands.reborrow());

        next_state.set(GameState::RoundAnnounce);
    }
}
