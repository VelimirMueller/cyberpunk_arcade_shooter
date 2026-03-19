use bevy::prelude::*;
use crate::app::GameData;
use crate::core::boss::components::{Boss, DashTrail, HazardZone, BossProjectile, ChargeTelegraph};
use crate::core::boss::systems::spawn_boss;
use crate::core::player::components::Player;
use crate::data::game_state::GameState;
use crate::systems::combat::EnemyParticle;

// ============ Round Announcement ============

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnouncePhase {
    ThreatLine,
    RoundNumber,
    BossName,
    FlavorText,
    Hold,
    FadeOut,
}

#[derive(Resource)]
pub struct RoundTimer {
    pub elapsed: f32,
    pub phase: AnnouncePhase,
    pub duration: f32,
}

impl RoundTimer {
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            phase: AnnouncePhase::ThreatLine,
            duration: 2.5,
        }
    }
}

pub fn start_round_announce(mut commands: Commands) {
    commands.insert_resource(RoundTimer::new());
}

pub fn round_announce_system(
    time: Res<Time>,
    mut round_timer: ResMut<RoundTimer>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    game_data: Res<GameData>,
) {
    round_timer.elapsed += time.delta().as_secs_f32();

    // Update phase based on elapsed time
    round_timer.phase = if round_timer.elapsed < 0.4 {
        AnnouncePhase::ThreatLine
    } else if round_timer.elapsed < 0.8 {
        AnnouncePhase::RoundNumber
    } else if round_timer.elapsed < 1.2 {
        AnnouncePhase::BossName
    } else if round_timer.elapsed < 2.2 {
        AnnouncePhase::FlavorText
    } else if round_timer.elapsed < 2.5 {
        AnnouncePhase::FadeOut
    } else {
        AnnouncePhase::Hold
    };

    if round_timer.elapsed >= round_timer.duration {
        // Spawn boss and transition to active
        spawn_boss(&mut commands, game_data.round);
        commands.remove_resource::<RoundTimer>();
        next_state.set(GameState::RoundActive);
    }
}

// ============ Score Tally ============

#[derive(Resource)]
pub struct ScoreTallyTimer {
    pub elapsed: f32,
    pub duration: f32,
    pub text_spawned: bool,
}

impl ScoreTallyTimer {
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            duration: 1.0,
            text_spawned: false,
        }
    }
}

#[derive(Component)]
pub struct RoundClearText;

pub fn boss_defeated_check(
    boss_query: Query<&Boss>,
    tally_timer: Option<Res<ScoreTallyTimer>>,
    mut commands: Commands,
) {
    // Guard: don't re-insert if timer already exists
    if tally_timer.is_some() {
        return;
    }

    for boss in boss_query.iter() {
        if boss.current_hp == 0 {
            commands.insert_resource(ScoreTallyTimer::new());
            return;
        }
    }
}

pub fn score_tally_system(
    time: Res<Time>,
    tally_timer: Option<ResMut<ScoreTallyTimer>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameData>,
    mut player_query: Query<&mut Player>,
    boss_query: Query<Entity, With<Boss>>,
    dash_trail_query: Query<Entity, With<DashTrail>>,
    hazard_query: Query<Entity, With<HazardZone>>,
    projectile_query: Query<Entity, With<BossProjectile>>,
    telegraph_query: Query<Entity, With<ChargeTelegraph>>,
    enemy_particle_query: Query<Entity, With<EnemyParticle>>,
    round_clear_query: Query<Entity, With<RoundClearText>>,
) {
    let Some(mut tally_timer) = tally_timer else {
        return;
    };

    tally_timer.elapsed += time.delta().as_secs_f32();

    // Spawn "ROUND CLEAR" text once
    if !tally_timer.text_spawned && round_clear_query.is_empty() {
        tally_timer.text_spawned = true;
        commands.spawn((
            Text::new("ROUND CLEAR"),
            TextFont {
                font_size: 64.0,
                ..default()
            },
            TextColor(Color::srgb(0.0, 8.0, 8.0)),
            TextShadow::default(),
            TextLayout::new_with_justify(JustifyText::Center),
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(25.0),
                top: Val::Percent(40.0),
                width: Val::Percent(50.0),
                ..default()
            },
            RoundClearText,
        ));
    }

    if tally_timer.elapsed >= tally_timer.duration {
        // Despawn boss
        for entity in boss_query.iter() {
            commands.entity(entity).despawn();
        }
        // Clean arena: despawn all boss-related entities
        for entity in dash_trail_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in hazard_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in projectile_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in telegraph_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in enemy_particle_query.iter() {
            commands.entity(entity).despawn();
        }

        // Restore player
        for mut player in player_query.iter_mut() {
            let half_max = player.max / 2;
            if player.current < half_max {
                player.current = half_max;
            }
            player.energy = 100;
        }

        // Increment round, reset kills
        game_data.round += 1;
        game_data.enemies_killed = 0;

        // Remove tally timer
        commands.remove_resource::<ScoreTallyTimer>();

        // Transition
        if game_data.round > game_data.total_rounds {
            next_state.set(GameState::Won);
        } else {
            next_state.set(GameState::RoundAnnounce);
        }
    }
}

pub fn despawn_round_clear(
    mut commands: Commands,
    query: Query<Entity, With<RoundClearText>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
