# More Power-Ups — Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the foundation (module split, catalog, `ProjectileOwner`, `max_energy`, tier-based visuals, HUD buff row) plus 4 common power-ups (Repair Kit, Energy Cell, Phase Shift, Glitch Blink). End state: 6 power-ups active (2 existing + 4 new commons).

**Architecture:** Split existing `src/systems/powerups.rs` into a module at `src/systems/powerups/`. Introduce `catalog.rs` as the single source of truth for kinds/tiers/colors. Add `ProjectileOwner` enum on `BossProjectile` now (unused by Phase 1 power-ups but needed at every spawn site for Phases 2–3). New common effects live in `effects/{instant,phase_shift,blink}.rs`. HUD gets a new "active buffs" row.

**Tech Stack:** Rust, Bevy 0.16.1, procedurally generated WAV audio (existing pattern in `src/systems/audio.rs`).

**Spec:** `docs/superpowers/specs/2026-04-21-more-powerups-design.md`

---

## File Structure

### New files

| File | Responsibility |
|------|---------------|
| `src/systems/powerups/mod.rs` | Module root; spawn + lifetime + pickup systems; re-exports public API |
| `src/systems/powerups/catalog.rs` | `PowerUpKind`, `PowerUpTier`, `PowerUpMeta`, `CATALOG`, `meta`, `roll_random_kind` + unit tests |
| `src/systems/powerups/hud.rs` | Active-buff indicator row (dots + duration bars) |
| `src/systems/powerups/effects/mod.rs` | Shared re-exports |
| `src/systems/powerups/effects/shockwave.rs` | Moved unchanged from `powerups.rs` |
| `src/systems/powerups/effects/laser.rs` | Moved unchanged from `powerups.rs` |
| `src/systems/powerups/effects/instant.rs` | Repair Kit + Energy Cell handlers |
| `src/systems/powerups/effects/phase_shift.rs` | `PhaseShiftActive` + tick system + visual pulse |
| `src/systems/powerups/effects/blink.rs` | `pick_safe_spot` pure function + teleport handler + burst visuals |

Note: the spec proposes further split into `spawn.rs` + `pickup.rs` — deferred to a later refactor (Phase 2 or 3) once those files grow. For Phase 1, keeping spawn + pickup in `mod.rs` is simpler and the file remains manageable.

### Deleted files

| File | Reason |
|------|--------|
| `src/systems/powerups.rs` | Replaced by `src/systems/powerups/` directory module |

### Modified files

| File | Changes |
|------|---------|
| `src/core/boss/components.rs` | Add `ProjectileOwner` enum; add `owner: ProjectileOwner` field to `BossProjectile` |
| `src/core/boss/attacks.rs` | Set `owner: ProjectileOwner::Boss` at all `BossProjectile` spawn sites |
| `src/core/player/components.rs` | Add `max_energy: u32` field to `Player` |
| `src/core/player/systems.rs` | Cap `add_energy` at `max_energy`; init `max_energy: 100` in `spawn_player` |
| `src/app.rs` | Init `max_energy: 100` at Player spawn in `menu_input_system`; register new systems; register `PhaseShiftActive` cleanup on round exit |
| `src/systems/collision.rs` | Route damage by `ProjectileOwner` (Player-owned vs boss = damage boss; Player-owned vs player = no-op); respect `PhaseShiftActive` |
| `src/systems/combat.rs` | Respect `PhaseShiftActive` in `player_particle_movement_system` (no change needed) — placeholder row, actual work is in `collision.rs` |
| `src/systems/audio.rs` | Add 4 new `SoundEffect` variants + generation |
| `src/systems/round.rs` | Update imports (`powerups::...` paths now through module) |
| `src/systems/mod.rs` | No change (already `pub mod powerups`, just becomes a directory) |

---

## Task 1: Move `powerups.rs` into a module (no behavior change)

**Files:**
- Delete: `src/systems/powerups.rs`
- Create: `src/systems/powerups/mod.rs` (same content as deleted `powerups.rs`)

- [ ] **Step 1: Verify current build + tests pass (baseline)**

Run: `cargo build`
Expected: compiles without errors

Run: `cargo test --lib`
Expected: all tests pass

- [ ] **Step 2: Move file into a directory**

```bash
mkdir -p src/systems/powerups
git mv src/systems/powerups.rs src/systems/powerups/mod.rs
```

- [ ] **Step 3: Verify build still passes (no content changes yet)**

Run: `cargo build`
Expected: compiles without errors (Rust finds `mod.rs` same as `powerups.rs`)

Run: `cargo test --lib`
Expected: all existing tests pass (laser_total_duration, laser_phase_from_elapsed)

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "refactor: move powerups.rs into powerups/ module dir"
```

---

## Task 2: Split Shockwave into `effects/shockwave.rs`

**Files:**
- Create: `src/systems/powerups/effects/mod.rs`
- Create: `src/systems/powerups/effects/shockwave.rs`
- Modify: `src/systems/powerups/mod.rs`

- [ ] **Step 1: Create `effects/mod.rs`**

Create `src/systems/powerups/effects/mod.rs`:

```rust
pub mod shockwave;
```

- [ ] **Step 2: Create `effects/shockwave.rs` with the shockwave-specific code**

Create `src/systems/powerups/effects/shockwave.rs` with the following content (copied from current `powerups/mod.rs`):

```rust
use crate::app::{GameEntity, ScreenShake};
use crate::core::boss::components::Boss;
use crate::core::boss::components::{BossProjectile, ChargeTelegraph, DashTrail, HazardZone};
use crate::systems::audio::{SoundEffect, SoundEvent};
use crate::systems::combat::EnemyParticle;
use bevy::prelude::*;

#[derive(Component)]
pub struct PowerUpShockwave {
    pub timer: Timer,
}

/// Apply shockwave effect: clear projectiles/hazards, damage boss, screen shake, spawn ring, play sound.
#[allow(clippy::too_many_arguments)]
pub fn apply_shockwave(
    commands: &mut Commands,
    player_pos: Vec3,
    boss_query: &mut Query<&mut Boss>,
    enemy_particle_query: &Query<Entity, With<EnemyParticle>>,
    boss_projectile_query: &Query<Entity, With<BossProjectile>>,
    dash_trail_query: &Query<Entity, With<DashTrail>>,
    hazard_zone_query: &Query<Entity, With<HazardZone>>,
    telegraph_query: &Query<Entity, With<ChargeTelegraph>>,
    screen_shake: &mut ScreenShake,
    sound_events: &mut EventWriter<SoundEvent>,
) {
    // 1. Despawn all projectiles/hazards
    for entity in enemy_particle_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in boss_projectile_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in dash_trail_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in hazard_zone_query.iter() {
        commands.entity(entity).despawn();
    }
    for entity in telegraph_query.iter() {
        commands.entity(entity).despawn();
    }

    // 2. Deal 20 damage to boss
    for mut boss in boss_query.iter_mut() {
        if boss.current_hp > 0 {
            boss.current_hp = boss.current_hp.saturating_sub(20);
            sound_events.write(SoundEvent(SoundEffect::EnemyHit));
        }
    }

    // 3. Screen shake
    screen_shake.intensity = 2.0;
    screen_shake.duration = 0.5;
    screen_shake.timer = 0.5;

    // 4. Spawn shockwave ring visual
    commands.spawn((
        Sprite {
            color: Color::srgba(8.0, 8.0, 8.0, 0.9),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..default()
        },
        Transform::from_translation(player_pos),
        PowerUpShockwave {
            timer: Timer::from_seconds(0.5, TimerMode::Once),
        },
        GameEntity,
    ));

    // 5. Sound
    sound_events.write(SoundEvent(SoundEffect::ShockwavePowerUp));
}

pub fn powerup_shockwave_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PowerUpShockwave, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut shockwave, mut transform, mut sprite) in query.iter_mut() {
        shockwave.timer.tick(time.delta());
        let progress = shockwave.timer.fraction();
        let scale = 1.0 + progress * 20.0;
        transform.scale = Vec3::splat(scale);
        let alpha = (1.0 - progress) * 0.9;
        sprite.color = Color::srgba(8.0, 8.0, 8.0, alpha);
        if shockwave.timer.finished() {
            commands.entity(entity).despawn();
        }
    }
}

