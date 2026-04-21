# More Power-Ups — Design

## Overview

Expand the power-up system from 2 items (Shockwave, Laser) to 14, organized into three rarity tiers with tier-first weighted spawning. The new items cover offensive, defensive, movement, and "weird tech" roles. Existing Shockwave and Laser move into the new catalog as rare and ultra-rare respectively, unchanged in behavior.

The change ships in three phases. After Phase 1 the game has 6 power-ups; after Phase 2, 11; after Phase 3, all 14.

## Scope

**In scope:**
- 12 new power-up types with distinct effects and visuals
- Rarity tier system (Common / Rare / Ultra-rare) with weighted spawning
- Tier-based pickup styling (size, pulse rate, glow ring)
- HUD row showing active timed buffs with remaining-duration bars
- Refactor of `src/systems/powerups.rs` into a module with per-effect files
- Small refactor: add `ProjectileOwner` enum to `BossProjectile`
- Mobile quality-tier degradations for expensive new visuals (bullet time overlay, hack RGB split, missile trails, etc.)
- Unit tests for pure-function pieces (rarity rolling, safe-spot scoring, homing math, ownership flip, timer refresh)

**Out of scope:**
- New boss mechanics or attacks
- Rebalancing existing Shockwave / Laser numbers (only their tier categorization changes)
- Achievements or unlocks around power-up use
- Stacking semantics beyond "refresh on re-pickup"
- Per-round power-up selection / limits

## Power-Up Catalog

All visuals are diamond-shaped sprites (45° rotated square), styled by tier (see "Pickup Visual Styling"). Colors are HDR (values >1.0) to read in the existing bloom pipeline.

### Common tier — frequent, mild single-hit effects

| # | Name | Effect | Color (HDR `srgb`) |
|---|---|---|---|
| 1 | **Repair Kit** | Instantly restore 25 HP (clamped to `max`) | `(0.0, 8.0, 2.0)` bright green |
| 2 | **Energy Cell** | Instantly +100 energy (saturating) | `(0.0, 4.0, 8.0)` electric blue |
| 3 | **Phase Shift** | 2s immunity to enemy projectiles (boss body + hazards still hurt) | `(6.0, 6.0, 8.0)` translucent white, alpha 0.7 |
| 4 | **Glitch Blink** | Instant teleport to the safest nearby spot | `(6.0, 0.5, 8.0)` electric purple |

### Rare tier — noticeable mid-duration buffs

| # | Name | Effect | Color |
|---|---|---|---|
| 5 | **Overclock** | 2.5× fire rate and 0 energy cost for 6s | `(8.0, 7.0, 0.0)` yellow |
| 6 | **Shield** | Absorb 3 hits or expire at 10s (whichever first) | `(8.0, 3.0, 0.0)` orange |
| 7 | **Bullet Time** | Enemy/projectile/hazard time scale × 0.25 for 4s; player unaffected | `(4.0, 0.0, 6.0)` deep purple |
| 8 | **Decoy Clone** | Fake player at mirrored position for 8s; boss targeting prefers decoy | `(6.0, 6.0, 6.0)` silver |
| 9 | **Gravity Well** | All enemy projectiles pulled into a vanishing point near player for 5s, despawn on arrival | `(3.0, 0.0, 5.0)` dark violet |
| — | **Shockwave** *(existing)* | Unchanged: screen clear + 20 dmg + shake | `(0.0, 8.0, 8.0)` cyan |

### Ultra-rare tier — game-changing moments

| # | Name | Effect | Color |
|---|---|---|---|
| 10 | **Reflector** | 5s bubble (~80px diameter) around player; enemy projectiles hitting it bounce back as Player-owned, 10 dmg on boss hit | `(8.0, 6.0, 0.0)` gold |
| 11 | **Hack** | 3s where boss's newly-fired projectiles damage the boss instead of the player (2 dmg per hit) | `(4.0, 8.0, 0.0)` glitch green |
| 12 | **Missile Swarm** | Spawn 6 homing missiles; each deals 10 dmg on boss hit; 8s lifetime | `(8.0, 1.0, 0.0)` warning red |
| — | **Laser** *(existing)* | Unchanged: 6.8s charge/active/fade beam | `(8.0, 0.0, 8.0)` magenta |

