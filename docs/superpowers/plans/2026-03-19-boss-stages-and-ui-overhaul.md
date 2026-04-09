# Boss Stages & UI Overhaul Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 5-round boss gauntlet with souls-like phase mechanics and cinematic HUD to Cyberpunk Bloom Cube.

**Architecture:** Monolithic `BossType` enum with pattern-matched AI. Each boss has 3 phases triggered at HP thresholds (50%, 20%). New `RoundAnnounce`/`RoundActive` game states replace `Playing`. UI rebuilt with structured Bevy UI nodes and layered text shadow glow.

**Tech Stack:** Rust, Bevy 0.16.1, Kira 0.9 (audio synthesis)

**Spec:** `docs/superpowers/specs/2026-03-19-boss-stages-and-ui-overhaul-design.md`

---

## File Structure

### New Files
| File | Responsibility |
|------|---------------|
| `src/core/boss/mod.rs` | Module declaration (components, systems, attacks) |
| `src/core/boss/components.rs` | Boss, BossType, BossPhase, AttackState, TransitionStyle, attack entity components (DashTrail, BeamSweep, HazardZone, ChargeTelegraph, BossProjectile) |
| `src/core/boss/systems.rs` | Boss AI dispatch, phase transition detection, spawn_boss(), boss movement per type |
| `src/core/boss/attacks.rs` | Per-boss attack pattern functions (phantom_attack, sentinel_attack, berserker_attack, weaver_attack, apex_attack) |
| `src/systems/round.rs` | Round state management, announcement system, score tally, between-round restoration |
| `src/ui/hud.rs` | Cinematic HUD: boss HP bar with phase markers, player HP/energy bars, score, round pips |
| `src/ui/menus.rs` | Overhauled title, pause, game over, game won screens |
| `src/ui/announcement.rs` | Round announcement UI (timed text reveal sequence) |

### Modified Files
| File | Changes |
|------|---------|
| `src/data/game_state.rs` | Replace `Playing` with `RoundAnnounce`, `RoundActive` |
| `src/core/mod.rs` | Replace `enemies` module with `boss` module |
| `src/systems/mod.rs` | Add `round` module |
| `src/ui/mod.rs` | Add `hud`, `menus`, `announcement` modules; remove unused modules |
| `src/app.rs` | Remove old UI/enemy setup; integrate new systems; rename GameData.wave→round; add total_rounds |
| `src/systems/combat.rs` | Replace `boss_shoot_system` Enemy query with Boss query; adapt particle spawning |
| `src/systems/collision.rs` | Replace Enemy queries with Boss queries; add hazard zone/beam/trail collision |
| `src/systems/game_over.rs` | Restart → RoundAnnounce; pause resumes → RoundActive; round progression; between-round HP restore |
| `src/systems/audio.rs` | Add 9 new synthesized sound effects |
| `src/systems/particles.rs` | Add dash trail, hazard zone, beam sweep, charge shockwave particle effects |

### Removed Files
| File | Reason |
|------|--------|
| `src/core/enemies/mod.rs` | Replaced by `src/core/boss/mod.rs` |
| `src/core/enemies/components.rs` | Replaced by `src/core/boss/components.rs` |
| `src/core/enemies/systems.rs` | Replaced by `src/core/boss/systems.rs` |

---

## Task 1: Game State & Data Model Foundation

**Files:**
- Modify: `src/data/game_state.rs`
- Modify: `src/app.rs:24-45` (GameData struct)
- Create: `src/core/boss/mod.rs`
- Create: `src/core/boss/components.rs`
- Modify: `src/core/mod.rs`

- [ ] **Step 1: Update GameState enum**

In `src/data/game_state.rs`, replace the enum:

```rust
#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    RoundAnnounce,
    RoundActive,
    Paused,
    GameOver,
    Won,
}
```

- [ ] **Step 2: Update GameData resource**

In `src/app.rs`, update the `GameData` struct. Rename `wave` to `round`, add `total_rounds`:

```rust
#[derive(Resource)]
pub struct GameData {
    pub score: u32,
    pub round: u32,
    pub high_score: u32,
    pub total_play_time: f32,
    pub enemies_killed: u32,
    pub total_enemies: u32,
    pub total_rounds: u32,
}
```

Default: `round: 1`, `total_rounds: 5`, `total_enemies: 1`.

- [ ] **Step 3: Create boss components**

Create `src/core/boss/components.rs`:

```rust
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
    pub phase_thresholds: (f32, f32), // (0.50, 0.20)
    pub transition_style: TransitionStyle,
    pub primary_timer: Timer,
    pub secondary_timer: Option<Timer>,
    pub attack_state: AttackState,
    pub combo_count: u32,      // for Berserker/Apex charge combos
    pub max_combo: u32,        // max charges in a combo
    pub cycle_index: u32,      // for Apex Protocol attack cycle (separate from combo_count)
    pub is_invulnerable: bool, // true during score tally pause
}

// Attack entity components
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

// Phase transition effect marker
#[derive(Component)]
pub struct PhaseTransitionEffect {
    pub timer: Timer,
    pub style: TransitionStyle,
}
```

- [ ] **Step 4: Create boss module files**

Create `src/core/boss/mod.rs`:

```rust
pub(crate) mod components;
pub(crate) mod systems;
pub(crate) mod attacks;
```

Create empty placeholder files:
- `src/core/boss/systems.rs` with `// Boss AI systems — implemented in Task 3`
- `src/core/boss/attacks.rs` with `// Per-boss attack patterns — implemented in Task 4-8`

- [ ] **Step 5: Update module registration**

In `src/core/mod.rs`, replace `pub(crate) mod enemies;` with `pub(crate) mod boss;`.

- [ ] **Step 6: Fix all compile errors from state rename**

Search entire codebase for `GameState::Playing` and replace with `GameState::RoundActive`. Key locations:
- `src/app.rs` lines 100-103 (system run conditions) — all `.run_if(in_state(GameState::Playing))` → `GameState::RoundActive`
- `src/app.rs` line 99 (pause toggle) — `GameState::Playing` → `GameState::RoundActive`
- `src/systems/game_over.rs` line 145 (restart transition) — `GameState::Playing` → `GameState::RoundAnnounce`
- `src/app.rs` line 414 (pause resume) — `GameState::Playing` → `GameState::RoundActive`

Search for `game_data.wave` and rename to `game_data.round`. Search for `Enemy` imports and comment out broken references temporarily (they will be replaced in Task 2).

- [ ] **Step 7: Verify compilation**

Run: `cargo check 2>&1 | head -40`