```

(Note: if `cargo build` warns about the `PowerUpShockwave` component or any unused import, remove the unused line. The file should compile clean with the imports above.)

- [ ] **Step 3: Remove shockwave-specific code from `powerups/mod.rs`**

In `src/systems/powerups/mod.rs`, delete:
- `PowerUpShockwave` component definition
- Inside the `PowerUpKind::Shockwave` match arm of `powerup_pickup_system`, replace with a call to `crate::systems::powerups::effects::shockwave::apply_shockwave(...)` passing the existing local/query references
- The `powerup_shockwave_system` function

Add near the top of `powerups/mod.rs`:

```rust
pub mod effects;
pub use effects::shockwave::{PowerUpShockwave, powerup_shockwave_system};
```

Update the `PowerUpKind::Shockwave` arm inside `powerup_pickup_system`:

```rust
PowerUpKind::Shockwave => {
    crate::systems::powerups::effects::shockwave::apply_shockwave(
        &mut commands,
        player_pos,
        &mut boss_query,
        &enemy_particle_query,
        &boss_projectile_query,
        &dash_trail_query,
        &hazard_zone_query,
        &telegraph_query,
        &mut screen_shake,
        &mut sound_events,
    );
}
```

- [ ] **Step 4: Build + test**

Run: `cargo build`
Expected: compiles without errors

Run: `cargo test --lib`
Expected: all tests pass

- [ ] **Step 5: Manual smoke test**

Run: `cargo run --release`
Start game → round 1 → wait for a cyan power-up to spawn → pick it up.
Expected: projectiles clear, screen shakes, boss takes 20 dmg, ring expands — same as before the refactor.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor(powerups): extract Shockwave effect into effects/shockwave.rs"
```

---

## Task 3: Split Laser into `effects/laser.rs`

**Files:**
- Create: `src/systems/powerups/effects/laser.rs`
- Modify: `src/systems/powerups/effects/mod.rs`
- Modify: `src/systems/powerups/mod.rs`

- [ ] **Step 1: Create `effects/laser.rs`**

Create `src/systems/powerups/effects/laser.rs` and move into it **verbatim** the following items currently in `powerups/mod.rs`:

- All `LASER_*` consts (`LASER_CHARGE_DURATION`, `LASER_ACTIVE_DURATION`, `LASER_FADE_DURATION`, `LASER_TOTAL_DURATION`)
- `LaserPhase` enum + `laser_phase_from_elapsed` fn + its `#[cfg(test)]` module
- All laser components: `LaserActive`, `LaserBeamCore`, `LaserBeamShell`, `LaserStreamParticle`, `LaserImpact`, `LaserMuzzle`, `LaserChargeParticle`, `LaserChargeOrb`
- All laser systems: `laser_system`, `laser_charge_particle_system`, `laser_charge_orb_system`, `laser_stream_particle_system`, `laser_impact_system`

Add to the top of the new file:

```rust
use crate::app::{GameEntity, ScreenShake};
use crate::core::boss::components::Boss;
use crate::core::player::components::Player;
use crate::systems::audio::{SoundEffect, SoundEvent};
use crate::systems::collision::collide;
use crate::utils::config::ENTITY_SCALE;
use bevy::prelude::*;
use rand::Rng;
```

Also move the helper function used when picking up Laser — extract it as `pub fn apply_laser_pickup(...)`:

```rust
pub fn apply_laser_pickup(
    commands: &mut Commands,
    player_entity: Entity,
    player_pos: Vec3,
    screen_shake: &mut ScreenShake,
    sound_events: &mut EventWriter<SoundEvent>,
) {
    commands.entity(player_entity).insert(LaserActive {
        timer: Timer::from_seconds(LASER_TOTAL_DURATION, TimerMode::Once),
        sound_timer: Timer::from_seconds(0.5, TimerMode::Repeating),
        phase: LaserPhase::Charging,
        charge_timer: Timer::from_seconds(LASER_CHARGE_DURATION, TimerMode::Once),
    });

    commands.spawn((
        Sprite {
            color: Color::srgba(0.4, 1.0, 0.4, 0.9),
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        },
        Transform::from_translation(player_pos.with_z(0.4)),
        LaserChargeOrb { scale: 1.0 },
        GameEntity,
    ));

    let mut rng = rand::thread_rng();
    for _ in 0..8 {
        let px = (rng.gen_range(-1.0_f32..1.0_f32)) * 600.0;
        let py = (rng.gen_range(-1.0_f32..1.0_f32)) * 350.0;
        let speed = rng.gen_range(200.0_f32..400.0_f32);
        commands.spawn((
            Sprite {
                color: Color::srgba(0.2, 1.0, 0.3, 0.8),
                custom_size: Some(Vec2::new(4.0, 4.0)),
                ..default()
            },
            Transform::from_xyz(px, py, 0.35),
            LaserChargeParticle {
                target: player_pos.truncate(),
                speed,
            },
            GameEntity,
        ));
    }

    screen_shake.intensity = 0.2;
    screen_shake.duration = 0.8;
    screen_shake.timer = 0.8;

    sound_events.write(SoundEvent(SoundEffect::LaserCharge));
}
```

- [ ] **Step 2: Update `effects/mod.rs`**

```rust
pub mod laser;
pub mod shockwave;
```

- [ ] **Step 3: Remove laser code from `powerups/mod.rs`; replace pickup arm with helper call**

In `src/systems/powerups/mod.rs`:
- Delete every laser item listed in Step 1 of this task
- Remove the `#[cfg(test)] mod tests { ... }` block that tests laser (it moved to `effects/laser.rs` with the code)
- Update re-exports at the top:

```rust
pub mod effects;
pub use effects::laser::{
    LaserActive, LaserBeamCore, LaserBeamShell, LaserChargeOrb, LaserChargeParticle, LaserImpact,
    LaserMuzzle, LaserPhase, LaserStreamParticle, laser_charge_orb_system,
    laser_charge_particle_system, laser_impact_system, laser_stream_particle_system, laser_system,
};
pub use effects::shockwave::{PowerUpShockwave, powerup_shockwave_system};
```

- Replace the `PowerUpKind::Laser` arm in `powerup_pickup_system`:

```rust
PowerUpKind::Laser => {
    crate::systems::powerups::effects::laser::apply_laser_pickup(
        &mut commands,
        player_entity,
        player_pos,
        &mut screen_shake,
        &mut sound_events,
    );
}
```

- [ ] **Step 4: Build + test**

Run: `cargo build`
Expected: compiles

Run: `cargo test --lib`
Expected: `effects::laser::tests::test_laser_total_duration` and `effects::laser::tests::test_laser_phase_from_elapsed` both pass

- [ ] **Step 5: Manual smoke test**

Run: `cargo run --release`
Pick up a magenta laser power-up. Expected: charge orb + converging particles → beam fires → hits boss → fades → cleans up. Identical to pre-refactor.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "refactor(powerups): extract Laser effect into effects/laser.rs"
```

---

## Task 4: Introduce `catalog.rs` with kinds, tiers, metadata, weighted roll

**Files:**
- Create: `src/systems/powerups/catalog.rs`
- Modify: `src/systems/powerups/mod.rs` (replace inline `PowerUpKind` enum)
- Modify: `src/systems/powerups/effects/shockwave.rs` (if it references the old enum — it doesn't; no change expected)

- [ ] **Step 1: Create `catalog.rs` with enums, metadata, and helpers**

Create `src/systems/powerups/catalog.rs`:

```rust
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpKind {
    // Common
    RepairKit,
    EnergyCell,
    PhaseShift,
    GlitchBlink,
    // Rare
    Shockwave,
    // Ultra-rare
    Laser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpTier {
    Common,
    Rare,
    UltraRare,
}

#[derive(Debug)]
pub struct PowerUpMeta {
    pub kind: PowerUpKind,
    pub tier: PowerUpTier,
    pub color: Color,
    pub display_name: &'static str,
}

pub const CATALOG: &[PowerUpMeta] = &[
    PowerUpMeta {
        kind: PowerUpKind::RepairKit,
        tier: PowerUpTier::Common,
        color: Color::srgb(0.0, 8.0, 2.0),
        display_name: "REPAIR",
    },
    PowerUpMeta {
        kind: PowerUpKind::EnergyCell,
        tier: PowerUpTier::Common,
        color: Color::srgb(0.0, 4.0, 8.0),
        display_name: "ENERGY",
    },
    PowerUpMeta {
        kind: PowerUpKind::PhaseShift,
        tier: PowerUpTier::Common,
        color: Color::srgba(6.0, 6.0, 8.0, 0.7),
        display_name: "PHASE",
    },
    PowerUpMeta {
        kind: PowerUpKind::GlitchBlink,
        tier: PowerUpTier::Common,
        color: Color::srgb(6.0, 0.5, 8.0),
        display_name: "BLINK",
    },
    PowerUpMeta {
        kind: PowerUpKind::Shockwave,
        tier: PowerUpTier::Rare,
        color: Color::srgb(0.0, 8.0, 8.0),
        display_name: "SHOCKWAVE",
    },
    PowerUpMeta {
        kind: PowerUpKind::Laser,
        tier: PowerUpTier::UltraRare,
        color: Color::srgb(8.0, 0.0, 8.0),
        display_name: "LASER",
    },
];

pub fn meta(kind: PowerUpKind) -> &'static PowerUpMeta {
    CATALOG
        .iter()
        .find(|m| m.kind == kind)
        .expect("every PowerUpKind must have a CATALOG entry")
}