## Rarity & Spawn Model

### Tier weighting

Spawn roll is tier-first, then uniform within tier.

- Tier roll: **55% Common / 35% Rare / 10% Ultra-rare**
- Per-item odds: Common ~13.75% each, Rare ~5.83% each, Ultra-rare ~2.5% each

Expected per fight (~60s, ~3 pickups at 15–20s spawn cadence): **~1.6 commons, ~1 rare, ~0.3 ultra-rares**.

### Spawn cadence (unchanged from current)

- `PowerUpTimer` resource with timer set to random 15–20s
- Only 1 power-up on screen at a time — skip spawn if one already exists
- On-screen lifetime: 10s before despawn
- Spawn position: random within `x ∈ [-500, 500]`, `y ∈ [-200, 200]`

### Pickup visual styling (tier at a glance)

| Tier | Size | Pulse rate | Glow ring |
|---|---|---|---|
| Common | 14px diamond | 4 Hz | none |
| Rare | 18px diamond | 6 Hz | thin white ring (1.5× sprite size, low alpha) |
| Ultra-rare | 22px diamond | 9 Hz | gold ring (1.8× sprite size) + subtle rotating particle halo |

The glow ring is implemented as a second sprite child of the pickup entity, scaled up from the main diamond.

## Architecture

### File layout

Split `src/systems/powerups.rs` into a module:

```
src/systems/powerups/
  mod.rs           — module root, pub re-exports, plugin wiring
  catalog.rs       — PowerUpKind enum, PowerUpTier enum, metadata table, weighted roll
  spawn.rs         — spawn timer, pickup entity spawning, pulse/glow animation
  pickup.rs        — collision detection + dispatch to effect modules
  hud.rs           — active-buff indicator row
  effects/
    mod.rs         — shared: EnemyTimeScale, HackActive, ProjectileOwner, DecoyTarget
    instant.rs     — Repair Kit, Energy Cell
    phase_shift.rs
    blink.rs
    overclock.rs
    shield.rs
    shockwave.rs   — existing code, unchanged behavior
    laser.rs       — existing code, unchanged behavior
    bullet_time.rs
    decoy.rs
    gravity_well.rs
    reflector.rs
    hack.rs
    missile_swarm.rs
```

Existing Laser code (~600 lines) and Shockwave code move verbatim into their new files; only import paths shift.

### Core pattern 1 — Catalog with metadata