Fix any remaining compile errors from the rename. The goal is a compiling project with the new state machine, even if gameplay is temporarily broken (enemies don't spawn yet).

- [ ] **Step 8: Commit**

```bash
git add src/data/game_state.rs src/app.rs src/core/boss/ src/core/mod.rs src/systems/game_over.rs src/systems/collision.rs src/systems/combat.rs
git commit -m "feat: add game state machine and boss data model foundation"
```

---

## Task 2: Boss Spawning & Basic AI Framework

**Files:**
- Create: `src/core/boss/systems.rs`
- Modify: `src/systems/combat.rs`
- Modify: `src/systems/collision.rs`
- Modify: `src/systems/game_over.rs`
- Remove: `src/core/enemies/` (delete directory)

- [ ] **Step 1: Implement spawn_boss()**

In `src/core/boss/systems.rs`:

```rust
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
    // (max_hp, transition_style, color, size_multiplier)
    match boss_type {
        BossType::GridPhantom => (150, TransitionStyle::Stagger,
            Color::srgb(0.0, 8.0, 8.0), 1.0),
        BossType::NeonSentinel => (200, TransitionStyle::Stagger,
            Color::srgb(8.0, 0.0, 8.0), 1.2),
        BossType::ChromeBerserker => (250, TransitionStyle::RageBurst,
            Color::srgb(8.0, 4.0, 0.0), 1.4),
        BossType::VoidWeaver => (300, TransitionStyle::Stagger,
            Color::srgb(4.0, 0.0, 8.0), 1.1),
        BossType::ApexProtocol => (400, TransitionStyle::RageBurst,
            Color::srgb(8.0, 8.0, 8.0), 1.6),
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
        Sprite {
            color,
            custom_size: Some(Vec2::new(size, size)),
            ..default()
        },
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
        },
        GameEntity,
    ));
}
```

- [ ] **Step 2: Implement boss phase transition detection**

Add to `src/core/boss/systems.rs`:

```rust
pub fn boss_phase_system(
    mut boss_query: Query<&mut Boss>,
    mut screen_shake: ResMut<crate::app::ScreenShake>,
    mut sound_events: EventWriter<crate::systems::audio::SoundEvent>,
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
            let old_phase = boss.phase;
            boss.phase = new_phase;

            match boss.transition_style {
                TransitionStyle::Stagger => {
                    boss.attack_state = AttackState::Recovery(
                        Timer::from_seconds(1.5, TimerMode::Once)
                    );
                    // TODO: spawn PhaseTransitionEffect entity for visual
                },
                TransitionStyle::RageBurst => {
                    screen_shake.intensity = 1.5;
                    screen_shake.duration = 0.5;
                    screen_shake.timer = 0.5; // Must match duration — timer counts DOWN
                    // TODO: spawn shockwave ring at boss position
                },
            }

            // Update combo limits for Berserker in later phases
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
```

- [ ] **Step 3: Add basic boss movement system**

Add to `src/core/boss/systems.rs` — a simple idle movement (float up and down) that all bosses use as baseline. Boss-specific movement is handled in attack patterns (Task 4+).

```rust
pub fn boss_idle_movement(
    time: Res<Time>,
    mut boss_query: Query<(&Boss, &mut Transform)>,
) {
    for (boss, mut transform) in boss_query.iter_mut() {
        // Only move when idle (not dashing, charging, etc.)
        if matches!(boss.attack_state, AttackState::Idle) {
            // Gentle float
            let t = time.elapsed_secs();
            transform.translation.y = 150.0 + (t * 1.5).sin() * 30.0;
        }
    }
}
```

- [ ] **Step 4: Update collision system**

In `src/systems/collision.rs`, replace `Enemy` queries with `Boss` queries. Change the function signature and logic:

Replace `enemy_query: Query<(Entity, &mut Enemy, &Transform, &Sprite), With<Enemy>>` with `boss_query: Query<(Entity, &mut Boss, &Transform, &Sprite), With<Boss>>`.

In the player-projectile-vs-enemy section: replace `enemy.current` with `boss.current_hp`, `enemy.max` with `boss.max_hp`, `enemy.is_dead` with a check for `boss.current_hp == 0`. Replace `enemy.last_collision_time` — add a `last_hit_time: Option<Instant>` field to Boss component.

On boss death: fire `DeathEvent`, set `game_data.enemies_killed += 1`, add score.

In the player-vs-enemy section: replace enemy queries with boss queries similarly.

Also add collision detection for `DashTrail`, `HazardZone`, and `BossProjectile` entities against the player (all deal 1 damage with same invincibility window pattern as current `EnemyParticle`).

- [ ] **Step 5: Update combat system**

In `src/systems/combat.rs`, replace `boss_shoot_system` to query `Boss` instead of `Enemy`. For now, make it a simple passthrough that fires projectiles from the boss position using the boss's primary timer — the real per-boss attack patterns come in Tasks 4-8.

```rust
pub(crate) fn boss_shoot_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(&mut Boss, &GlobalTransform, &Transform)>,
    mut sound_events: EventWriter<SoundEvent>,
) {
    for (mut boss, global_transform, local_transform) in query.iter_mut() {
        boss.primary_timer.tick(time.delta());
        if !boss.primary_timer.just_finished() {
            continue;
        }
        if !matches!(boss.attack_state, AttackState::Idle) {
            continue;
        }

        // Basic projectile fire (placeholder — replaced by per-boss attacks)
        let pos = global_transform.translation().truncate();
        let scale = local_transform.scale.xy();
        let half = scale * 25.0;

        let corners = [
            Vec2::new(half.x, half.y),
            Vec2::new(-half.x, half.y),
            Vec2::new(half.x, -half.y),
            Vec2::new(-half.x, -half.y),
        ];

        sound_events.write(SoundEvent(crate::systems::audio::SoundEffect::EnemyShoot));

        for corner in corners {
            let world_pos = pos + corner;
            let velocity = corner.normalize_or_zero() * 120.0;
            spawn_enemy_particle_sprite(&mut commands, world_pos, velocity);
        }
    }
}
```

- [ ] **Step 6: Update game_over restart logic**

In `src/systems/game_over.rs`, update `restart_listener`:
- Replace `Enemy` despawn query with `Boss` despawn query
- Add despawn queries for `DashTrail`, `HazardZone`, `BeamSweep`, `ChargeTelegraph`, `BossProjectile`
- Reset `game_data.round = 1`
- Transition to `GameState::RoundAnnounce` instead of `GameState::Playing`
- In `pause_menu_system`: resume to `GameState::RoundActive`

- [ ] **Step 7: Delete old enemies module**

Remove: `src/core/enemies/components.rs`, `src/core/enemies/systems.rs`, `src/core/enemies/mod.rs`

Remove the `src/core/enemies/` directory.

- [ ] **Step 8: Update app.rs system registration**

In `src/app.rs`:
- Remove old enemy system imports (`create_enemies`, `enemy_movement_system`, `enemy_rotation`)
- Add new boss system imports
- In startup: remove `create_enemies` call
- Add `boss_phase_system` and `boss_idle_movement` to `RoundActive` state systems
- Keep `boss_shoot_system` in `RoundActive` state

- [ ] **Step 9: Verify compilation and basic gameplay**

Run: `cargo check 2>&1 | head -40`
Then: `cargo run` — verify game starts, menu works. Gameplay will be broken (no boss spawns yet) until Task 3.

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat: replace enemy system with boss framework, update collision and combat"
```

---

## Task 3: Round Flow & Announcement System

**Files:**
- Create: `src/systems/round.rs`
- Create: `src/ui/announcement.rs`
- Modify: `src/systems/mod.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/app.rs`

- [ ] **Step 1: Create round management system**

Create `src/systems/round.rs`:

```rust
use bevy::prelude::*;
use crate::app::{GameData, GameEntity};
use crate::data::game_state::GameState;
use crate::core::boss::systems::spawn_boss;
use crate::core::player::components::Player;

#[derive(Resource)]
pub struct RoundTimer {
    pub timer: Timer,
    pub phase: AnnouncementPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnnouncementPhase {
    ThreatLine,    // 0.0-0.4s
    RoundNumber,   // 0.4-0.8s
    BossName,      // 0.8-1.2s
    FlavorText,    // 1.2-1.5s
    Hold,          // 1.5-2.2s
    FadeOut,       // 2.2-2.5s
}

pub fn start_round_announce(
    mut commands: Commands,
) {
    commands.insert_resource(RoundTimer {
        timer: Timer::from_seconds(2.5, TimerMode::Once),
        phase: AnnouncementPhase::ThreatLine,
    });
}

pub fn round_announce_system(
    mut commands: Commands,
    time: Res<Time>,
    mut round_timer: ResMut<RoundTimer>,
    game_data: Res<GameData>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    round_timer.timer.tick(time.delta());
    let elapsed = round_timer.timer.elapsed_secs();

    // Update announcement phase
    round_timer.phase = if elapsed < 0.4 {
        AnnouncementPhase::ThreatLine
    } else if elapsed < 0.8 {
        AnnouncementPhase::RoundNumber
    } else if elapsed < 1.2 {
        AnnouncementPhase::BossName
    } else if elapsed < 1.5 {
        AnnouncementPhase::FlavorText
    } else if elapsed < 2.2 {
        AnnouncementPhase::Hold
    } else {
        AnnouncementPhase::FadeOut
    };

    if round_timer.timer.finished() {
        // Spawn boss and transition to active gameplay
        spawn_boss(&mut commands, game_data.round);
        next_state.set(GameState::RoundActive);
    }
}

#[derive(Resource)]
pub struct ScoreTallyTimer {
    pub timer: Timer,
}

pub fn boss_defeated_check(
    game_data: Res<GameData>,
    boss_query: Query<&crate::core::boss::components::Boss>,
    mut commands: Commands,
) {
    // If boss is dead and no tally timer exists, start one
    if boss_query.is_empty() {
        return; // Boss not spawned yet or already despawned
    }
    for boss in boss_query.iter() {
        if boss.current_hp == 0 {
            commands.insert_resource(ScoreTallyTimer {
                timer: Timer::from_seconds(1.0, TimerMode::Once),
            });
            return;
        }
    }
}

pub fn score_tally_system(
    mut commands: Commands,
    time: Res<Time>,
    mut tally_timer: Option<ResMut<ScoreTallyTimer>>,
    mut game_data: ResMut<GameData>,
    mut next_state: ResMut<NextState<GameState>>,
    mut player_query: Query<&mut Player>,
    boss_query: Query<Entity, With<crate::core::boss::components::Boss>>,
) {
    let Some(mut tally) = tally_timer else { return };
    tally.timer.tick(time.delta());

    // Player is invulnerable during tally
    if let Ok(mut player) = player_query.single_mut() {
        // Mark invulnerable by setting last_collision_time to now (effectively infinite cooldown)
    }

    if tally.timer.finished() {
        // Despawn dead boss AND all hazards/projectiles (anything with GameEntity except Player/HUD)
        for entity in boss_query.iter() {
            commands.entity(entity).despawn();
        }
        // Also despawn all DashTrail, HazardZone, BossProjectile, ChargeTelegraph, EnemyParticle
        // Query each type and despawn. Alternatively, query all GameEntity and filter out Player.

        // Restore player HP (50% of max or current, whichever higher)
        if let Ok(mut player) = player_query.single_mut() {
            let half = player.max / 2;
            if player.current < half {
                player.current = half;
            }
            player.energy = 100;
        }

        // Advance round
        game_data.round += 1;
        game_data.enemies_killed = 0;

        if game_data.round > game_data.total_rounds {
            next_state.set(GameState::Won);
        } else {
            next_state.set(GameState::RoundAnnounce);
        }

        commands.remove_resource::<ScoreTallyTimer>();
    } else {
        // During tally, show "ROUND CLEAR" overlay
        // Spawn once at tally start (check if already spawned via marker component)
        // Text: "ROUND CLEAR" centered, cyan, with score bonus text below
    }
}
```

- [ ] **Step 2: Create announcement UI**

Create `src/ui/announcement.rs`:

```rust
use bevy::prelude::*;
use crate::app::GameEntity;
use crate::core::boss::systems::boss_type_for_round;
use crate::core::boss::components::BossType;
use crate::systems::round::{RoundTimer, AnnouncementPhase};

#[derive(Component)]
pub struct AnnouncementEntity;

#[derive(Component)]
pub struct AnnouncementText {
    pub visible_after: f32,  // seconds after announcement starts
}

fn boss_name(boss_type: BossType) -> &'static str {
    match boss_type {
        BossType::GridPhantom => "GRID PHANTOM",
        BossType::NeonSentinel => "NEON SENTINEL",
        BossType::ChromeBerserker => "CHROME BERSERKER",
        BossType::VoidWeaver => "VOID WEAVER",
        BossType::ApexProtocol => "APEX PROTOCOL",
    }
}

fn boss_flavor(boss_type: BossType) -> &'static str {
    match boss_type {
        BossType::GridPhantom => "PHASE SHIFT PROTOCOL ACTIVE",
        BossType::NeonSentinel => "TARGETING ARRAY ONLINE",
        BossType::ChromeBerserker => "RAGE INHIBITORS OFFLINE",
        BossType::VoidWeaver => "DIMENSIONAL TEAR DETECTED",
        BossType::ApexProtocol => "ALL SYSTEMS MAXIMUM OUTPUT",
    }
}