/// Pure-function tier selector from a 0..=99 roll.
/// 0-54 = Common (55%), 55-89 = Rare (35%), 90-99 = UltraRare (10%).
pub fn tier_from_roll(n: u8) -> PowerUpTier {
    match n {
        0..=54 => PowerUpTier::Common,
        55..=89 => PowerUpTier::Rare,
        _ => PowerUpTier::UltraRare,
    }
}

pub fn kinds_in_tier(tier: PowerUpTier) -> Vec<PowerUpKind> {
    CATALOG
        .iter()
        .filter(|m| m.tier == tier)
        .map(|m| m.kind)
        .collect()
}

pub fn roll_random_kind() -> PowerUpKind {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let tier_roll: u8 = rng.gen_range(0..100);
    let tier = tier_from_roll(tier_roll);
    let kinds = kinds_in_tier(tier);
    kinds[rng.gen_range(0..kinds.len())]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_has_a_catalog_entry() {
        // Each PowerUpKind variant must appear in CATALOG.
        let kinds = [
            PowerUpKind::RepairKit,
            PowerUpKind::EnergyCell,
            PowerUpKind::PhaseShift,
            PowerUpKind::GlitchBlink,
            PowerUpKind::Shockwave,
            PowerUpKind::Laser,
        ];
        for kind in kinds {
            // meta() panics if missing, which would fail the test
            let _ = meta(kind);
        }
        assert_eq!(CATALOG.len(), kinds.len());
    }

    #[test]
    fn tier_boundaries() {
        assert_eq!(tier_from_roll(0), PowerUpTier::Common);
        assert_eq!(tier_from_roll(54), PowerUpTier::Common);
        assert_eq!(tier_from_roll(55), PowerUpTier::Rare);
        assert_eq!(tier_from_roll(89), PowerUpTier::Rare);
        assert_eq!(tier_from_roll(90), PowerUpTier::UltraRare);
        assert_eq!(tier_from_roll(99), PowerUpTier::UltraRare);
    }

    #[test]
    fn roll_distribution_within_tolerance() {
        // Over 10_000 rolls, expect ~55% Common / ~35% Rare / ~10% Ultra within ±2%.
        let n = 10_000usize;
        let mut common = 0u32;
        let mut rare = 0u32;
        let mut ultra = 0u32;
        for _ in 0..n {
            match meta(roll_random_kind()).tier {
                PowerUpTier::Common => common += 1,
                PowerUpTier::Rare => rare += 1,
                PowerUpTier::UltraRare => ultra += 1,
            }
        }
        let common_pct = common as f32 / n as f32;
        let rare_pct = rare as f32 / n as f32;
        let ultra_pct = ultra as f32 / n as f32;
        assert!(
            (common_pct - 0.55).abs() < 0.02,
            "common_pct = {} (expected ~0.55)",
            common_pct
        );
        assert!(
            (rare_pct - 0.35).abs() < 0.02,
            "rare_pct = {} (expected ~0.35)",
            rare_pct
        );
        assert!(
            (ultra_pct - 0.10).abs() < 0.02,
            "ultra_pct = {} (expected ~0.10)",
            ultra_pct
        );
    }
}
```

- [ ] **Step 2: Wire catalog into `powerups/mod.rs`**

At the top of `src/systems/powerups/mod.rs`, add:

```rust
pub mod catalog;
pub use catalog::{PowerUpKind, PowerUpTier, meta};
```

Delete the existing inline `PowerUpKind` enum in `mod.rs` (the one with just `Shockwave, Laser`). All references to `PowerUpKind::Shockwave` and `PowerUpKind::Laser` now resolve via the `pub use`.

- [ ] **Step 3: Update `powerup_spawn_system` to use the catalog**

In `src/systems/powerups/mod.rs`, replace the current 50/50 kind-selection logic inside `powerup_spawn_system`. Find:

```rust
let kind = if rand::random::<bool>() {
    PowerUpKind::Shockwave
} else {
    PowerUpKind::Laser
};

let color = match kind {
    PowerUpKind::Shockwave => Color::srgb(0.0, 8.0, 8.0),
    PowerUpKind::Laser => Color::srgb(8.0, 0.0, 8.0),
};
```

Replace with:

```rust
let kind = catalog::roll_random_kind();
let color = meta(kind).color;
```

- [ ] **Step 4: Run tests**

Run: `cargo test --lib catalog`
Expected: 3 tests pass (every_kind_has_a_catalog_entry, tier_boundaries, roll_distribution_within_tolerance)

Run: `cargo test --lib`
Expected: all tests pass

Run: `cargo build`
Expected: compiles

- [ ] **Step 5: Manual smoke test**

Run: `cargo run --release`
Play a round. Watch for power-up spawns. With only 2 old kinds (Shockwave/Laser) still being implemented, commons will spawn but their pickup arms don't exist yet — **expected behavior is that commons spawn visually but do nothing on pickup**. This is intermediate state and gets fixed in Task 8+.

Wait — pickup_system has no arm for commons yet, so picking one up will pattern-match exhaustively or compile-fail. Check compile output from Step 4.

If compile fails due to non-exhaustive match, add a catch-all arm in `powerup_pickup_system` temporarily:

```rust
PowerUpKind::RepairKit
| PowerUpKind::EnergyCell
| PowerUpKind::PhaseShift
| PowerUpKind::GlitchBlink => {
    // TODO(Task 8+): implement
}
```

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat(powerups): add catalog with tiered rarity roll"
```

---

## Task 5: Add `ProjectileOwner` to `BossProjectile`

**Files:**
- Modify: `src/core/boss/components.rs`
- Modify: `src/core/boss/attacks.rs` (2 spawn sites)
- Modify: `src/systems/collision.rs`

- [ ] **Step 1: Add `ProjectileOwner` enum and field**

In `src/core/boss/components.rs`, find:

```rust
#[derive(Component)]
pub struct BossProjectile {
    pub velocity: Vec2,
    #[allow(dead_code)]
    pub damage: u32,
}
```

Replace with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileOwner {
    Boss,
    Player,
}

#[derive(Component)]
pub struct BossProjectile {
    pub velocity: Vec2,
    #[allow(dead_code)]
    pub damage: u32,
    pub owner: ProjectileOwner,
}
```

- [ ] **Step 2: Set `owner: ProjectileOwner::Boss` at every existing spawn site**

In `src/core/boss/attacks.rs`, find the two `BossProjectile { velocity, damage: 5 }` blocks (around lines 171 and 343 per the current file).

For each, update the literal to:

```rust
BossProjectile {
    velocity,
    damage: 5,
    owner: ProjectileOwner::Boss,
},
```

Also add to the `use` statement at the top of `src/core/boss/attacks.rs`:

```rust
use crate::core::boss::components::*;
```

(If already present, `ProjectileOwner` is covered by the wildcard; no extra change needed.)

- [ ] **Step 3: Route damage by owner in `detect_collisions`**

In `src/systems/collision.rs`, find the `BossProjectile vs Player` branch (around line 88). Replace it with a version that checks owner:

```rust
// BossProjectile vs Player (only Boss-owned projectiles hurt player)
for projectile_transform in boss_projectile_query.iter() {
    // Skip: need owner; requires passing owner-aware query. See Step 4.
}
```

This requires updating the query type. Actual change in Step 4.

- [ ] **Step 4: Update the query + collision logic**

In `src/systems/collision.rs`, change the signature of `detect_collisions`:

Find:

```rust
boss_projectile_query: Query<&Transform, With<BossProjectile>>,
```

Replace with:

```rust
boss_projectile_query: Query<(&Transform, &BossProjectile)>,
```

Update the use-import at the top of `src/systems/collision.rs` if needed:

```rust
use crate::core::boss::components::{Boss, BossProjectile, DashTrail, HazardZone, ProjectileOwner};
```

Replace the entire `BossProjectile vs Player` loop with owner-aware routing:

```rust
// BossProjectile vs Player — only Boss-owned projectiles damage the player
for (projectile_transform, projectile) in boss_projectile_query.iter() {
    if projectile.owner != ProjectileOwner::Boss {
        continue;
    }
    let projectile_size = Vec2::new(6.0, 6.0);
    let projectile_pos = projectile_transform.translation;

    if collide(player_pos, player_size, projectile_pos, projectile_size) {
        if player.current > 0
            && player
                .last_collision_time
                .is_none_or(|t| t.elapsed().as_secs_f32() > 0.075)
        {
            player.current -= 1;
            player.last_collision_time = Some(crate::utils::time_compat::Instant::now());
            trigger_screen_shake(&mut screen_shake);
            trigger_damage_flash(player_entity, commands.reborrow());
            sound_events.write(SoundEvent(SoundEffect::PlayerHit));
        }

        if player.current == 0 {
            next_state.set(GameState::GameOver);
        }
    }
}
```

- [ ] **Step 5: Also route Player-owned BossProjectile to damage boss**

Immediately before the existing `PlayerParticle vs Boss` loop in `detect_collisions`, add a new loop for Player-owned `BossProjectile`:

```rust
// Player-owned BossProjectile vs Boss (for reflected/hacked projectiles)
for (_boss_entity, mut boss, boss_transform, boss_sprite) in &mut boss_query {
    let boss_size = boss_sprite.custom_size.unwrap_or(Vec2::ONE);
    let boss_pos = boss_transform.translation;
    for (projectile_transform, projectile) in boss_projectile_query.iter() {
        if projectile.owner != ProjectileOwner::Player {
            continue;
        }
        let projectile_size = Vec2::new(6.0, 6.0);
        if collide(
            projectile_transform.translation,
            projectile_size,
            boss_pos,
            boss_size,
        ) {
            if boss.is_invulnerable {
                continue;
            }
            if boss
                .last_hit_time
                .is_some_and(|t| t.elapsed().as_secs_f32() < 0.075)
            {
                continue;
            }
            if boss.current_hp > 0 {
                let dmg = projectile.damage.max(1);
                boss.current_hp = boss.current_hp.saturating_sub(dmg);
                boss.last_hit_time = Some(crate::utils::time_compat::Instant::now());
                let mult = score_multiplier(game_data.round);
                game_data.score += (10.0 * mult) as u32;
                sound_events.write(SoundEvent(SoundEffect::EnemyHit));
            }
        }
    }
}
```

- [ ] **Step 6: Add a unit test for owner routing**

Append to the `#[cfg(test)] mod tests` block in `src/systems/collision.rs`:

```rust
#[test]
fn projectile_owner_enum_equality() {
    use crate::core::boss::components::ProjectileOwner;
    assert_eq!(ProjectileOwner::Boss, ProjectileOwner::Boss);
    assert_ne!(ProjectileOwner::Boss, ProjectileOwner::Player);
}
```

(Pure-function-style; full integration of owner routing is tested via manual smoke.)

- [ ] **Step 7: Build + test**

Run: `cargo build`
Expected: compiles

Run: `cargo test --lib`
Expected: all tests pass, including new `projectile_owner_enum_equality`

- [ ] **Step 8: Manual smoke test**

Run: `cargo run --release`
Play round 1. Expect boss projectiles still hurt player (Boss-owned default unchanged). Reflected/hacked projectiles don't exist yet — no regressions expected.

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat(boss): add ProjectileOwner enum and route damage by owner"
```

---

## Task 6: Add `max_energy` field to `Player`

**Files:**
- Modify: `src/core/player/components.rs`
- Modify: `src/core/player/systems.rs`
- Modify: `src/app.rs`
- Modify: `src/systems/round.rs`

- [ ] **Step 1: Add field to `Player` struct**

In `src/core/player/components.rs`, change:

```rust
#[derive(Component)]
pub struct Player {
    pub current: u32,
    pub max: u32,
    pub energy: u32,
    pub last_collision_time: Option<Instant>,
    pub last_shot_time: Option<Instant>,
}
```

to:

```rust
#[derive(Component)]
pub struct Player {
    pub current: u32,
    pub max: u32,
    pub energy: u32,
    pub max_energy: u32,
    pub last_collision_time: Option<Instant>,
    pub last_shot_time: Option<Instant>,
}
```

- [ ] **Step 2: Cap `add_energy` at `max_energy`**

In `src/core/player/systems.rs`, change:

```rust
#[allow(dead_code)]
fn add_energy(player: &mut Player) {
    player.energy += 1;
}
```

to:

```rust
fn add_energy(player: &mut Player) {
    if player.energy < player.max_energy {
        player.energy += 1;
    }
}
```

(Also drop the `#[allow(dead_code)]` — it's called on line 22 of the same file, so the lint is spurious.)

- [ ] **Step 3: Initialize `max_energy: 100` at every Player spawn site**

Find every `Player { ... }` literal and add `max_energy: 100,`:

In `src/core/player/systems.rs` `spawn_player`:

```rust
Player {
    current: 100,
    max: 100,
    last_collision_time: None,
    energy: 100,
    max_energy: 100,
    last_shot_time: None,
},
```

In `src/app.rs` `menu_input_system`:

```rust
Player {
    current: 100,
    max: 100,
    last_collision_time: None,
    energy: 100,
    max_energy: 100,
    last_shot_time: None,
},
```

- [ ] **Step 4: Reset `energy` respecting cap in `score_tally_system`**

In `src/systems/round.rs`, find:

```rust
player.energy = 100;
```

Replace with:

```rust
player.energy = player.max_energy;
```

- [ ] **Step 5: Add a unit test**

Append to `src/core/player/components.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_default_shape_compiles() {
        // Sanity that all fields present and default-constructible via struct literal
        let p = Player {
            current: 100,
            max: 100,
            energy: 50,
            max_energy: 100,
            last_collision_time: None,
            last_shot_time: None,
        };
        assert_eq!(p.max_energy, 100);
        assert!(p.energy < p.max_energy);
    }
}
```

- [ ] **Step 6: Build + test**

Run: `cargo build`
Expected: compiles

Run: `cargo test --lib`
Expected: all tests pass

- [ ] **Step 7: Manual smoke test**

Run: `cargo run --release`
Play a round. Move around a lot without shooting. HUD energy bar maxes at 100 and stops growing (previously it would grow unbounded internally, though HUD capped the visual). Shoot normally — confirm still works.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat(player): add max_energy field; cap energy regen and reset"
```

---

## Task 7: Tier-based pickup visual styling (size, pulse, glow ring)

**Files:**
- Modify: `src/systems/powerups/mod.rs` (spawn + lifetime systems)

- [ ] **Step 1: Add helper fns to `catalog.rs`**

Append to `src/systems/powerups/catalog.rs`:

```rust
impl PowerUpTier {
    pub fn base_size_px(&self) -> f32 {
        match self {
            PowerUpTier::Common => 14.0,
            PowerUpTier::Rare => 18.0,
            PowerUpTier::UltraRare => 22.0,
        }
    }

    pub fn pulse_hz(&self) -> f32 {
        match self {
            PowerUpTier::Common => 4.0,
            PowerUpTier::Rare => 6.0,
            PowerUpTier::UltraRare => 9.0,
        }
    }

    /// (glow_scale_factor, glow_alpha, glow_color).
    /// glow_color is None for Common (no glow).
    pub fn glow(&self) -> Option<(f32, f32, Color)> {
        match self {
            PowerUpTier::Common => None,
            PowerUpTier::Rare => Some((1.5, 0.35, Color::srgba(8.0, 8.0, 8.0, 0.35))),
            PowerUpTier::UltraRare => Some((1.8, 0.45, Color::srgba(8.0, 6.0, 0.0, 0.45))),
        }
    }
}
```

- [ ] **Step 2: Add `PowerUpGlow` marker component**

In `src/systems/powerups/mod.rs`, near the existing `PowerUp` component, add:

```rust
#[derive(Component)]
pub struct PowerUpGlow;
```

- [ ] **Step 3: Update `powerup_spawn_system` to spawn tier-styled sprite + optional glow ring**

In `src/systems/powerups/mod.rs`, replace the `powerup_spawn_system` body's spawning section (from `let color = meta(kind).color;` to the end) with:

```rust
let meta = meta(kind);
let color = meta.color;
let base_size = meta.tier.base_size_px();

use crate::utils::config::ENTITY_SCALE;
let pickup_entity = commands
    .spawn((
        Sprite {
            color,
            custom_size: Some(Vec2::new(base_size * ENTITY_SCALE, base_size * ENTITY_SCALE)),
            ..default()
        },
        Transform::from_xyz(x, y, 0.5)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
        PowerUp {
            kind,
            lifetime: Timer::from_seconds(10.0, TimerMode::Once),
        },
        GameEntity,
    ))
    .id();

if let Some((scale, _alpha, glow_color)) = meta.tier.glow() {
    commands.entity(pickup_entity).with_children(|children| {
        children.spawn((
            Sprite {
                color: glow_color,
                custom_size: Some(Vec2::new(
                    base_size * ENTITY_SCALE * scale,
                    base_size * ENTITY_SCALE * scale,
                )),
                ..default()
            },
            // z offset behind the main diamond
            Transform::from_xyz(0.0, 0.0, -0.05),
            PowerUpGlow,
            GameEntity,
        ));
    });
}