Single source of truth for color, tier, and display name. Used by spawn, pickup, and HUD.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpKind {
    // Common
    RepairKit, EnergyCell, PhaseShift, GlitchBlink,
    // Rare
    Overclock, Shield, BulletTime, DecoyClone, GravityWell, Shockwave,
    // Ultra-rare
    Reflector, Hack, MissileSwarm, Laser,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerUpTier { Common, Rare, UltraRare }

pub struct PowerUpMeta {
    pub kind: PowerUpKind,
    pub tier: PowerUpTier,
    pub color: Color,
    pub display_name: &'static str,
}

pub const CATALOG: &[PowerUpMeta] = &[ /* 14 entries */ ];

pub fn meta(kind: PowerUpKind) -> &'static PowerUpMeta { /* … */ }
pub fn roll_random_kind() -> PowerUpKind { /* tier-first roll */ }
```

### Core pattern 2 — Timed player buffs

Timed buffs are components attached to the Player entity, each with its own tick system.

```rust
#[derive(Component)] pub struct PhaseShiftActive(pub Timer);
#[derive(Component)] pub struct OverclockActive(pub Timer);
#[derive(Component)] pub struct ShieldCharges { pub n: u8, pub timer: Timer }
#[derive(Component)] pub struct ReflectorActive(pub Timer);
```

**Refresh-on-re-pickup rule** (no stacking): picking up the same buff while active resets its timer to full and, for Shield, resets `n` to 3. No stacking of duration or charges.

### Core pattern 3 — World-state effects (resources)

```rust
#[derive(Resource)]
pub struct EnemyTimeScale { pub scale: f32, pub timer: Timer }
// Initialized at startup with scale = 1.0 and a finished Timer. Stays inserted for lifetime of the app.

#[derive(Resource)]
pub struct HackActive { pub timer: Timer }
// Inserted on Hack pickup, removed when timer finishes. Presence = hack is active.
```

Systems affected by `EnemyTimeScale` multiply their effective `dt` by `scale`. When the timer expires, `scale` resets to 1.0.

`HackActive` uses the insert-on-pickup / remove-on-expire pattern; systems check for its presence with `Option<Res<HackActive>>`.

### Core pattern 4 — Projectile ownership

Add to existing `BossProjectile`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileOwner { Boss, Player }

pub struct BossProjectile {
    pub velocity: Vec2,
    pub damage: u32,
    pub owner: ProjectileOwner,   // NEW; default Boss at all existing spawn sites
}
```

All existing spawn sites in `src/core/boss/attacks.rs` and related files explicitly set `owner: ProjectileOwner::Boss` as part of Phase 1. Only Hack and Reflector produce `Player`-owned projectiles. `detect_collisions` reads `owner` to route damage (Boss → player; Player → boss; Player-owned never damages player).

### Affected existing systems (read-only touches)

| System | Reads | Purpose |
|---|---|---|
| `player_shoot_system` | `OverclockActive` | faster cooldown, zero energy cost while active |
| `detect_collisions` | `PhaseShiftActive`, `ShieldCharges`, `ReflectorActive`, `DecoyTarget`, `ProjectileOwner` | skip / absorb / reflect / retarget / owner-based damage |
| `boss_attack_system` | `DecoyTarget`, `HackActive` | aim at decoy when present; set `Player` owner on new projectiles during hack |
| `boss_projectile_system` | `EnemyTimeScale`, any `GravityAttractor` entity | multiply `dt` during bullet time; curve toward attractor during gravity well |
| `hazard_zone_system` | `EnemyTimeScale` | multiply `dt` |
| `dash_trail` lifetime | `EnemyTimeScale` | multiply `dt` |
| `particle_movement_system` (enemy particles) | `EnemyTimeScale`, any `GravityAttractor` entity | multiply `dt`; steer toward attractor |
| `boss_idle_movement` | `EnemyTimeScale` | multiply `dt` |

### HUD — active-buff indicator row

Under the boss HP bar: a horizontal row of up to 6 colored dots, one per active timed buff. Each dot:
- Diameter 12px, color from catalog
- Thin horizontal bar beneath it shrinking left-to-right with remaining duration
- For `ShieldCharges`, show the remaining charge count as small text "3" / "2" / "1" overlaid

Updated each frame by a system that queries all active buff components on the player and active world-state resources.

## Per-Power-Up Implementation Notes

### Trivial effects

**Repair Kit**: `player.current = (player.current + 25).min(player.max);` Visual: brief green flash on player sprite via `DamageFlash` reused with green color, or dedicated `HealFlash` component.

**Energy Cell**: `player.energy = (player.energy + 100).min(player.max_energy);` — note: `Player` currently has no `max_energy` field (see `src/core/player/components.rs`). Add `pub max_energy: u32` to `Player` struct, default 100, initialized at all player spawn sites. Visual: brief blue flash + audio zap.

### Phase Shift

- Add `PhaseShiftActive(Timer::from_seconds(2.0, Once))` to player on pickup
- In `detect_collisions`: skip the `EnemyParticle vs Player` and `BossProjectile vs Player` branches when component is present; boss body + hazards + dash trails still check
- Visual: player sprite pulses translucent (alpha 0.5, flicker at 8 Hz)
- Tick system: remove component when timer finishes

### Glitch Blink

```rust
fn pick_safe_spot(
    player_pos: Vec3,
    boss_pos: Vec3,
    threats: &[Vec3],       // projectiles + hazards + dash trails
    bounds: (f32, f32, f32, f32),  // x_min, x_max, y_min, y_max
) -> Vec3 {
    const CANDIDATES: usize = 20;
    const MIN_BOSS_DIST: f32 = 200.0;
    // Sample 20 random points in bounds
    // Filter: dist(candidate, boss_pos) >= MIN_BOSS_DIST
    // Score: sum(1.0 / (dist(candidate, threat) + 10.0)) for all threats
    //        + 1.0 / (dist(candidate, boss_pos) + 10.0)
    // Pick lowest score
    // Fallback if none pass the hard gate: corner farthest from boss
}
```

Visual: purple lightning burst (6 short particle streaks) at both old and new positions, 0.2s lifetime.

### Overclock

- `OverclockActive(Timer::from_seconds(6.0, Once))` on player
- `player_shoot_system` reads it:
  - `SHOT_COOLDOWN` effective value: `0.15 / 2.5 = 0.06s`
  - Skip energy subtraction entirely
- Visual: yellow chevron glow around player (child sprite)
- Tick system removes component on expire

### Shield

- `ShieldCharges { n: 3, timer: Timer::from_seconds(10.0, Once) }` on player
- In `detect_collisions` for each damage path (enemy particles, projectiles, hazards, dash trails, boss body):
  - If `ShieldCharges` present and `n > 0`: consume 1 charge, cancel the damage, spawn a small flash at collision point
  - If `n == 0` after consume: remove component
- Visual: orange ring child sprite, size scales down with remaining charges (larger at 3, smaller at 1)
- Tick system removes component on timer expire

### Bullet Time

- On pickup: set `EnemyTimeScale { scale: 0.25, timer: Timer::from_seconds(4.0, Once) }`
- Tick system: when timer finishes, reset `scale = 1.0`
- Affected systems (listed in "Affected existing systems") multiply `dt` by `scale` inside their time-dependent updates
- **Critical**: `Time::delta()` is global; do not multiply the global Time. Each affected system reads the resource and multiplies locally: `let scaled_dt = time.delta().as_secs_f32() * enemy_time_scale.scale;`
- Visual: blue tint full-screen overlay (simple colored quad, alpha 0.08) + chromatic aberration (desktop only)
- Audio: low-pitched drone via new `SoundEffect::BulletTimeDrone` (1-second looped sample, played on repeat during effect)

### Decoy Clone

- Spawn `DecoyPlayer` entity at mirror position: `decoy_pos = -player_pos.truncate()` (reflect through origin)
- Entity has: sprite identical to player at 70% alpha, flicker animation, `DecoyTarget` marker, `Timer::from_seconds(8.0, Once)` component
- Boss targeting: in `boss_attack_system`, when computing the target position for aimed attacks (dash, charge, aimed projectile spawn direction), if any `DecoyTarget` entity exists AND `HackActive` is NOT present, use decoy position; otherwise use player position (see Hack section for why hack bypasses decoy)
- In-flight telegraphs are **not** retargeted — they keep their original target captured at telegraph time
- Collision: in `detect_collisions`, only `Boss`-owned `EnemyParticle` and `BossProjectile` entities check against decoy position. A hit despawns the projectile and flashes the decoy red (no damage to player). `Player`-owned projectiles (reflected / hacked) pass through decoy freely. Dash trails, hazard zones, and boss body do NOT interact with decoy — they only threaten the real player.
- On expire or round end: despawn decoy entity

### Gravity Well

- On pickup: spawn `GravityAttractor` entity at `player_pos + forward * 50px`, with `Timer::from_seconds(5.0, Once)` component
- Sprite: dark violet swirling vortex, rotating 180°/s, ~40px diameter, with child particle spawner
- `boss_projectile_system` and enemy `particle_movement_system` check for an active `GravityAttractor`:
  - Apply acceleration toward attractor: `velocity += (attractor_pos - pos).normalize() * 400.0 * dt`
  - If `dist(projectile, attractor) < 10.0`: despawn projectile
- Does NOT affect boss body, hazards, or dash trails — only enemy projectiles and enemy particles
- On expire: despawn attractor + its child visuals

### Reflector

- Add `ReflectorActive(Timer::from_seconds(5.0, Once))` to player
- Spawn `ReflectorBubble` entity as child of player: gold ring sprite, 80px diameter
- In `detect_collisions`: when checking enemy projectile vs player, first check vs bubble (distance from player center ≤ 40px). If hit:
  - Reverse `projectile.velocity`
  - Set `projectile.owner = ProjectileOwner::Player`
  - Set `projectile.damage = 10` (rewards reflection)
  - Play bounce sound
- Player-owned projectile hitting boss (already handled via `ProjectileOwner` routing) deals its damage
- On expire: remove component, despawn bubble

### Hack

- Insert `HackActive { timer: Timer::from_seconds(3.0, Once) }` resource
- While active, in `boss_attack_system`:
  - **Aim override**: projectile aim targets **boss center** (not player, not decoy). This ensures the boss's own projectiles fly back through its hitbox.
  - Newly spawned `BossProjectile`: `owner = ProjectileOwner::Player`, `damage = 2`
- When inactive: projectiles spawn normally (`owner = Boss`, aim at player or decoy)
- In-flight Boss-owned projectiles from before pickup are unaffected (still threaten player) — intentional, creates a "hack just started, dodge the old stuff" beat
- `detect_collisions` routes `Player`-owned projectile hitting boss → apply damage; `Player`-owned vs player → no-op
- Visual: boss sprite flickers glitch-green at 12 Hz for duration; full-screen RGB split overlay (desktop only; mobile uses static green tint)
- Tick system removes `HackActive` resource when timer expires

### Missile Swarm

- On pickup, spawn 6 `HomingMissile` entities at player position, initial velocities evenly fanned 360° at 200 px/s
- Each missile:
  ```rust
  #[derive(Component)]
  pub struct HomingMissile {
      pub velocity: Vec2,
      pub lifetime: Timer,       // 8s
      pub trail_timer: Timer,    // 0.05s repeating (0.1s on mobile)
  }
  ```
- Update each frame:
  - Compute desired direction toward current boss position
  - Rotate `velocity` toward desired direction with max rate `4.0 rad/s * dt`
  - Position += velocity * dt
  - On collision with boss: despawn missile, deal 10 dmg (respecting boss `is_invulnerable` and `last_hit_time` cooldown)
  - On `trail_timer` finish: spawn a fading red particle at current position
- Visual: small red triangle sprite
- On boss death: despawn all missiles (handled by round-end cleanup rule)
- On lifetime expire: despawn silently

## Interaction Rules

- **Same buff while active**: refresh timer to full; do not stack duration; reset charge counts (Shield resets to 3)
- **Different buffs**: all can be active simultaneously (Shield + Overclock + Bullet Time + Missile Swarm is valid)
- **Pickup during Laser**: existing laser behavior suppresses normal shooting; Overclock has no visible effect but timer still ticks (intentional — simpler)
- **Round transition** (`OnExit(GameState::RoundActive)`):
  - Remove all timed buff components from player: `PhaseShiftActive`, `OverclockActive`, `ShieldCharges`, `ReflectorActive`
  - Despawn entity-based effects: decoy, homing missiles, gravity attractor, reflector bubble
  - Reset world-state resources: `EnemyTimeScale.scale = 1.0`, remove `HackActive` resource
- **Pause** (`GameState::Paused`): all effect systems are `run_if(in_state(RoundActive))` and naturally pause
- **Boss death while effect active**: timed buffs continue until timer; entity effects (missiles, decoy) despawn when boss dies; damage calcs no-op if boss HP is 0

## Mobile Quality-Tier Degradations

Read existing `QualityTier` resource; fall back to cheaper visuals when `QualityTier::Mobile`:

| Effect | Desktop | Mobile |
|---|---|---|
| Bullet Time overlay | blue tint + chromatic aberration | blue tint only |
| Missile trail spawn rate | every 0.05s | every 0.1s |
| Hack RGB split | full shader | static green tint |
| Glitch Blink particle burst | 16 particles per endpoint | 6 particles per endpoint |
| Gravity Well vortex | 12 rotating particles | 4 particles |
| Reflector / Shield / Decoy / Overclock glow | identical (cheap) | identical |
| Instant buffs (Repair, Energy, Blink teleport) | identical | identical |

## Audio

Existing sound system generates WAV procedurally at startup (see `src/systems/audio.rs`). New sound effects to add to `SoundEffect` enum and `generate_sound`:

| SoundEffect | Usage | Character |
|---|---|---|
| `RepairKitPickup` | Repair Kit pickup | quick ascending chime (three notes, 0.15s) |
| `EnergyCellPickup` | Energy Cell pickup | electric zap (high-freq saw + noise, 0.1s) |
| `PhaseShiftStart` / `PhaseShiftEnd` | Phase Shift in/out | reverse-shimmer / shimmer (0.2s each) |
| `GlitchBlink` | Blink teleport | digital glitch burst (0.15s) |
| `OverclockStart` | Overclock pickup | revving synth (0.3s) |
| `ShieldStart` | Shield pickup | deploying-dome whoosh (0.3s) |
| `ShieldHit` | Shield absorbs a hit | short bell tone (0.1s) |
| `BulletTimeDrone` | Bullet Time duration | low-pitched drone (1s, looped) |
| `DecoyDeploy` | Decoy spawn | duplicated shimmer (0.3s) |
| `GravityWellActive` | Gravity Well duration | low rumble (0.5s, looped) |
| `Reflector` | Reflect bounce | high-pitched ping per bounce (0.08s) |
| `HackStart` | Hack pickup | digital corrupt squeal (0.3s) |
| `MissileLaunch` | Missile Swarm pickup | volley whoosh (0.3s) |

Follow existing pattern: simple sample-generation math, WAV encoded at startup, played via `SoundEvent` / `play_sounds`.

## Testing

### Pure-function unit tests

Following existing codebase style (all tests are pure functions, co-located in `#[cfg(test)]` modules).

- **`roll_random_kind` distribution**: run 10,000 rolls, verify tier percentages within ±2% of 55/35/10
- **Per-item uniformity within tier**: within-tier roll is uniform within ±3% over 10,000 rolls
- **`pick_safe_spot` scoring**: given mock threat positions + boss position, pure function returns the expected-best candidate for seeded RNG
- **Homing missile steering**: given initial velocity and target, after N ticks of 16ms, missile velocity vector converges to target direction within a tolerance
- **Projectile owner flip under Reflector**: calling the bounce helper flips owner Boss→Player and reverses velocity
- **Timer refresh rule**: picking up Overclock while `OverclockActive` exists resets its timer to full (`TimerMode::Once` reset), does not create a second component
- **Shield charge consume**: calling the absorb helper decrements `n`; removes component when `n == 0`

### Manual smoke test checklist (per power-up)

Document in spec; execute before Phase-end ship.

- [ ] Pickup sprite shows correct color + tier size + glow ring
- [ ] Pickup triggers effect + correct audio plays
- [ ] HUD indicator appears for timed buffs; bar shrinks with duration
- [ ] Effect expires cleanly; no orphaned entities (check with existing entity query helpers in debug builds)
- [ ] Re-pickup while active refreshes timer; no stacking of duration/charges
- [ ] Round transition cleans up all effect state
- [ ] Boss death during active effect: missiles/decoy/attractor/bubble despawn; timed buffs tick out normally
- [ ] Mobile build runs without perf regression: test with `QualityTier::Mobile` forced, observe frame time during Bullet Time + Missile Swarm + Gravity Well simultaneously

## Phased Rollout

Three shippable milestones. Each phase ends with the game in a working, tested state.

### Phase 1 — Foundation + Commons

1. Split `src/systems/powerups.rs` into module per "File layout" above
2. Move existing Laser and Shockwave code verbatim into `effects/laser.rs` and `effects/shockwave.rs` (only import paths change)
3. Add `catalog.rs` with `PowerUpKind`, `PowerUpTier`, `PowerUpMeta`, `CATALOG`, `meta`, `roll_random_kind`
4. Add `ProjectileOwner` enum to `BossProjectile`; default `Boss` at all existing spawn sites in `src/core/boss/attacks.rs` and related
5. Rework `spawn.rs` to use catalog + tier-weighted roll + tier-based visual styling (size/pulse/glow)
6. Add `hud.rs` with active-buff indicator row (empty when no buffs active)
7. Add `max_energy: u32` field to `Player` struct (default 100); initialize at all spawn sites
8. Implement 4 commons: Repair Kit, Energy Cell, Phase Shift, Glitch Blink
9. Add sound effects for the 4 commons
10. Mobile-tier visual degradations for Glitch Blink burst
11. Unit tests for: rarity roll distribution, safe-spot scoring, timer refresh rule (Phase Shift)

Ship state: 6 power-ups active (2 old + 4 new commons).

### Phase 2 — Rares

1. Add `EnemyTimeScale` resource (default `scale = 1.0`)
2. Thread `EnemyTimeScale` through affected systems (boss projectile, hazard, dash trail, enemy particles, boss attack timers, boss idle movement)
3. Add `DecoyTarget` marker and update `boss_attack_system` to prefer it when present
4. Implement 5 new rares: Overclock, Shield, Bullet Time, Decoy, Gravity Well
5. Bullet Time visual overlay (desktop + mobile variants)
6. Gravity Well attractor entity + projectile steering
7. Decoy spawning + targeting override + collision absorption
8. Add sound effects for rares
9. Unit tests for: shield charge consume, bullet time `scale` application, decoy targeting preference

Ship state: 11 power-ups active.

### Phase 3 — Ultra-rares

1. Implement Reflector: bubble entity + projectile bounce path + `ReflectorActive` component
2. Implement Hack: `HackActive` resource + `boss_attack_system` owner flip + visual overlay
3. Implement Missile Swarm: `HomingMissile` entities + seeking system + trail particles
4. Mobile-tier degradations for Hack RGB split, Missile trail rate, Reflector particle halo
5. Add sound effects for ultra-rares
6. Unit tests for: projectile owner flip, homing missile convergence

Ship state: all 14 power-ups active.

## Risks & Open Questions

- **Risk: Bullet Time leaks into player systems.** If the `EnemyTimeScale` resource is accidentally used in a shared tick helper, the player could slow too. Mitigation: only explicitly-listed systems multiply by it; test by playing through Bullet Time and verifying player shooting cadence matches normal.
- **Decoy + Hack interaction (resolved in Decoy/Hack sections above)**: Decoy absorbs only `Boss`-owned projectiles; `Player`-owned (reflected / hacked) pass through. During Hack, boss aim bypasses decoy and targets boss center. This keeps Hack reliably hitting the boss even when Decoy is out.
- **Risk: Hitbox on boss during Hack.** If boss is moving fast, Player-owned projectiles might miss the boss body and fly offscreen. Acceptable — hack is already a big buff; not perfectly efficient is fine.
- **Risk: Homing missiles orbit boss.** If turn rate too low vs. boss movement, missiles can orbit without hitting. Mitigation: initial 4 rad/s turn rate plus 8s lifetime; tune if needed during Phase 3.
- **Open question (defer)**: should pickup visuals hint at tier via subtle screen-side flash on spawn (e.g., brief edge glow in pickup color when an ultra-rare spawns)? Not in scope; revisit after Phase 3 ship if pickups feel same-y.

## Success Criteria

1. All 14 power-ups implementable, visually distinct, audibly distinct
2. Tier-first spawn roll produces observable frequency differences (commons feel frequent, ultra-rares feel rare)
3. HUD clearly communicates active buffs and remaining durations
4. No power-up breaks existing combat flow (boss attacks, phase transitions, round transitions all work with all effects active)
5. Mobile build maintains frame rate with all effects active simultaneously
6. Existing Shockwave and Laser behave identically to today's implementation
