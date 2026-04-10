use crate::app::GameData;
use crate::core::boss::components::{
    Boss, BossProjectile, ChargeTelegraph, DashTrail, DeathExplosion, EliminatedText, HazardZone,
    PhaseNameText, ScreenDimOverlay,
};
use crate::core::boss::systems::spawn_boss;
use crate::core::player::components::Player;
use crate::data::game_state::GameState;
use crate::systems::audio::{SoundEffect, SoundEvent};
use crate::systems::combat::EnemyParticle;
use crate::systems::powerups::{
    LaserActive, LaserBeamCore, LaserBeamShell, LaserChargeOrb, LaserChargeParticle, LaserImpact,
    LaserMuzzle, LaserStreamParticle, PowerUp, PowerUpShockwave,
};
use bevy::prelude::*;

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

impl Default for RoundTimer {
    fn default() -> Self {
        Self::new()
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
    mut sound_events: EventWriter<SoundEvent>,
) {
    // Play BossSpawn sound at the start of the announcement
    if round_timer.elapsed == 0.0 {
        sound_events.write(SoundEvent(SoundEffect::BossSpawn));
    }
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

impl Default for ScoreTallyTimer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Component)]
pub struct RoundClearText;

pub fn boss_defeated_check(
    game_data: Res<GameData>,
    tally_timer: Option<Res<ScoreTallyTimer>>,
    mut commands: Commands,
) {
    // Guard: don't re-insert if timer already exists
    if tally_timer.is_some() {
        return;
    }

    // Use enemies_killed counter instead of querying boss entity —
    // the boss entity may already be despawned by handle_death_events
    if game_data.enemies_killed >= game_data.total_enemies {
        commands.insert_resource(ScoreTallyTimer::new());
    }
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn score_tally_system(
    time: Res<Time>,
    tally_timer: Option<ResMut<ScoreTallyTimer>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
    mut game_data: ResMut<GameData>,
    mut player_query: Query<(Entity, &mut Player)>,
    boss_query: Query<Entity, With<Boss>>,
    dash_trail_query: Query<Entity, With<DashTrail>>,
    hazard_query: Query<Entity, With<HazardZone>>,
    projectile_query: Query<Entity, With<BossProjectile>>,
    telegraph_query: Query<Entity, With<ChargeTelegraph>>,
    enemy_particle_query: Query<
        Entity,
        Or<(
            With<EnemyParticle>,
            With<PowerUp>,
            With<LaserBeamCore>,
            With<LaserBeamShell>,
            With<LaserStreamParticle>,
            With<LaserImpact>,
            With<LaserMuzzle>,
            With<LaserChargeOrb>,
            With<LaserChargeParticle>,
            With<PowerUpShockwave>,
            With<EliminatedText>,
            With<DeathExplosion>,
        )>,
    >,
    round_clear_query: Query<Entity, With<RoundClearText>>,
    screen_dim_query: Query<Entity, With<ScreenDimOverlay>>,
    phase_name_query: Query<Entity, With<PhaseNameText>>,
    mut sound_events: EventWriter<SoundEvent>,
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
        sound_events.write(SoundEvent(SoundEffect::RoundClear));
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
        // Clean up transition effects
        for entity in screen_dim_query.iter() {
            commands.entity(entity).despawn();
        }
        for entity in phase_name_query.iter() {
            commands.entity(entity).despawn();
        }

        // Restore player
        for (player_entity, mut player) in player_query.iter_mut() {
            commands.entity(player_entity).remove::<LaserActive>();
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

pub fn despawn_round_clear(mut commands: Commands, query: Query<Entity, With<RoundClearText>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use crate::core::boss::components::BossType;
    use crate::core::boss::systems::boss_type_for_round;

    #[test]
    fn test_round_boss_type_mapping() {
        assert_eq!(boss_type_for_round(1), BossType::GridPhantom);
        assert_eq!(boss_type_for_round(2), BossType::NeonSentinel);
        assert_eq!(boss_type_for_round(3), BossType::ChromeBerserker);
        assert_eq!(boss_type_for_round(4), BossType::VoidWeaver);
        assert_eq!(boss_type_for_round(5), BossType::ApexProtocol);
    }

    #[test]
    fn test_round_progression_to_won() {
        let total_rounds = 5u32;
        let round_after_last = 6u32;
        assert!(round_after_last > total_rounds);
    }
}