pub fn spawn_announcement_ui(
    mut commands: Commands,
    game_data: Res<crate::app::GameData>,
) {
    let round = game_data.round;
    let boss_type = boss_type_for_round(round);

    // Root container — full screen overlay
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        AnnouncementEntity,
        GameEntity,
    )).with_children(|parent| {
        // Line 1: // INCOMING THREAT //
        parent.spawn((
            Text::new("// INCOMING THREAT //"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::srgba(1.0, 0.0, 1.0, 0.0)), // starts invisible
            AnnouncementText { visible_after: 0.0 },
            Node { margin: UiRect::bottom(Val::Px(12.0)), ..default() },
        ));

        // Line 2: ROUND N
        parent.spawn((
            Text::new(format!("ROUND {}", round)),
            TextFont { font_size: 48.0, ..default() },
            TextColor(Color::srgba(0.0, 1.0, 1.0, 0.0)),
            AnnouncementText { visible_after: 0.4 },
            Node { margin: UiRect::bottom(Val::Px(8.0)), ..default() },
        ));

        // Line 3: Boss name
        parent.spawn((
            Text::new(boss_name(boss_type)),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::srgba(1.0, 0.0, 1.0, 0.0)),
            AnnouncementText { visible_after: 0.8 },
            Node { margin: UiRect::bottom(Val::Px(16.0)), ..default() },
        ));

        // Line 4: Flavor text
        parent.spawn((
            Text::new(boss_flavor(boss_type)),
            TextFont { font_size: 11.0, ..default() },
            TextColor(Color::srgba(0.4, 0.4, 0.4, 0.0)),
            AnnouncementText { visible_after: 1.2 },
        ));
    });
}

pub fn update_announcement_ui(
    round_timer: Option<Res<RoundTimer>>,
    mut text_query: Query<(&AnnouncementText, &mut TextColor)>,
) {
    let Some(timer) = round_timer else { return };
    let elapsed = timer.timer.elapsed_secs();
    let fade_start = 2.2;
    let fade_end = 2.5;

    for (ann_text, mut color) in text_query.iter_mut() {
        if elapsed < ann_text.visible_after {
            // Not yet visible
            color.0 = color.0.with_alpha(0.0);
        } else if elapsed >= fade_start {
            // Fading out
            let fade_pct = ((elapsed - fade_start) / (fade_end - fade_start)).clamp(0.0, 1.0);
            color.0 = color.0.with_alpha(1.0 - fade_pct);
        } else {
            // Visible — quick fade in
            let since_visible = elapsed - ann_text.visible_after;
            let alpha = (since_visible / 0.15).clamp(0.0, 1.0);
            color.0 = color.0.with_alpha(alpha);
        }
    }
}