// Reset timer for next spawn
let duration = 15.0 + rand::random::<f32>() * 5.0;
powerup_timer.timer = Timer::from_seconds(duration, TimerMode::Once);
```

Remove the old direct `commands.spawn((... PowerUp ..., GameEntity))` call that preceded this section (keep only the catalog-based version).

- [ ] **Step 4: Update `powerup_lifetime_system` to pulse at tier-specific rate**

In `src/systems/powerups/mod.rs`, replace `powerup_lifetime_system`:

```rust
pub fn powerup_lifetime_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PowerUp, &mut Sprite)>,
) {
    let t = time.elapsed_secs();
    for (entity, mut powerup, mut sprite) in query.iter_mut() {
        powerup.lifetime.tick(time.delta());

        let hz = meta(powerup.kind).tier.pulse_hz();
        let pulse = 0.6 + 0.4 * (t * hz).sin();
        let base = meta(powerup.kind).color;
        // Preserve original HDR RGB; pulse modulates alpha only
        let [r, g, b, _a] = base.to_srgba().to_f32_array();
        sprite.color = Color::srgba(r, g, b, pulse);

        if powerup.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}
```

- [ ] **Step 5: Build**

Run: `cargo build`
Expected: compiles

- [ ] **Step 6: Manual smoke test**

Run: `cargo run --release`
Play long enough for several power-ups to spawn.
- Commons (Repair green, Energy blue, Phase white, Blink purple): small diamonds, slow pulse, no glow ring
- Shockwave (cyan): larger, faster pulse, white glow ring
- Laser (magenta): largest, fastest pulse, gold glow ring + halo (halo not added yet; just gold ring for now)

Expected: tiers are visually distinct at a glance.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(powerups): tier-based pickup styling (size, pulse, glow ring)"
```

---

## Task 8: Implement Repair Kit + Energy Cell (instant effects)

**Files:**
- Create: `src/systems/powerups/effects/instant.rs`
- Modify: `src/systems/powerups/effects/mod.rs`
- Modify: `src/systems/powerups/mod.rs` (pickup dispatch)
- Modify: `src/systems/audio.rs` (2 new sound effects)

- [ ] **Step 1: Add 2 new `SoundEffect` variants + synthesis**

In `src/systems/audio.rs`, add to the `SoundEffect` enum:

```rust
RepairKitPickup,
EnergyCellPickup,
```

Add them to `ALL_EFFECTS` const array.

Add to the `generate_sound` match:

```rust
SoundEffect::RepairKitPickup => {
    // Quick ascending chime: three notes C5 E5 G5 over 0.15s
    let duration = 0.15;
    let num_samples = (sample_rate * duration) as usize;
    let notes = [523.25_f32, 659.25, 783.99];
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate;
            let note_idx = ((t / duration) * 3.0) as usize;
            let note_idx = note_idx.min(2);
            let freq = notes[note_idx];
            let local_t = t - (note_idx as f32 * duration / 3.0);
            let envelope = (1.0 - (local_t / (duration / 3.0)).min(1.0)) * 0.9;
            (t * freq * std::f32::consts::TAU).sin() * envelope * volume * 0.3
        })
        .collect()
}
SoundEffect::EnergyCellPickup => {
    // Electric zap: high-freq saw + noise, 0.1s
    let duration = 0.1;
    let num_samples = (sample_rate * duration) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate;
            let envelope = 1.0 - (t / duration);
            let saw = (t * 1800.0 * std::f32::consts::TAU).sin().signum() * 0.3;
            let noise = (rand::random::<f32>() * 2.0 - 1.0) * 0.2;
            (saw + noise) * envelope * volume * 0.3
        })
        .collect()
}
```

- [ ] **Step 2: Create `effects/instant.rs`**

Create `src/systems/powerups/effects/instant.rs`:

```rust
use crate::core::player::components::Player;
use crate::systems::audio::{SoundEffect, SoundEvent};
use bevy::prelude::*;

const REPAIR_AMOUNT: u32 = 25;
const ENERGY_AMOUNT: u32 = 100;

pub fn apply_repair_kit(player: &mut Player, sound_events: &mut EventWriter<SoundEvent>) {
    let new_hp = player.current.saturating_add(REPAIR_AMOUNT).min(player.max);
    player.current = new_hp;
    sound_events.write(SoundEvent(SoundEffect::RepairKitPickup));
}

pub fn apply_energy_cell(player: &mut Player, sound_events: &mut EventWriter<SoundEvent>) {
    let new_energy = player
        .energy
        .saturating_add(ENERGY_AMOUNT)
        .min(player.max_energy);
    player.energy = new_energy;
    sound_events.write(SoundEvent(SoundEffect::EnergyCellPickup));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_player(current: u32, max: u32, energy: u32, max_energy: u32) -> Player {
        Player {
            current,
            max,
            energy,
            max_energy,
            last_collision_time: None,
            last_shot_time: None,
        }
    }

    // apply_repair_kit / apply_energy_cell take a SoundEvent writer; we only test the cap logic
    // via pure fns factored out below.
    fn repair_into(current: u32, max: u32) -> u32 {
        current.saturating_add(REPAIR_AMOUNT).min(max)
    }

    fn energy_into(energy: u32, max_energy: u32) -> u32 {
        energy.saturating_add(ENERGY_AMOUNT).min(max_energy)
    }

    #[test]
    fn repair_kit_caps_at_max() {
        let _ = test_player(0, 100, 0, 100); // smoke
        assert_eq!(repair_into(0, 100), 25);
        assert_eq!(repair_into(80, 100), 100);
        assert_eq!(repair_into(100, 100), 100);
    }

    #[test]
    fn energy_cell_caps_at_max_energy() {
        assert_eq!(energy_into(0, 100), 100);
        assert_eq!(energy_into(50, 100), 100);
        assert_eq!(energy_into(100, 100), 100);
    }
}
```

- [ ] **Step 3: Re-export from `effects/mod.rs`**

Update `src/systems/powerups/effects/mod.rs`:

```rust
pub mod instant;
pub mod laser;
pub mod shockwave;
```

- [ ] **Step 4: Dispatch in `powerup_pickup_system`**

In `src/systems/powerups/mod.rs`, inside `powerup_pickup_system`, replace the temporary `PowerUpKind::RepairKit | ... => { // TODO }` arm with explicit arms:

The pickup system needs access to `&mut Player`. Update the player query to use `Query<(Entity, &Transform, &Sprite, &mut Player)>`:

```rust
pub fn powerup_pickup_system(
    mut commands: Commands,
    mut player_query: Query<(Entity, &Transform, &Sprite, &mut Player)>,
    // ... rest unchanged
)
```

Inside the loop, extract the mutable player once:

```rust
let Ok((player_entity, player_transform, player_sprite, mut player)) = player_query.single_mut() else {
    return;
};
let player_pos = player_transform.translation;
let player_size = player_sprite.custom_size.unwrap_or(Vec2::ONE);
```

Replace the old `Shockwave` and `Laser` arms with these, plus add Repair/EnergyCell:

```rust
match powerup.kind {
    PowerUpKind::Shockwave => {
        crate::systems::powerups::effects::shockwave::apply_shockwave(
            &mut commands,
            player_pos,
            &mut boss_query,
            &enemy_particle_query,
            &boss_projectile_query,
            &dash_trail_query,
            &hazard_zone_query,
            &telegraph_query,
            &mut screen_shake,
            &mut sound_events,
        );
    }
    PowerUpKind::Laser => {
        crate::systems::powerups::effects::laser::apply_laser_pickup(
            &mut commands,
            player_entity,
            player_pos,
            &mut screen_shake,
            &mut sound_events,
        );
    }
    PowerUpKind::RepairKit => {
        crate::systems::powerups::effects::instant::apply_repair_kit(
            &mut player,
            &mut sound_events,
        );
    }
    PowerUpKind::EnergyCell => {
        crate::systems::powerups::effects::instant::apply_energy_cell(
            &mut player,
            &mut sound_events,
        );
    }
    PowerUpKind::PhaseShift | PowerUpKind::GlitchBlink => {
        // Implemented in Tasks 9 and 10
    }
}
```

Note: `boss_projectile_query` may need its signature updated to `Query<Entity, With<BossProjectile>>` if the collision-routing change in Task 5 altered it. The shockwave apply_shockwave only needs entities for despawning. Look at the current query signature; the pickup system's `boss_projectile_query` is separate from `detect_collisions`' query and should remain `Query<Entity, With<BossProjectile>>`.

- [ ] **Step 5: Build + test**

Run: `cargo build`
Expected: compiles

Run: `cargo test --lib`
Expected: new instant tests pass: `repair_kit_caps_at_max`, `energy_cell_caps_at_max_energy`

- [ ] **Step 6: Manual smoke test**

Run: `cargo run --release`
Take damage, then pick up a green Repair Kit. HP goes up by 25 (capped at 100). Pick up a blue Energy Cell — energy refills to 100.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(powerups): implement Repair Kit and Energy Cell (common)"
```

---

## Task 9: Implement Phase Shift

**Files:**
- Create: `src/systems/powerups/effects/phase_shift.rs`
- Modify: `src/systems/powerups/effects/mod.rs`
- Modify: `src/systems/powerups/mod.rs` (pickup dispatch)
- Modify: `src/systems/collision.rs` (skip enemy-projectile branches)
- Modify: `src/systems/audio.rs` (1 new sound)
- Modify: `src/app.rs` (register tick system; cleanup on round exit)

- [ ] **Step 1: Add `PhaseShift` sound**

In `src/systems/audio.rs`, add to `SoundEffect`:

```rust
PhaseShift,
```

Add to `ALL_EFFECTS`.

Add to `generate_sound` match:

```rust
SoundEffect::PhaseShift => {
    // Reverse-shimmer: pitched-up ascending with noise, 0.2s
    let duration = 0.2;
    let num_samples = (sample_rate * duration) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate;
            let progress = t / duration;
            let freq = 600.0 + (600.0 * progress);
            let envelope = progress * (1.0 - progress) * 4.0;
            let noise = (rand::random::<f32>() * 2.0 - 1.0) * 0.15;
            let tone = (t * freq * std::f32::consts::TAU).sin() * 0.4;
            (noise + tone) * envelope * volume * 0.35
        })
        .collect()
}
```

- [ ] **Step 2: Create `effects/phase_shift.rs`**

Create `src/systems/powerups/effects/phase_shift.rs`:

```rust
use crate::core::player::components::Player;
use crate::systems::audio::{SoundEffect, SoundEvent};
use bevy::prelude::*;

