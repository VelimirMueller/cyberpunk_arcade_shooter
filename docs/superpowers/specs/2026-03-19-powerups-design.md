# Power-Up System Design

## Overview

Add two power-up pickups that spawn on a random timer: Shockwave (instant arena-clear panic button) and Laser (6-second wide beam mode replacing normal shots).

## Spawning

- `PowerUpTimer` resource with a `Timer` set to a random duration between 15-20 seconds (re-randomized each cycle)
- On fire: spawn a power-up entity at a random position within play bounds (x: ±500, y: ±200)
- Random 50/50 choice between Shockwave and Laser
- Only 1 power-up on screen at a time — skip spawn if one already exists
- Power-up entity has a 10-second lifetime; despawns if not picked up
- Visual: small diamond sprite (rotated 45° square, ~16x16) with gentle sine pulse on alpha
  - Shockwave: cyan color `Color::srgb(0.0, 8.0, 8.0)`
  - Laser: magenta color `Color::srgb(8.0, 0.0, 8.0)`

## Pickup

- AABB collision: player position overlaps power-up entity → consume
- Despawn the pickup entity
- Apply the effect immediately
- Play a pickup sound effect

## Shockwave Power-Up

On pickup (instant, one-shot):
1. Despawn ALL `EnemyParticle`, `BossProjectile`, `DashTrail`, `HazardZone`, `ChargeTelegraph` entities
2. Deal 20 damage to the boss (reduce `boss.current_hp` by 20, clamped to 0)
3. Screen shake via `ScreenShake` resource directly: `screen_shake.intensity = 2.0; screen_shake.duration = 0.5; screen_shake.timer = 0.5;` (do NOT use `trigger_screen_shake()` which hard-codes different values)
4. Spawn expanding white shockwave ring visual at player position — spawn a `ShockwaveRing` entity manually (white circle sprite that expands and fades over 0.3s) rather than depending on `ShockwaveAssets`/`ColorMaterial`. Keep it simple: a `Sprite` with a `ShockwaveRing` component from particles.rs if compatible, or a new `PowerUpShockwave { timer: Timer }` component with its own expand+fade system.
5. Sound: new `SoundEffect::ShockwavePowerUp` — deep boom (50 Hz thump + white noise, 400ms)

### `powerup_pickup_system` parameters

```
fn powerup_pickup_system(
    mut commands: Commands,
    player_query: Query<(&Transform, &Sprite, Entity), With<Player>>,
    powerup_query: Query<(Entity, &Transform, &Sprite, &PowerUp)>,
    mut boss_query: Query<&mut Boss>,
    enemy_particle_query: Query<Entity, With<EnemyParticle>>,
    boss_projectile_query: Query<Entity, With<BossProjectile>>,
    dash_trail_query: Query<Entity, With<DashTrail>>,
    hazard_zone_query: Query<Entity, With<HazardZone>>,
    telegraph_query: Query<Entity, With<ChargeTelegraph>>,
    mut screen_shake: ResMut<ScreenShake>,
    mut sound_events: EventWriter<SoundEvent>,
)
```

## Laser Power-Up

On pickup (timed mode, 6 seconds):
1. Add `LaserActive { timer: Timer::from_seconds(6.0, TimerMode::Once), sound_timer: Timer::from_seconds(0.5, TimerMode::Repeating) }` component to the player entity
2. Spawn a `LaserBeam` entity: tall thin sprite (~10px wide, ~600px tall) that extends upward from the player
   - Color: bright green HDR `Color::srgb(1.0, 8.0, 0.7)`
   - Position: follows player Transform each frame
   - Rotation: follows player Transform rotation
3. While `LaserActive` is present:
   - Normal particle shooting is suppressed — in `player_shoot_system` (combat.rs), add `Option<&LaserActive>` to the player query (import `LaserActive` from `crate::systems::powerups::LaserActive`). If `Some`, early-return.
   - Player energy does not deplete
   - Each frame: AABB collision between `LaserBeam` sprite and boss entity. Use a local `collide()` helper in powerups.rs (copy the 5-line AABB function — `collision.rs::collide` is private). Or make `collide` in collision.rs `pub(crate)`.
   - On overlap: deal 1 damage to boss using `boss.last_laser_hit_time` (a NEW field on Boss, separate from `last_hit_time`) with 75ms cooldown. This avoids conflicting with the player-particle cooldown on `last_hit_time`.
   - Sound: `laser_system` ticks `sound_timer` (0.5s repeating) and fires `SoundEvent(SoundEffect::LaserHum)` each time it triggers. This re-fires the one-shot sound every 500ms for a pseudo-sustained effect. No looping audio needed.
4. When timer expires:
   - Remove `LaserActive` from player
   - Despawn `LaserBeam` entity
   - Revert to normal shooting

## New Components

```
PowerUp { kind: PowerUpKind, lifetime: Timer }
enum PowerUpKind { Shockwave, Laser }
LaserActive { timer: Timer, sound_timer: Timer }
LaserBeam                                       // marker for the beam sprite
PowerUpTimer { timer: Timer }                   // resource for spawn scheduling
PowerUpShockwave { timer: Timer }               // expanding ring visual (if not reusing ShockwaveRing)
```

All power-up pickup entities, LaserBeam, and PowerUpShockwave carry the `GameEntity` marker for cleanup.

## Boss Component Addition

Add to `Boss` struct in `src/core/boss/components.rs`:
```rust
pub last_laser_hit_time: Option<std::time::Instant>,  // separate cooldown for laser beam hits
```

## Round Transition Cleanup

In `src/systems/round.rs` `score_tally_system`: add `PowerUp` and `LaserBeam` entity queries to the cleanup section alongside existing DashTrail/HazardZone/etc. despawns. Also remove `LaserActive` component from player if present.

## New File

`src/systems/powerups.rs` — all power-up logic:
- `setup_powerup_timer()` — inserts `PowerUpTimer` resource
- `powerup_spawn_system()` — ticks timer, spawns pickup entity
- `powerup_lifetime_system()` — ticks pickup lifetime, despawns expired, animates pulse
- `powerup_pickup_system()` — collision detection, applies shockwave or starts laser
- `laser_system()` — updates LaserBeam position/rotation, ticks timers, beam-vs-boss collision, cleanup on expire
- `powerup_shockwave_system()` — expands and fades the shockwave ring visual, despawns when done

## Modified Files

- `src/systems/mod.rs` — add `pub(crate) mod powerups;`
- `src/systems/combat.rs` — in `player_shoot_system`, add `Option<&LaserActive>` to player query, early-return if Some. Import: `use crate::systems::powerups::LaserActive;`
- `src/systems/collision.rs` — make `collide()` function `pub(crate)` instead of private
- `src/systems/audio.rs` — add `ShockwavePowerUp` and `LaserHum` variants + synthesis
- `src/core/boss/components.rs` — add `last_laser_hit_time: Option<std::time::Instant>` to Boss
- `src/core/boss/systems.rs` — initialize `last_laser_hit_time: None` in `spawn_boss()`
- `src/systems/round.rs` — add PowerUp/LaserBeam cleanup in `score_tally_system`
- `src/app.rs` — register power-up systems in RoundActive

## System Registration

- **OnEnter(RoundAnnounce)**: `setup_powerup_timer` (reset timer each round)
- **Update + RoundActive**: `powerup_spawn_system`, `powerup_lifetime_system`, `powerup_pickup_system`, `laser_system`, `powerup_shockwave_system`