pub fn despawn_announcement_ui(
    mut commands: Commands,
    query: Query<Entity, With<AnnouncementEntity>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
```

- [ ] **Step 3: Register round and announcement systems**

In `src/systems/mod.rs`, add:
```rust
pub(crate) mod round;
```

In `src/ui/mod.rs`, add:
```rust
pub(crate) mod announcement;
```

- [ ] **Step 4: Wire systems into app.rs**

In `src/app.rs`, register:

- **OnEnter(RoundAnnounce)**: `start_round_announce`, `spawn_announcement_ui`
- **Update + RoundAnnounce**: `round_announce_system`, `update_announcement_ui`
- **OnExit(RoundAnnounce)**: `despawn_announcement_ui`
- **Update + RoundActive**: add `boss_defeated_check`, `score_tally_system` alongside existing combat/collision systems
- **Menu input**: transition to `GameState::RoundAnnounce` instead of setting up enemies directly

Update `menu_input_system` to transition to `RoundAnnounce` on ENTER press (it currently spawns entities and goes to `Playing`). Move player/barrier spawning to `OnEnter(RoundAnnounce)` or keep in menu input but only spawn player/barriers, not enemies.

- [ ] **Step 5: Verify round flow**

Run: `cargo run`

Expected: Menu → press ENTER → see announcement overlay (2.5s) → boss spawns → gameplay. Kill boss → score tally → next round announcement → new boss. After round 5 → Won screen.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add round flow system with announcement UI and boss spawning"
```

---

## Task 4: Grid Phantom (Round 1) Attack Patterns

**Files:**
- Modify: `src/core/boss/attacks.rs`
- Modify: `src/core/boss/systems.rs`

- [ ] **Step 1: Implement phantom attack patterns**

In `src/core/boss/attacks.rs`, implement the Grid Phantom's dash + trail attack:

```rust
use bevy::prelude::*;
use crate::core::boss::components::*;
use crate::core::player::components::Player;
use crate::app::GameEntity;

/// Grid Phantom: Straight-line dashes with telegraph
/// NOTE: Also fires slow homing BossProjectile entities between dashes (during Recovery state).
/// Homing: each frame, steer velocity toward player at ~80px/s with gentle turn rate.
/// Spawn 1-2 homing projectiles per recovery period.
pub fn phantom_attack(
    boss: &mut Boss,
    boss_transform: &Transform,
    player_transform: &Transform,
    commands: &mut Commands,
    time_delta: f32,
) {
    match &mut boss.attack_state {
        AttackState::Idle => {
            // Start wind-up toward player
            boss.attack_state = AttackState::WindUp(
                Timer::from_seconds(1.0, TimerMode::Once)
            );
            // Spawn telegraph line toward player
            let start = boss_transform.translation.truncate();
            let end = player_transform.translation.truncate();
            commands.spawn((
                Sprite {
                    color: Color::srgba(0.0, 1.0, 1.0, 0.3),
                    custom_size: Some(Vec2::new(3.0, start.distance(end))),
                    ..default()
                },
                Transform::from_translation(((start + end) / 2.0).extend(0.5))
                    .with_rotation(Quat::from_rotation_z(
                        (end - start).to_angle() - std::f32::consts::FRAC_PI_2
                    )),
                ChargeTelegraph {
                    start,
                    end,
                    lifetime: Timer::from_seconds(1.0, TimerMode::Once),
                },
                GameEntity,
            ));
        }
        AttackState::WindUp(timer) => {
            timer.tick(std::time::Duration::from_secs_f32(time_delta));
            if timer.finished() {
                // Dash toward last known player position
                let target = player_transform.translation.truncate();
                boss.attack_state = AttackState::Dashing {
                    target,
                    speed: 800.0,
                };
            }
        }
        AttackState::Dashing { target, speed } => {
            let current = boss_transform.translation.truncate();
            let distance = current.distance(*target);
            if distance < 10.0 {
                // Arrived — enter recovery
                let recovery_time = match boss.phase {
                    BossPhase::Phase1 => 3.0,
                    BossPhase::Phase2 => 2.0,
                    BossPhase::Phase3 => 1.0,
                };

                // In Phase 2+, leave a trail
                if boss.phase != BossPhase::Phase1 {
                    let trail_lifetime = match boss.phase {
                        BossPhase::Phase2 => 2.0,
                        BossPhase::Phase3 => 4.0,
                        _ => 0.0,
                    };
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(0.0, 1.0, 1.0, 0.4),
                            custom_size: Some(Vec2::new(20.0, 20.0)),
                            ..default()
                        },
                        Transform::from_translation(current.extend(0.1)),
                        DashTrail {
                            lifetime: Timer::from_seconds(trail_lifetime, TimerMode::Once),
                            damage: 1,
                        },
                        GameEntity,
                    ));
                }

                // Phase 3: chain dashes
                if boss.phase == BossPhase::Phase3 && boss.combo_count < 1 {
                    boss.combo_count += 1;
                    boss.attack_state = AttackState::WindUp(
                        Timer::from_seconds(0.3, TimerMode::Once)
                    );
                } else {
                    boss.combo_count = 0;
                    boss.attack_state = AttackState::Recovery(
                        Timer::from_seconds(recovery_time, TimerMode::Once)
                    );
                }
            }
            // Movement is handled by the boss movement system reading Dashing state
        }
        AttackState::Recovery(timer) => {
            timer.tick(std::time::Duration::from_secs_f32(time_delta));
            if timer.finished() {
                boss.attack_state = AttackState::Idle;
            }
        }
        _ => {}
    }
}
```

- [ ] **Step 2: Wire phantom attack into boss AI dispatch**

In `src/core/boss/systems.rs`, add a `boss_attack_system` that dispatches to per-boss attack functions:

```rust
pub fn boss_attack_system(
    time: Res<Time>,
    mut commands: Commands,
    mut boss_query: Query<(&mut Boss, &Transform)>,
    player_query: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player_query.single() else { return };

    for (mut boss, boss_transform) in boss_query.iter_mut() {
        let delta = time.delta_secs();
        match boss.boss_type {
            BossType::GridPhantom => {
                crate::core::boss::attacks::phantom_attack(
                    &mut boss, boss_transform, player_transform, &mut commands, delta,
                );
            }
            // Other bosses — fall through to basic timer shoot for now
            _ => {
                boss.primary_timer.tick(time.delta());
            }
        }
    }
}
```

Update `boss_idle_movement` to handle `Dashing` state — move boss toward target at speed:

```rust
AttackState::Dashing { target, speed } => {
    let direction = (*target - transform.translation.truncate()).normalize_or_zero();
    transform.translation += (direction * *speed * time.delta_secs()).extend(0.0);
}
```

- [ ] **Step 3: Add dash trail and telegraph lifetime systems**

Add to `src/core/boss/systems.rs`:

```rust
pub fn hazard_lifetime_system(
    mut commands: Commands,
    time: Res<Time>,
    mut trail_query: Query<(Entity, &mut DashTrail, &mut Sprite)>,
    mut telegraph_query: Query<(Entity, &mut ChargeTelegraph)>,
) {
    for (entity, mut trail, mut sprite) in trail_query.iter_mut() {
        trail.lifetime.tick(time.delta());
        // Fade out
        let alpha = 0.4 * (1.0 - trail.lifetime.fraction());
        sprite.color = sprite.color.with_alpha(alpha);
        if trail.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
    for (entity, mut telegraph) in telegraph_query.iter_mut() {
        telegraph.lifetime.tick(time.delta());
        if telegraph.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
```

- [ ] **Step 4: Register new systems and test**

Add `boss_attack_system` and `hazard_lifetime_system` to `RoundActive` systems in `app.rs`.

Run: `cargo run` — play Round 1. Grid Phantom should dash at player with telegraph line, leave trails in Phase 2+.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: implement Grid Phantom (Round 1) dash and trail attack patterns"
```

---

## Task 5: Neon Sentinel (Round 2) Attack Patterns

**Files:**
- Modify: `src/core/boss/attacks.rs`
- Modify: `src/core/boss/systems.rs`

- [ ] **Step 1: Implement sentinel beam sweep attack**

Add to `src/core/boss/attacks.rs`:

```rust
/// Neon Sentinel: Rotating beam sweeps
pub fn sentinel_attack(
    boss: &mut Boss,
    boss_transform: &Transform,
    commands: &mut Commands,
    time_delta: f32,
) {
    boss.primary_timer.tick(std::time::Duration::from_secs_f32(time_delta));

    if boss.primary_timer.just_finished() {
        let pos = boss_transform.translation.truncate();
        let rotation_speed = match boss.phase {
            BossPhase::Phase1 => 1.5,
            BossPhase::Phase2 => 2.5,
            BossPhase::Phase3 => 3.0,
        };

        let num_beams = match boss.phase {
            BossPhase::Phase1 => 1,
            BossPhase::Phase2 => 2,
            BossPhase::Phase3 => 3,
        };

        let base_angle = boss_transform.rotation.to_euler(EulerRot::ZYX).0;

        for i in 0..num_beams {
            let angle_offset = if num_beams > 1 {
                (i as f32 / num_beams as f32) * std::f32::consts::TAU
            } else {
                0.0
            };

            // Spawn beam projectiles in a line
            let beam_angle = base_angle + angle_offset;
            let direction = Vec2::from_angle(beam_angle);
            let beam_length = 600.0;
            let segments = 12;

            for seg in 0..segments {
                let dist = (seg as f32 / segments as f32) * beam_length;
                let seg_pos = pos + direction * dist;
                let speed = if boss.phase == BossPhase::Phase3 {
                    // Spread: slight angle variation
                    let spread = (seg as f32 * 0.02) * if seg % 2 == 0 { 1.0 } else { -1.0 };
                    let spread_dir = Vec2::from_angle(beam_angle + spread);
                    spread_dir * 80.0
                } else {
                    direction * 0.1 // Nearly stationary beam segments
                };

                commands.spawn((
                    Sprite {
                        color: Color::srgb(8.0, 0.0, 8.0),
                        custom_size: Some(Vec2::new(4.0, 4.0)),
                        ..default()
                    },
                    Transform::from_translation(seg_pos.extend(0.3)),
                    BossProjectile { velocity: speed, damage: 1 },
                    GameEntity,
                ));
            }
        }
    }

    // Rotate the boss (Sentinel is stationary but rotates)
    // Rotation handled in boss_idle_movement via a rotation addition
}
```

- [ ] **Step 2: Add sentinel to AI dispatch**

In `boss_attack_system`, add the `NeonSentinel` match arm calling `sentinel_attack`.

In `boss_idle_movement`, add rotation for Sentinel:
```rust
if boss.boss_type == BossType::NeonSentinel {
    let rot_speed = match boss.phase {
        BossPhase::Phase1 => 1.0,
        BossPhase::Phase2 => 1.8,
        BossPhase::Phase3 => 2.5,
    };
    transform.rotate_z(rot_speed * time.delta_secs());
}
```

- [ ] **Step 3: Add BossProjectile movement and cleanup**

Add to `src/core/boss/systems.rs`:

```rust
pub fn boss_projectile_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &BossProjectile)>,
) {
    for (entity, mut transform, projectile) in query.iter_mut() {
        transform.translation += (projectile.velocity * time.delta_secs()).extend(0.0);

        // Despawn if off screen
        if transform.translation.x.abs() > 700.0 || transform.translation.y.abs() > 400.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

Register in `RoundActive` systems.

- [ ] **Step 4: Test Round 2**

Run: `cargo run` — beat Round 1, verify Round 2 Sentinel fires beam sweeps with rotation. Phase 2 should fire from 2 angles, Phase 3 should split into 3 spread patterns.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: implement Neon Sentinel (Round 2) beam sweep attack patterns"
```

---

## Task 6: Chrome Berserker (Round 3) Attack Patterns

**Files:**
- Modify: `src/core/boss/attacks.rs`
- Modify: `src/core/boss/systems.rs`

- [ ] **Step 1: Implement berserker charge attack**

Add to `src/core/boss/attacks.rs`:

```rust
/// Chrome Berserker: Charges at player with combos
pub fn berserker_attack(
    boss: &mut Boss,
    boss_transform: &Transform,
    player_transform: &Transform,
    commands: &mut Commands,
    time_delta: f32,
    screen_shake: &mut crate::app::ScreenShake,
) {
    match &mut boss.attack_state {
        AttackState::Idle => {
            boss.attack_state = AttackState::WindUp(
                Timer::from_seconds(0.8, TimerMode::Once)
            );
            // Screen shake during wind-up
            screen_shake.intensity = 0.3;
            screen_shake.duration = 0.8;
            screen_shake.timer = 0.0;
        }
        AttackState::WindUp(timer) => {
            timer.tick(std::time::Duration::from_secs_f32(time_delta));
            if timer.finished() {
                let target = player_transform.translation.truncate();
                boss.attack_state = AttackState::Charging {
                    target,
                    speed: 1000.0,
                };
            }
        }
        AttackState::Charging { target, speed } => {
            let current = boss_transform.translation.truncate();
            if current.distance(*target) < 15.0 {
                // Arrived — Phase 3: emit shockwave
                if boss.phase == BossPhase::Phase3 {
                    let pos = boss_transform.translation;
                    // Spawn expanding shockwave ring (reuse existing ShockwaveRing)
                    // For now spawn damage zone
                    commands.spawn((
                        Sprite {
                            color: Color::srgba(1.0, 0.5, 0.0, 0.5),
                            custom_size: Some(Vec2::new(80.0, 80.0)),
                            ..default()
                        },
                        Transform::from_translation(pos),
                        DashTrail {
                            lifetime: Timer::from_seconds(0.5, TimerMode::Once),
                            damage: 1,
                        },
                        GameEntity,
                    ));
                    screen_shake.intensity = 1.0;
                    screen_shake.duration = 0.3;
                    screen_shake.timer = 0.0;
                }

                // Check combo
                boss.combo_count += 1;
                if boss.combo_count < boss.max_combo {
                    // Chain another charge
                    boss.attack_state = AttackState::WindUp(
                        Timer::from_seconds(0.3, TimerMode::Once)
                    );
                } else {
                    boss.combo_count = 0;
                    let recovery = match boss.phase {
                        BossPhase::Phase1 => 2.0,
                        BossPhase::Phase2 | BossPhase::Phase3 => 1.0,
                    };
                    boss.attack_state = AttackState::Recovery(
                        Timer::from_seconds(recovery, TimerMode::Once)
                    );
                }
            }
        }
        AttackState::Recovery(timer) => {
            timer.tick(std::time::Duration::from_secs_f32(time_delta));
            if timer.finished() {
                boss.attack_state = AttackState::Idle;
            }
        }
        _ => {}
    }
}
```

- [ ] **Step 2: Wire berserker into AI dispatch**

Add `ChromeBerserker` arm in `boss_attack_system`. Pass `screen_shake` as parameter.

Handle `Charging` state in `boss_idle_movement` same as `Dashing` (move toward target at speed).

- [ ] **Step 3: Test Round 3**

Run: `cargo run` — verify Berserker charges with wind-up, combos in Phase 2, shockwaves in Phase 3.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: implement Chrome Berserker (Round 3) charge and combo attacks"
```

---

## Task 7: Void Weaver (Round 4) Attack Patterns

**Files:**
- Modify: `src/core/boss/attacks.rs`
- Modify: `src/core/boss/systems.rs`

- [ ] **Step 1: Implement weaver hazard zone attacks**

Add to `src/core/boss/attacks.rs`:

```rust
/// Void Weaver: Spawns hazard zones, teleports between them
pub fn weaver_attack(
    boss: &mut Boss,
    boss_transform: &Transform,
    player_transform: &Transform,
    commands: &mut Commands,
    time_delta: f32,
    hazard_count: usize,
) {
    boss.primary_timer.tick(std::time::Duration::from_secs_f32(time_delta));

    let max_zones = match boss.phase {
        BossPhase::Phase1 | BossPhase::Phase2 => {
            if boss.phase == BossPhase::Phase1 { 3 } else { 4 }
        }
        BossPhase::Phase3 => 4,
    };

    if boss.primary_timer.just_finished() && hazard_count < max_zones {
        // Spawn hazard zone at random position
        let x = (rand::random::<f32>() - 0.5) * 1000.0;
        let y = (rand::random::<f32>() - 0.5) * 400.0;

        let drift = if boss.phase != BossPhase::Phase1 {
            let to_player = (player_transform.translation.truncate() - Vec2::new(x, y)).normalize_or_zero();
            Some(to_player * 30.0)
        } else {
            None
        };

        let explodes = boss.phase == BossPhase::Phase3;
        let explosion_timer = if explodes {
            Some(Timer::from_seconds(3.0, TimerMode::Once))
        } else {
            None
        };

        commands.spawn((
            Sprite {
                color: Color::srgba(0.5, 0.0, 1.0, 0.3),
                custom_size: Some(Vec2::new(60.0, 60.0)),
                ..default()
            },
            Transform::from_xyz(x, y, 0.1),
            HazardZone {
                radius: 30.0,
                lifetime: Timer::from_seconds(5.0, TimerMode::Once),
                drift_velocity: drift,
                explodes,
                explosion_timer,
                damage: 1,
            },
            GameEntity,
        ));
    }

    // Teleport between zones (handled via attack state cycle)
    match &mut boss.attack_state {
        AttackState::Idle => {
            let teleport_cd = match boss.phase {
                BossPhase::Phase1 => 3.0,
                BossPhase::Phase2 => 2.5,
                BossPhase::Phase3 => 1.5,
            };
            boss.attack_state = AttackState::Recovery(
                Timer::from_seconds(teleport_cd, TimerMode::Once)
            );
        }
        AttackState::Recovery(timer) => {
            timer.tick(std::time::Duration::from_secs_f32(time_delta));
            if timer.finished() {
                // Teleport to random position
                boss.attack_state = AttackState::Idle;
                // Actual teleport handled by movement system below
            }
        }
        _ => {}
    }
}
```

- [ ] **Step 2: Add hazard zone system**

Add to `src/core/boss/systems.rs`:

```rust
pub fn hazard_zone_system(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut HazardZone, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut zone, mut transform, mut sprite) in query.iter_mut() {
        zone.lifetime.tick(time.delta());

        // Drift toward player
        if let Some(drift) = zone.drift_velocity {
            transform.translation += (drift * time.delta_secs()).extend(0.0);
        }

        // Explosion countdown
        if let Some(ref mut explosion_timer) = zone.explosion_timer {
            explosion_timer.tick(time.delta());
            if explosion_timer.finished() {
                // Expand briefly then despawn
                sprite.custom_size = Some(Vec2::new(120.0, 120.0));
                sprite.color = Color::srgba(1.0, 0.0, 1.0, 0.8);
                zone.radius = 60.0;
                zone.explosion_timer = None; // One-shot
                zone.lifetime = Timer::from_seconds(0.3, TimerMode::Once); // quick despawn
            }
        }

        // Fade and despawn
        let alpha = 0.3 * (1.0 - zone.lifetime.fraction());
        sprite.color = sprite.color.with_alpha(alpha.max(0.05));

        if zone.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
```

- [ ] **Step 3: Wire weaver into AI dispatch**

Add `VoidWeaver` arm in `boss_attack_system`. Pass hazard zone count from a query.

Handle Weaver teleport in `boss_idle_movement` — when recovery finishes, pick random position:
```rust
if boss.boss_type == BossType::VoidWeaver && matches!(boss.attack_state, AttackState::Idle) {
    let x = (rand::random::<f32>() - 0.5) * 1000.0;
    let y = (rand::random::<f32>() - 0.5) * 300.0;
    transform.translation = Vec3::new(x, y, 0.0);
}
```

Register `hazard_zone_system` in `RoundActive`.

- [ ] **Step 4: Add hazard zone collision in collision.rs**

In `detect_collisions`, add a section checking player vs `HazardZone` entities (same AABB pattern with `zone.radius` as half-size).

- [ ] **Step 5: Test Round 4**

Run: `cargo run` — verify Weaver spawns hazard zones, teleports, zones drift in Phase 2, zones explode in Phase 3.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: implement Void Weaver (Round 4) hazard zone and teleport attacks"
```

---

## Task 8: Apex Protocol (Round 5) Attack Patterns

**Files:**
- Modify: `src/core/boss/attacks.rs`
- Modify: `src/core/boss/systems.rs`

- [ ] **Step 1: Implement apex composite attack**

Add to `src/core/boss/attacks.rs`:

```rust
/// Apex Protocol: Combines all previous boss attacks on a cycle
pub fn apex_attack(
    boss: &mut Boss,
    boss_transform: &Transform,
    player_transform: &Transform,
    commands: &mut Commands,
    time_delta: f32,
    screen_shake: &mut crate::app::ScreenShake,
    hazard_count: usize,
) {
    // Cycle through attack types based on combo_count as a cycle index
    // Phase 1: dash + beam cycle
    // Phase 2: + charge
    // Phase 3: + hazard zones
    let cycle_len = match boss.phase {
        BossPhase::Phase1 => 2, // dash, beam
        BossPhase::Phase2 => 3, // dash, beam, charge
        BossPhase::Phase3 => 4, // dash, beam, charge, hazard
    };

    let current_attack = boss.cycle_index % cycle_len;

    match current_attack {
        0 => phantom_attack(boss, boss_transform, player_transform, commands, time_delta),
        1 => sentinel_attack(boss, boss_transform, commands, time_delta),
        2 => berserker_attack(boss, boss_transform, player_transform, commands, time_delta, screen_shake),
        3 => weaver_attack(boss, boss_transform, player_transform, commands, time_delta, hazard_count),
        _ => {}
    }

    // When an attack cycle completes (recovery→idle), advance cycle_index
    // Note: cycle_index is separate from combo_count to avoid conflicts with sub-attack combo tracking
    if matches!(boss.attack_state, AttackState::Idle) {
        boss.cycle_index += 1;
    }
}
```

- [ ] **Step 2: Wire apex into AI dispatch**

Add `ApexProtocol` arm in `boss_attack_system`.

- [ ] **Step 3: Test final boss**

Run: `cargo run` — play through all 5 rounds. Verify Apex cycles through previous boss attacks, adding more in Phase 2 and 3.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: implement Apex Protocol (Round 5) composite attack patterns"
```

---

## Task 9: Cinematic HUD

**Files:**
- Create: `src/ui/hud.rs`
- Modify: `src/app.rs` (remove old UI setup)
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create HUD module with layout structure**

Create `src/ui/hud.rs` with component markers and the spawn function:

```rust
use bevy::prelude::*;
use crate::app::GameEntity;

// HUD component markers
#[derive(Component)] pub struct HudRoot;
#[derive(Component)] pub struct BossNameText;
#[derive(Component)] pub struct BossHpBar;
#[derive(Component)] pub struct BossHpFill;
#[derive(Component)] pub struct BossPhaseMarker50;
#[derive(Component)] pub struct BossPhaseMarker20;
#[derive(Component)] pub struct BossPhasePip(pub u8); // 1, 2, 3
#[derive(Component)] pub struct PlayerHpFill;
#[derive(Component)] pub struct PlayerHpText;
#[derive(Component)] pub struct PlayerEnergyFill;
#[derive(Component)] pub struct PlayerEnergyText;
#[derive(Component)] pub struct ScoreValueText;
#[derive(Component)] pub struct RoundPip(pub u32); // 1-5
#[derive(Component)] pub struct RoundLabelText;
```

- [ ] **Step 2: Implement HUD spawn function**

The spawn function builds the full Bevy UI node tree:
- Root: absolute position, full screen, `PointerEvents::None`
- Top center: boss name + HP bar container + phase pips
- Bottom left: player label + HP bar + energy bar
- Bottom center: round pips + label
- Bottom right: score label + value

Use `Node` with flexbox for layout. HP bars: outer container with `BackgroundColor(dark)` + inner fill child with `BackgroundColor(colored)` and `width: Val::Percent(hp_pct)`.

Glow effect: spawn a duplicate text node behind each colored text with lower opacity and slight scale increase to simulate glow (layered text shadow approach).

- [ ] **Step 3: Implement HUD update systems**

```rust
pub fn update_boss_hud(
    boss_query: Query<&crate::core::boss::components::Boss>,
    mut name_query: Query<&mut Text, With<BossNameText>>,
    mut fill_query: Query<&mut Node, With<BossHpFill>>,
    mut pip_query: Query<(&BossPhasePip, &mut BackgroundColor)>,
) {
    let Ok(boss) = boss_query.single() else { return };
    let hp_pct = (boss.current_hp as f32 / boss.max_hp as f32 * 100.0).max(0.0);

    // Update HP bar fill width
    if let Ok(mut node) = fill_query.single_mut() {
        node.width = Val::Percent(hp_pct);
    }

    // Update phase pips
    for (pip, mut bg) in pip_query.iter_mut() {
        let cleared = match pip.0 {
            1 => boss.phase != BossPhase::Phase1,
            2 => boss.phase == BossPhase::Phase3,
            3 => false,
            _ => false,
        };
        if cleared {
            bg.0 = Color::srgba(0.15, 0.05, 0.05, 1.0); // dimmed
        } else {
            bg.0 = Color::srgb(1.0, 0.0, 0.24); // active red
        }
    }
}

pub fn update_player_hud(
    player_query: Query<&crate::core::player::components::Player>,
    mut hp_fill: Query<&mut Node, With<PlayerHpFill>>,
    mut hp_text: Query<&mut Text, With<PlayerHpText>>,
    mut energy_fill: Query<&mut Node, (With<PlayerEnergyFill>, Without<PlayerHpFill>)>,
    mut energy_text: Query<&mut Text, (With<PlayerEnergyText>, Without<PlayerHpText>)>,
) {
    let Ok(player) = player_query.single() else { return };

    if let Ok(mut node) = hp_fill.single_mut() {
        node.width = Val::Percent(player.current as f32);
    }
    if let Ok(mut text) = hp_text.single_mut() {
        **text = format!("{}", player.current);
    }
    if let Ok(mut node) = energy_fill.single_mut() {
        node.width = Val::Percent(player.energy as f32);
    }
    if let Ok(mut text) = energy_text.single_mut() {
        **text = format!("{}", player.energy);
    }
}

pub fn update_score_hud(
    game_data: Res<crate::app::GameData>,
    mut score_text: Query<&mut Text, With<ScoreValueText>>,
    mut round_pips: Query<(&RoundPip, &mut BackgroundColor)>,
    mut round_label: Query<&mut Text, (With<RoundLabelText>, Without<ScoreValueText>)>,
) {
    if let Ok(mut text) = score_text.single_mut() {
        **text = format!("{}", game_data.score);
    }
    for (pip, mut bg) in round_pips.iter_mut() {
        if pip.0 < game_data.round {
            bg.0 = Color::srgba(0.1, 0.1, 0.1, 1.0); // completed — dim
        } else {
            bg.0 = Color::srgb(0.0, 1.0, 0.8); // remaining — cyan
        }
    }
    if let Ok(mut text) = round_label.single_mut() {
        **text = format!("ROUND {} / {}", game_data.round, game_data.total_rounds);
    }
}
```

- [ ] **Step 4: Remove old UI from app.rs**

Remove the old text-based UI setup:
- Remove `AnimatedText`, `EnergyText`, `EnemyHpText`, `ScoreText`, `WaveText` component definitions and their spawn code
- Remove `update_health_ui`, `update_energy_ui`, `update_enemy_health_ui`, `update_score_ui` systems
- Replace with new HUD spawn (on entering `RoundAnnounce` for first round) and update systems

- [ ] **Step 5: Register HUD systems**

In `app.rs`:
- **OnEnter(RoundAnnounce)**: `spawn_hud` (every time — HUD entities carry `GameEntity` and get despawned on restart, so must be recreated each time RoundAnnounce is entered; use a guard query to skip if HUD already exists)
- **Update + RoundActive**: `update_boss_hud`, `update_player_hud`, `update_score_hud`

In `src/ui/mod.rs`:
```rust
pub(crate) mod hud;
pub(crate) mod menus;
pub(crate) mod announcement;
```

- [ ] **Step 6: Test HUD**

Run: `cargo run` — verify cinematic HUD appears with boss HP bar at top, player stats bottom-left, score bottom-right, round pips bottom-center. All update dynamically during gameplay.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add cinematic HUD with boss HP bar, player stats, score, round pips"
```

---

## Task 10: Menu Screens Overhaul

**Files:**
- Create: `src/ui/menus.rs`
- Modify: `src/app.rs` (replace old menu/pause/gameover UI)

- [ ] **Step 1: Create styled menu screens**

Create `src/ui/menus.rs` with functions for each screen:

- `spawn_title_menu()` — "CYBERPUNK BLOOM CUBE" with styled container, neon text, high score, "PRESS ENTER TO START" with pulse
- `spawn_pause_menu()` — dark overlay, bordered container, styled option list
- `spawn_game_over_screen()` — "GAME OVER" with glow, final score, round reached, restart instruction
- `spawn_game_won_screen()` — "VICTORY" with glow, final score, restart instruction
- Corresponding `despawn_*` functions for each

All use the same color system from the HUD (cyan, magenta, red accents on near-black backgrounds). Glow via layered text shadows.

- [ ] **Step 2: Replace old menu code in app.rs**

Remove old `setup_menu`, `game_over_system`, `game_won_system` inline UI code. Replace with calls to new `menus.rs` functions.

Wire as:
- **OnEnter(Menu)**: `spawn_title_menu`
- **OnExit(Menu)**: `despawn_title_menu`
- **OnEnter(Paused)**: `spawn_pause_menu`
- **OnExit(Paused)**: `despawn_pause_menu`
- **OnEnter(GameOver)**: `spawn_game_over_screen`
- **OnExit(GameOver)**: `despawn_game_over_screen`
- **OnEnter(Won)**: `spawn_game_won_screen`
- **OnExit(Won)**: `despawn_game_won_screen`

- [ ] **Step 3: Test all screens**

Run: `cargo run` — verify title screen, pause menu, game over, and victory screens all display with the new styling.

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "feat: overhaul menu screens with cinematic styling"
```

---

## Task 11: New Audio Effects

**Files:**
- Modify: `src/systems/audio.rs`

- [ ] **Step 1: Add new SoundEffect variants**

Add to the `SoundEffect` enum:
```rust
BossSpawn,      // Low rumble + rising tone
PhaseShift,     // Glitch/distortion burst
RageBurst,      // Impact + bass drop
DashTelegraph,  // Rising whine
BeamSweep,      // Sustained mid-frequency
ChargeWindUp,   // Accelerating rumble
HazardSpawn,    // Bubble/pop
HazardExplode,  // Sharp crack
RoundClear,     // Triumphant chord
```

- [ ] **Step 2: Implement synthesis for each sound**

Follow the existing pattern in `audio.rs` (frequency sweeps with envelope-based amplitude decay). Each sound is a procedurally generated WAV buffer:

- **BossSpawn**: 80→200 Hz sweep over 500ms, heavy reverb
- **PhaseShift**: White noise burst + 300→100 Hz sweep, 200ms
- **RageBurst**: 50 Hz thump + noise, 300ms
- **DashTelegraph**: 200→800 Hz sweep, 800ms
- **BeamSweep**: 400 Hz sustained tone, 500ms
- **ChargeWindUp**: 100→400 Hz accelerating, 800ms
- **HazardSpawn**: 600→200 Hz pop, 100ms
- **HazardExplode**: Noise + 200 Hz, 150ms
- **RoundClear**: 400+500+600 Hz chord, 800ms

- [ ] **Step 3: Wire sounds into boss attacks**

Add `SoundEvent` writes at the appropriate points in:
- `spawn_boss()` → BossSpawn
- `boss_phase_system()` → PhaseShift or RageBurst
- `phantom_attack()` WindUp → DashTelegraph
- `sentinel_attack()` fire → BeamSweep
- `berserker_attack()` WindUp → ChargeWindUp
- `weaver_attack()` spawn zone → HazardSpawn, explosion → HazardExplode
- `score_tally_system()` → RoundClear

- [ ] **Step 4: Test audio**

Run: `cargo run` with sound on. Verify each boss has appropriate sound effects for their attacks and phase transitions.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add synthesized audio effects for boss attacks, phases, and rounds"
```

---

## Task 12: Polish & Difficulty Scaling

**Files:**
- Modify: `src/core/boss/systems.rs`
- Modify: `src/core/boss/attacks.rs`
- Modify: `src/systems/particles.rs`

- [ ] **Step 1: Add global difficulty scaling**

In `boss_config()`, scale projectile speed by round:
```rust
let speed_mult = 1.0 + (round as f32 - 1.0) * 0.1; // +10% per round
```

Apply to all BossProjectile velocities.

Add score multiplier to `GameData`:
```rust
pub fn score_multiplier(round: u32) -> f32 {
    match round {
        1 => 1.0,
        2 => 1.5,
        3 => 2.0,
        4 => 2.5,
        5 => 3.0,
        _ => 3.0,
    }
}
```

Apply multiplier in collision.rs score additions.

- [ ] **Step 2: Add phase transition visual effects**

In `src/systems/particles.rs`, add:
- **Stagger flash**: White overlay sprite that fades over 0.5s
- **RageBurst shockwave**: Reuse existing `ShockwaveRing` from death effect but with boss color
- **Phase shift text**: Spawn "PHASE SHIFT" text entity with fade-out timer

Wire these into `boss_phase_system` transition events.

- [ ] **Step 3: Add boss glow pulse**

In `boss_idle_movement` or a new `boss_visual_system`, modulate boss sprite color alpha based on phase:
- Phase 1: steady glow
- Phase 2: slow pulse (sin wave on alpha)
- Phase 3: rapid pulse

Boss color intensifies per phase (multiply RGB by 1.0, 1.3, 1.6).

- [ ] **Step 4: Full playthrough test**

Run: `cargo run` — play through all 5 rounds start to finish. Verify:
- Round announcements display correctly for each boss
- Each boss has distinct attack patterns that escalate across phases
- Phase transitions have visual/audio feedback (stagger vs rage burst)
- HUD updates correctly (boss HP, player stats, round pips, score)
- Between-round HP restoration works
- Menu/pause/game over/won screens are styled
- Difficulty scales noticeably across rounds
- Final boss combines previous mechanics

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add difficulty scaling, phase transition effects, and boss visual polish"
```

---

## Summary

| Task | Description | Dependencies |
|------|-------------|-------------|
| 1 | Game state & data model foundation | None |
| 2 | Boss spawning & basic AI framework | Task 1 |
| 3 | Round flow & announcement system | Task 2 |
| 4 | Grid Phantom (Round 1) attacks | Task 2 |
| 5 | Neon Sentinel (Round 2) attacks | Task 2 |
| 6 | Chrome Berserker (Round 3) attacks | Task 2 |
| 7 | Void Weaver (Round 4) attacks | Task 2 |
| 8 | Apex Protocol (Round 5) attacks | Tasks 4-7 |
| 9 | Cinematic HUD | Task 3 |
| 10 | Menu screens overhaul | Task 1 |
| 11 | New audio effects | Task 2 |
| 12 | Polish & difficulty scaling | Tasks 1-11 |

Tasks 4-7 and 9-11 can be parallelized after Task 3 completes.

---

## Implementation Notes

### System Ordering
Boss systems should run in this order within `RoundActive`:
1. `boss_attack_system` (decides what to do)
2. `boss_idle_movement` (moves boss based on attack state)
3. `boss_phase_system` (checks HP thresholds after damage)
4. `boss_projectile_system`, `hazard_lifetime_system`, `hazard_zone_system` (update attack entities)
5. `detect_collisions` (damage resolution)
6. `boss_defeated_check`, `score_tally_system` (round flow)

Use `.after()` chains in system registration to enforce this.

### Energy Regeneration
Player energy regen currently happens in `update_game_data` system (in app.rs) — adds +1 energy per frame when moving. This system must be preserved and run during `RoundActive`. Verify it still works after the state rename.

### ScreenShake Convention
The existing `ScreenShake.timer` counts DOWN (decremented each frame). To trigger a shake, set `timer = duration`. Setting `timer = 0.0` means no shake. All plan code must follow this convention.