pub const PHASE_SHIFT_DURATION: f32 = 2.0;

#[derive(Component)]
pub struct PhaseShiftActive(pub Timer);

/// Apply Phase Shift: add or refresh `PhaseShiftActive` on player; play sound.
/// If the component already exists, its timer is reset to full.
pub fn apply_phase_shift(
    commands: &mut Commands,
    player_entity: Entity,
    existing: Option<&mut PhaseShiftActive>,
    sound_events: &mut EventWriter<SoundEvent>,
) {
    let fresh = Timer::from_seconds(PHASE_SHIFT_DURATION, TimerMode::Once);
    if let Some(active) = existing {
        active.0 = fresh;
    } else {
        commands.entity(player_entity).insert(PhaseShiftActive(fresh));
    }
    sound_events.write(SoundEvent(SoundEffect::PhaseShift));
}

/// Tick timer and remove component when done. Also drives sprite alpha flicker.
pub fn phase_shift_tick_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut PhaseShiftActive, &mut Sprite), With<Player>>,
) {
    let t = time.elapsed_secs();
    for (entity, mut active, mut sprite) in query.iter_mut() {
        active.0.tick(time.delta());
        if active.0.finished() {
            commands.entity(entity).remove::<PhaseShiftActive>();
            // Restore base color (full alpha); approximated by preserving current color channels at alpha 1.0
            let [r, g, b, _a] = sprite.color.to_srgba().to_f32_array();
            sprite.color = Color::srgba(r, g, b, 1.0);
        } else {
            // Flicker alpha at 8 Hz between 0.35 and 0.75
            let flicker = 0.35 + 0.40 * (0.5 + 0.5 * (t * 8.0 * std::f32::consts::TAU).sin());
            let [r, g, b, _a] = sprite.color.to_srgba().to_f32_array();
            sprite.color = Color::srgba(r, g, b, flicker);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_shift_duration_matches_constant() {
        let timer = Timer::from_seconds(PHASE_SHIFT_DURATION, TimerMode::Once);
        assert!((timer.duration().as_secs_f32() - PHASE_SHIFT_DURATION).abs() < f32::EPSILON);
    }

    #[test]
    fn timer_reset_is_full_duration() {
        // Simulates the "refresh on re-pickup" rule in isolation
        let fresh = Timer::from_seconds(PHASE_SHIFT_DURATION, TimerMode::Once);
        assert_eq!(fresh.fraction(), 0.0);
        assert!(!fresh.finished());
    }
}
```

- [ ] **Step 3: Re-export from `effects/mod.rs`**

Update `src/systems/powerups/effects/mod.rs`:

```rust
pub mod instant;
pub mod laser;
pub mod phase_shift;
pub mod shockwave;
```

- [ ] **Step 4: Wire pickup dispatch**

In `src/systems/powerups/mod.rs`, update `powerup_pickup_system`:

Change the player query to include `Option<&mut PhaseShiftActive>`:

```rust
mut player_query: Query<(Entity, &Transform, &Sprite, &mut Player, Option<&mut crate::systems::powerups::effects::phase_shift::PhaseShiftActive>)>,
```

Extract after the `single_mut`:

```rust
let Ok((player_entity, player_transform, player_sprite, mut player, existing_phase_shift)) = player_query.single_mut() else {
    return;
};
let mut existing_phase_shift = existing_phase_shift;
```

Replace the `PhaseShift | GlitchBlink` TODO arm with:

```rust
PowerUpKind::PhaseShift => {
    crate::systems::powerups::effects::phase_shift::apply_phase_shift(
        &mut commands,
        player_entity,
        existing_phase_shift.as_deref_mut(),
        &mut sound_events,
    );
}
PowerUpKind::GlitchBlink => {
    // Implemented in Task 10
}
```

- [ ] **Step 5: Skip enemy-projectile and enemy-particle collisions during Phase Shift**

In `src/systems/collision.rs`, add a query param to `detect_collisions`:

```rust
phase_shift_query: Query<(), With<crate::systems::powerups::effects::phase_shift::PhaseShiftActive>>,
```

Add at the top of the player loop:

```rust
let phase_shifting = !phase_shift_query.is_empty();
```

In the `EnemyParticle vs Player` loop and the Boss-owned `BossProjectile vs Player` loop, add at the top of each:

```rust
if phase_shifting { continue; }
```

- [ ] **Step 6: Register tick system**

In `src/app.rs`, add to the power-ups system set (look for the block around line 227 that registers `powerup_spawn_system`):

```rust
.add_systems(
    Update,
    (
        powerup_spawn_system,
        powerup_lifetime_system,
        powerup_pickup_system,
        laser_system,
        powerup_shockwave_system,
        laser_charge_particle_system,
        laser_charge_orb_system,
        laser_stream_particle_system,
        laser_impact_system,
        crate::systems::powerups::effects::phase_shift::phase_shift_tick_system,
    )
        .run_if(in_state(GameState::RoundActive)),
)
```

- [ ] **Step 7: Cleanup on round exit**

Create a tiny cleanup system at the top of `src/systems/powerups/mod.rs`:

```rust
use crate::systems::powerups::effects::phase_shift::PhaseShiftActive;

pub fn cleanup_player_buffs_on_round_exit(
    mut commands: Commands,
    player_query: Query<Entity, With<crate::core::player::components::Player>>,
) {
    for entity in player_query.iter() {
        commands.entity(entity).remove::<PhaseShiftActive>();
    }
}
```

Register in `src/app.rs`:

```rust
.add_systems(
    OnExit(GameState::RoundActive),
    crate::systems::powerups::cleanup_player_buffs_on_round_exit,
)
```

- [ ] **Step 8: Build + test**

Run: `cargo build`
Expected: compiles

Run: `cargo test --lib`
Expected: new `phase_shift` tests pass

- [ ] **Step 9: Manual smoke test**

Run: `cargo run --release`
Pick up a white Phase Shift power-up during a projectile barrage. For 2 seconds, enemy projectiles pass through you harmlessly; player sprite flickers. Dash trails and hazards still hurt. After 2s, flicker ends and damage resumes.

- [ ] **Step 10: Commit**

```bash
git add -A
git commit -m "feat(powerups): implement Phase Shift (common)"
```

---

## Task 10: Implement Glitch Blink

**Files:**
- Create: `src/systems/powerups/effects/blink.rs`
- Modify: `src/systems/powerups/effects/mod.rs`
- Modify: `src/systems/powerups/mod.rs` (pickup dispatch, particle animator)
- Modify: `src/systems/audio.rs` (1 new sound)
- Modify: `src/app.rs` (register particle tick system)

- [ ] **Step 1: Add `GlitchBlink` sound**

In `src/systems/audio.rs`, add to `SoundEffect`:

```rust
GlitchBlink,
```

Add to `ALL_EFFECTS`.

Add to `generate_sound`:

```rust
SoundEffect::GlitchBlink => {
    // Digital glitch burst: stuttering noise + pitched-up blips, 0.15s
    let duration = 0.15;
    let num_samples = (sample_rate * duration) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate;
            let envelope = 1.0 - (t / duration);
            let stutter = if ((t * 120.0) as u32) % 2 == 0 { 1.0 } else { 0.0 };
            let noise = (rand::random::<f32>() * 2.0 - 1.0) * 0.4 * stutter;
            let blip = (t * 1600.0 * std::f32::consts::TAU).sin() * 0.4 * stutter;
            (noise + blip) * envelope * volume * 0.35
        })
        .collect()
}
```

- [ ] **Step 2: Create `effects/blink.rs` with `pick_safe_spot` + particle + tests**

Create `src/systems/powerups/effects/blink.rs`:

```rust
use crate::app::GameEntity;
use crate::systems::audio::{SoundEffect, SoundEvent};
use crate::utils::config::{ENTITY_SCALE, QualityTier};
use bevy::prelude::*;
use rand::Rng;

