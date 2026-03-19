use bevy::prelude::*;
use crate::core::player::systems::spawn_player;
use crate::core::boss::components::{Boss, DashTrail, HazardZone, BeamSweep, ChargeTelegraph, BossProjectile};
use crate::core::player::components::{Player, PlayerParticle};
use crate::systems::combat::EnemyParticle;
use crate::app::{GameEntity, ScoreText, WaveText};
use crate::data::game_state::GameState;
use crate::core::world::barriers::components::Barrier;
use crate::core::world::barriers::systems::spawn_barriers;

#[derive(Component)]
pub struct AnimatedGameOverText;

pub(crate) fn game_over_system(mut commands: Commands) {
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("GAME OVER"),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            font_size: 97.0,
            ..default()
        },
        TextShadow::default(),
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(35.0),
            top: Val::Percent(45.0),
            ..default()
        },
        AnimatedGameOverText,
        GameEntity
    ));
}

pub(crate) fn game_won_system(mut commands: Commands) {
    commands.spawn((
        // Accepts a `String` or any type that converts into a `String`, such as `&str`
        Text::new("GAME WON"),
        TextFont {
            // This font is loaded and will be used instead of the default font.
            font_size: 97.0,
            ..default()
        },
        TextShadow::default(),
        // Set the justification of the Text
        TextLayout::new_with_justify(JustifyText::Center),
        // Set the style of the Node itself.
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(35.0),
            top: Val::Percent(45.0),
            ..default()
        },
        AnimatedGameOverText,
        GameEntity
    ));
}

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
    ui_query: Query<Entity, Or<(With<ScoreText>, With<WaveText>)>>,
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
        for entity in &ui_query {
            commands.entity(entity).despawn();
        }

        // Spawn fresh game entities
        spawn_player(commands.reborrow());
        spawn_barriers(commands.reborrow());

        // Respawn score UI
        commands.spawn((
            Text::new("Score: 0"),
            TextFont { font_size: 17.0, ..default() },
            TextShadow::default(),
            TextLayout::new_with_justify(JustifyText::Center),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(55.0),
                left: Val::Px(10.0),
                ..default()
            },
            ScoreText,
        ));

        commands.spawn((
            Text::new("Round: 1"),
            TextFont { font_size: 17.0, ..default() },
            TextShadow::default(),
            TextLayout::new_with_justify(JustifyText::Center),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(75.0),
                left: Val::Px(10.0),
                ..default()
            },
            WaveText,
        ));

        next_state.set(GameState::RoundAnnounce);
    }
}

pub fn despawn_game_over_text(
    mut commands: Commands,
    query: Query<Entity, With<AnimatedGameOverText>>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