pub const BLINK_MIN_BOSS_DIST: f32 = 200.0;
pub const BLINK_CANDIDATES: usize = 20;
pub const BLINK_BOUNDS: (f32, f32, f32, f32) = (-550.0, 550.0, -300.0, 300.0);

#[derive(Component)]
pub struct BlinkParticle {
    pub lifetime: Timer,
}

/// Choose a teleport destination minimizing a threat-weighted score.
/// Pure function — testable without a Bevy world.
pub fn pick_safe_spot(
    boss_pos: Vec2,
    threats: &[Vec2],
    bounds: (f32, f32, f32, f32),
    candidates: &[Vec2],
) -> Vec2 {
    // Filter candidates that meet the boss-distance minimum
    let mut filtered: Vec<Vec2> = candidates
        .iter()
        .copied()
        .filter(|c| (*c - boss_pos).length() >= BLINK_MIN_BOSS_DIST)
        .collect();

    // Fallback: if none pass the hard gate, use farthest corner from boss
    if filtered.is_empty() {
        let (x_min, x_max, y_min, y_max) = bounds;
        let corners = [
            Vec2::new(x_min, y_min),
            Vec2::new(x_min, y_max),
            Vec2::new(x_max, y_min),
            Vec2::new(x_max, y_max),
        ];
        return corners
            .into_iter()
            .max_by(|a, b| {
                let da = (*a - boss_pos).length();
                let db = (*b - boss_pos).length();
                da.partial_cmp(&db).unwrap()
            })
            .expect("4 corners always present");
    }

    // Score each candidate (lower = safer)
    filtered.sort_by(|a, b| {
        let score_a = score_candidate(*a, boss_pos, threats);
        let score_b = score_candidate(*b, boss_pos, threats);
        score_a.partial_cmp(&score_b).unwrap()
    });
    filtered[0]
}

fn score_candidate(candidate: Vec2, boss_pos: Vec2, threats: &[Vec2]) -> f32 {
    let mut score = 0.0;
    for threat in threats {
        let d = (candidate - *threat).length();
        score += 1.0 / (d + 10.0);
    }
    score += 1.0 / ((candidate - boss_pos).length() + 10.0);
    score
}

/// Sample N random candidates in bounds.
pub fn sample_candidates(bounds: (f32, f32, f32, f32), n: usize) -> Vec<Vec2> {
    let mut rng = rand::thread_rng();
    let (x_min, x_max, y_min, y_max) = bounds;
    (0..n)
        .map(|_| {
            Vec2::new(
                rng.gen_range(x_min..x_max),
                rng.gen_range(y_min..y_max),
            )
        })
        .collect()
}

/// Spawn a lightning-burst of N particles at `pos` (for blink origin + destination).
pub fn spawn_blink_burst(
    commands: &mut Commands,
    pos: Vec3,
    particle_count: usize,
) {
    let mut rng = rand::thread_rng();
    for _ in 0..particle_count {
        let angle = rng.gen_range(0.0..std::f32::consts::TAU);
        let length = rng.gen_range(6.0..14.0) * ENTITY_SCALE;
        let offset = Vec2::new(angle.cos(), angle.sin()) * length;
        commands.spawn((
            Sprite {
                color: Color::srgba(6.0, 0.5, 8.0, 0.9),
                custom_size: Some(Vec2::new(2.0, length)),
                ..default()
            },
            Transform::from_translation(pos.with_z(0.55) + offset.extend(0.0))
                .with_rotation(Quat::from_rotation_z(angle)),
            BlinkParticle {
                lifetime: Timer::from_seconds(0.2, TimerMode::Once),
            },
            GameEntity,
        ));
    }
}

/// Particle count depends on quality tier.
pub fn blink_particle_count(quality: &QualityTier) -> usize {
    match quality {
        QualityTier::Desktop => 16,
        QualityTier::Mobile => 6,
    }
}

pub fn blink_particle_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut BlinkParticle, &mut Sprite)>,
) {
    for (entity, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime.tick(time.delta());
        if particle.lifetime.finished() {
            commands.entity(entity).despawn();
            continue;
        }
        let progress = particle.lifetime.fraction();
        let alpha = (1.0 - progress) * 0.9;
        let [r, g, b, _a] = sprite.color.to_srgba().to_f32_array();
        sprite.color = Color::srgba(r, g, b, alpha);
    }
}

/// Sound effect for blink pickup.
pub fn play_blink_sound(sound_events: &mut EventWriter<SoundEvent>) {
    sound_events.write(SoundEvent(SoundEffect::GlitchBlink));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_candidate_far_from_threats() {
        let boss_pos = Vec2::new(0.0, 0.0);
        let threats = vec![Vec2::new(300.0, 0.0), Vec2::new(-300.0, 0.0)];
        let candidates = vec![
            Vec2::new(300.0, 5.0),  // very close to a threat
            Vec2::new(-280.0, 10.0), // very close to other threat
            Vec2::new(0.0, 400.0),  // far from both (passes boss-dist gate)
        ];
        let bounds = (-550.0, 550.0, -300.0, 300.0);
        let result = pick_safe_spot(boss_pos, &threats, bounds, &candidates);
        assert_eq!(result, Vec2::new(0.0, 400.0));
    }

    #[test]
    fn filters_candidates_too_close_to_boss() {
        let boss_pos = Vec2::new(0.0, 0.0);
        let threats = vec![];
        // Two candidates: one within 200px of boss (filtered), one outside (kept)
        let candidates = vec![Vec2::new(50.0, 50.0), Vec2::new(250.0, 0.0)];
        let bounds = (-550.0, 550.0, -300.0, 300.0);
        let result = pick_safe_spot(boss_pos, &threats, bounds, &candidates);
        assert_eq!(result, Vec2::new(250.0, 0.0));
    }

    #[test]
    fn fallback_to_corner_when_all_too_close() {
        let boss_pos = Vec2::new(0.0, 0.0);
        let threats = vec![];
        // All candidates too close to boss
        let candidates = vec![Vec2::new(50.0, 50.0), Vec2::new(-50.0, 50.0)];
        let bounds = (-550.0, 550.0, -300.0, 300.0);
        let result = pick_safe_spot(boss_pos, &threats, bounds, &candidates);
        // Farthest corner from (0,0) is one of the four — expect length ~= sqrt(550^2 + 300^2)
        let dist = (result - boss_pos).length();
        assert!(dist > BLINK_MIN_BOSS_DIST, "result dist = {}", dist);
    }

    #[test]
    fn particle_count_varies_by_quality() {
        assert_eq!(blink_particle_count(&QualityTier::Desktop), 16);
        assert_eq!(blink_particle_count(&QualityTier::Mobile), 6);
    }
}
```

- [ ] **Step 3: Re-export from `effects/mod.rs`**

```rust
pub mod blink;
pub mod instant;
pub mod laser;
pub mod phase_shift;
pub mod shockwave;
```

- [ ] **Step 4: Wire pickup dispatch**

In `src/systems/powerups/mod.rs`, the pickup system needs access to: boss position (to compute safe-spot distance), threat entities, and the `QualityTier` resource.

Add new params to `powerup_pickup_system`:

```rust
boss_transform_query: Query<&Transform, (With<Boss>, Without<Player>, Without<PowerUp>)>,
all_enemy_particles_xform: Query<&Transform, (With<EnemyParticle>, Without<Player>)>,
all_boss_projectiles_xform: Query<&Transform, (With<BossProjectile>, Without<Player>)>,
all_hazards_xform: Query<&Transform, (With<HazardZone>, Without<Player>)>,
all_dash_trails_xform: Query<&Transform, (With<DashTrail>, Without<Player>)>,
quality: Res<QualityTier>,
```

Note: these are **separate queries** from the ones used for Shockwave's despawn logic — those take `Entity` only. Don't combine.

Add imports at the top of `src/systems/powerups/mod.rs`:

```rust
use crate::utils::config::QualityTier;
```

Replace the GlitchBlink arm:

```rust
PowerUpKind::GlitchBlink => {
    use crate::systems::powerups::effects::blink;

    // Collect threat positions
    let mut threats: Vec<Vec2> = Vec::new();
    for t in all_enemy_particles_xform.iter() {
        threats.push(t.translation.truncate());
    }
    for t in all_boss_projectiles_xform.iter() {
        threats.push(t.translation.truncate());
    }
    for t in all_hazards_xform.iter() {
        threats.push(t.translation.truncate());
    }
    for t in all_dash_trails_xform.iter() {
        threats.push(t.translation.truncate());
    }

    let boss_pos = boss_transform_query
        .single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    let candidates = blink::sample_candidates(blink::BLINK_BOUNDS, blink::BLINK_CANDIDATES);
    let destination = blink::pick_safe_spot(boss_pos, &threats, blink::BLINK_BOUNDS, &candidates);

    // Burst at old position + new
    let particles = blink::blink_particle_count(&quality);
    blink::spawn_blink_burst(&mut commands, player_pos, particles);

    // Teleport
    commands
        .entity(player_entity)
        .insert(Transform::from_translation(destination.extend(0.0))
            .with_rotation(player_transform.rotation));

    blink::spawn_blink_burst(&mut commands, destination.extend(0.0), particles);
    blink::play_blink_sound(&mut sound_events);
}
```

Note: Bevy 0.16 `single()` returns a `Result`; adjust `.map` + `.unwrap_or` as above or use `.ok()`.

- [ ] **Step 5: Register particle tick system**

In `src/app.rs`, add to the powerup system set:

```rust
crate::systems::powerups::effects::blink::blink_particle_system,
```

- [ ] **Step 6: Build + test**

Run: `cargo build`
Expected: compiles

Run: `cargo test --lib blink`
Expected: 4 tests pass (picks_candidate_far_from_threats, filters_candidates_too_close_to_boss, fallback_to_corner_when_all_too_close, particle_count_varies_by_quality)

Run: `cargo test --lib`
Expected: all tests pass

- [ ] **Step 7: Manual smoke test**

Run: `cargo run --release`
Pick up a purple Glitch Blink power-up in a crowded projectile situation. Player teleports away from bullets. Purple lightning particles burst at both origin and destination.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat(powerups): implement Glitch Blink (common)"
```

---

## Task 11: HUD active-buff indicator row

**Files:**
- Create: `src/systems/powerups/hud.rs`
- Modify: `src/systems/powerups/mod.rs` (re-export)
- Modify: `src/ui/hud.rs` (add mount node for buffs)
- Modify: `src/app.rs` (register buff HUD sync system)

- [ ] **Step 1: Add a `BuffHudRoot` mount node to the HUD**

In `src/ui/hud.rs`, inside `spawn_hud` after the `BossHpFill` row (search for `top.spawn(Node { flex_direction: FlexDirection::Row,` that creates phase pips row, and add below it):

```rust
// Active-buff indicator row
top.spawn((
    Node {
        flex_direction: FlexDirection::Row,
        margin: UiRect::top(Val::Px(6.0)),
        column_gap: Val::Px(6.0),
        ..default()
    },
    crate::systems::powerups::hud::BuffHudRoot,
));
```

Add to the component markers section at the top of `src/ui/hud.rs`:

```rust
// (no declaration here — BuffHudRoot is defined in powerups/hud.rs and used via fully-qualified path)
```

- [ ] **Step 2: Create `powerups/hud.rs`**

Create `src/systems/powerups/hud.rs`:

```rust
use crate::core::player::components::Player;
use crate::systems::powerups::effects::phase_shift::{PHASE_SHIFT_DURATION, PhaseShiftActive};
use bevy::prelude::*;

/// Marker on the UI node where buff dots are children.
#[derive(Component)]
pub struct BuffHudRoot;

/// Marker on each dot so we can despawn them on refresh.
#[derive(Component)]
pub struct BuffHudDot;

/// Sync the buff row: each frame, clear existing dots and respawn per active buff.
/// Kept simple because there are ≤ 6 possible dots and sync happens at 60fps — cheap.
pub fn sync_buff_hud_system(
    mut commands: Commands,
    root_query: Query<Entity, With<BuffHudRoot>>,
    existing_dots: Query<Entity, With<BuffHudDot>>,
    player_query: Query<&PhaseShiftActive, With<Player>>,
) {
    let Ok(root) = root_query.single() else { return };

    // Despawn previous-frame dots
    for entity in existing_dots.iter() {
        commands.entity(entity).despawn();
    }

    // Enumerate currently active buffs and spawn a dot per buff
    for phase_shift in player_query.iter() {
        let remaining = (1.0 - phase_shift.0.fraction()).clamp(0.0, 1.0);
        let meta = crate::systems::powerups::catalog::meta(
            crate::systems::powerups::catalog::PowerUpKind::PhaseShift,
        );
        spawn_buff_dot(&mut commands, root, meta.color, remaining);
    }
}

fn spawn_buff_dot(commands: &mut Commands, root: Entity, color: Color, remaining: f32) {
    commands.entity(root).with_children(|parent| {
        parent
            .spawn((
                Node {
                    width: Val::Px(14.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BuffHudDot,
            ))
            .with_children(|col| {
                // The dot
                col.spawn((
                    Node {
                        width: Val::Px(12.0),
                        height: Val::Px(12.0),
                        ..default()
                    },
                    BackgroundColor(color),
                ));
                // Duration bar
                col.spawn((
                    Node {
                        width: Val::Px(14.0),
                        height: Val::Px(2.0),
                        margin: UiRect::top(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.04, 0.04, 0.04)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: Val::Percent(remaining * 100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(color),
                    ));
                });
            });
    });
}
```

- [ ] **Step 3: Re-export `BuffHudRoot` from `powerups/mod.rs`**

In `src/systems/powerups/mod.rs`, add:

```rust
pub mod hud;
```

- [ ] **Step 4: Register sync system**

In `src/app.rs`, add to the powerups Update set:

```rust
crate::systems::powerups::hud::sync_buff_hud_system,
```

- [ ] **Step 5: Build**

Run: `cargo build`
Expected: compiles

- [ ] **Step 6: Manual smoke test**

Run: `cargo run --release`
Pick up a Phase Shift. A small white-ish dot appears under the boss HP bar with a bar that shrinks over 2 seconds. Disappears when the effect ends.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat(powerups): HUD active-buff indicator row"
```

---

## Task 12: Phase 1 integration — smoke test + final commit

**Files:** none modified

- [ ] **Step 1: Full build + test**

Run: `cargo build --release`
Expected: compiles

Run: `cargo test --lib`
Expected: all tests pass, including:
- `catalog::tests::every_kind_has_a_catalog_entry`
- `catalog::tests::tier_boundaries`
- `catalog::tests::roll_distribution_within_tolerance`
- `effects::laser::tests::test_laser_total_duration`
- `effects::laser::tests::test_laser_phase_from_elapsed`
- `effects::instant::tests::repair_kit_caps_at_max`
- `effects::instant::tests::energy_cell_caps_at_max_energy`
- `effects::phase_shift::tests::phase_shift_duration_matches_constant`
- `effects::phase_shift::tests::timer_reset_is_full_duration`
- `effects::blink::tests::picks_candidate_far_from_threats`
- `effects::blink::tests::filters_candidates_too_close_to_boss`
- `effects::blink::tests::fallback_to_corner_when_all_too_close`
- `effects::blink::tests::particle_count_varies_by_quality`
- existing `systems::collision::tests::*`
- `core::player::components::tests::player_default_shape_compiles`

- [ ] **Step 2: Manual smoke checklist**

Run: `cargo run --release`
For each power-up, find one in a round and verify:

- [ ] **Repair Kit (green)** — common size, slow pulse, no glow. HP increases by 25.
- [ ] **Energy Cell (blue)** — common size, slow pulse, no glow. Energy refills to 100.
- [ ] **Phase Shift (white)** — common size, slow pulse, no glow. 2s projectile immunity; sprite flickers; HUD shows a white-ish dot.
- [ ] **Glitch Blink (purple)** — common size, slow pulse, no glow. Player teleports; purple particles burst at both ends.
- [ ] **Shockwave (cyan)** — rare size (larger), faster pulse, white glow ring. Screen clears; boss takes 20 dmg.
- [ ] **Laser (magenta)** — ultra-rare size (largest), fastest pulse, gold glow ring. Charge → beam → fade. Identical to pre-refactor.

For each timed buff (currently only Phase Shift), verify:
- [ ] Re-picking same buff refreshes timer (no stacking; dot duration bar resets to full)
- [ ] Round end cleans up the component (next round starts clean)

- [ ] **Step 3: Mobile quality-tier spot check**

Run with mobile tier forced (see `src/utils/config.rs` for `QualityTier`; either edit the default or run on a mobile device).

- [ ] Glitch Blink burst shows 6 particles per endpoint (not 16)
- [ ] No noticeable frame-rate regression during normal play

- [ ] **Step 4: Phase 1 completion commit**

If any smoke-test fixes were made during Step 2–3, commit them:

```bash
git add -A
git commit -m "chore(powerups): Phase 1 smoke-test fixes"
```

If no fixes needed, no commit.

The next commit message to reference Phase 1 completion (for the next plan) can be the tip of branch here.

---

## End of Phase 1 Plan

After this plan ships:
- Game has 6 power-ups (4 new commons + existing Shockwave + Laser)
- Catalog, tier-weighted roll, tier-based visuals, and HUD infrastructure in place
- `ProjectileOwner` enum and `max_energy` field ready for Phase 2 and 3 power-ups

The next plan (Phase 2) adds the rare tier (Overclock, Shield, Bullet Time, Decoy, Gravity Well) and introduces `EnemyTimeScale` + decoy targeting. Phase 3 adds Reflector, Hack, and Missile Swarm.
