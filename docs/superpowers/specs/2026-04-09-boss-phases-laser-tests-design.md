# Boss Phases, Laser Visuals, and Test Infrastructure

Date: 2026-04-09
Status: Approved

## Overview

Three interconnected changes to the cyberpunk arcade shooter:

1. Expand boss phases from 3 to 4 with cinematic transitions and a dramatic death sequence
2. Replace the flat green laser rectangle with a dual-layer cyberpunk particle storm beam
3. Add ~25 tests (pure logic + ECS integration) and a GitHub Actions CI pipeline

## 1. Boss Phase System

### Phase Structure

Current: 3 phases at 50% and 20% HP thresholds.
New: 4 phases at 60%, 30%, and 10% HP thresholds.

| Phase | HP Range | Name | Behavior |
|-------|----------|------|----------|
| Phase1 | 100-60% | Normal | Base color, standard attack patterns, default timers |
| Phase2 | 60-30% | Enraged | Color shifts brighter, attacks speed up, new patterns added |
| Phase3 | 30-10% | Overdrive | Boss sprite pulses/flickers, dual-layer attacks, arena hazards |
| Phase4 | 10-0% | Desperation | Boss flashes erratically, fastest attacks, ambient screen shake |

### Phase Transitions (~1.5s each)

When HP crosses a threshold:

1. Boss becomes invulnerable (`is_invulnerable = true`)
2. Screen dims (spawn dark semi-transparent overlay)
3. Boss sprite morph pulse: scale 1.2x -> 0.8x -> snap to 1.0x with new base color
4. Phase name text flashes on screen ("ENRAGED!", "OVERDRIVE!", "DESPERATION!")
5. Shockwave ring expands outward from boss position
6. Screen shake (intensity scales with phase: 1.0, 1.5, 2.0)
7. Invulnerability ends, new phase active

Implementation: Add `PhaseTransition` component with a `Timer` and step enum to drive the sequence. The `boss_phase_system` checks HP thresholds and attaches this component. A new `phase_transition_system` drives the animation steps and removes the component when complete.

### BossPhase Enum Change

```rust
pub enum BossPhase {
    Phase1,      // was Phase1 (100-50%)
    Phase2,      // was Phase2 (50-20%)
    Phase3,      // new (30-10%)
    Phase4,      // was Phase3 (20-0%), now Desperation
}
```

Update `phase_thresholds` from `(f32, f32)` to `(f32, f32, f32)` storing (0.60, 0.30, 0.10).

### Desperation Mode (Phase 4, 10-0% HP)

When entering Phase 4:
- Boss sprite flashes between base color and white at 4Hz
- All attack timers reduced to 60% of Phase 3 values
- Ambient screen shake (low intensity 0.3, continuous)
- Boss movement speed increased 30%

### Boss Death Sequence

When HP reaches 0:

1. Boss freezes in place, becomes invulnerable (prevent further damage events)
2. Three staggered explosions at random offsets within 30px of boss center, 0.3s apart each. Each explosion: orange-white burst sprite (scale 0→2.0 over 0.2s, then fade)
3. Brief white screen flash (0.1s overlay at 80% opacity)
4. Boss entity shatters into 30-40 `ShatterParticle` entities flying outward with random velocities and gravity
5. "ELIMINATED" text spawns at boss position, large cyan font, fades over 1.5s
6. Score popup spawns below text showing kill bonus
7. 1.5s pause before triggering `ScoreTallyTimer`

Implementation: Add `BossDeathSequence` component with step enum and timers. Attach when HP hits 0 instead of immediately firing `DeathEvent`. The existing `DeathEvent` fires at step 4 (shatter).

## 2. Laser Beam Visual System

### Lifecycle

Replace the current instant-on/instant-off flat rectangle with a 3-phase beam:

**Charge-Up (0.8s)** — triggered on power-up pickup:
- 8-12 `LaserChargeParticle` entities spawn at random positions around the screen edges and converge toward the player over 0.8s
- At player position: energy orb sprite grows from 6px to 24px diameter, pulsing at 3Hz
- 4 orbiting particles circle the orb, tightening from 30px to 10px radius
- Subtle screen vibration (ScreenShake intensity 0.2, duration 0.8s)
- Player cannot fire normal particles during charge (already suppressed by `LaserActive`)

**Active Beam (5.2s)** — 7 entity types:

1. `LaserBeamCore` — narrow bright beam (6px wide, full beam length). Color: white center fading to `#00ffcc`. HDR emissive for bloom interaction.
2. `LaserBeamShell` — wider translucent beam (32px wide) around the core. Color: `rgba(0, 255, 136, 0.35)` with 1px border at 25% opacity. Pulses width between 30-36px at 1.7Hz.
3. `LaserArc` — 2 electric arc paths rendered between core and shell edges. Regenerated every 3 frames with randomized zigzag control points. Colors: `#00ffcc` (primary arc, 70% opacity) and `#ff44ff` (secondary arc, 40% opacity).
4. `LaserStreamParticle` — small particles (2-4px) spawned at the muzzle and drifting upward along beam edges. 3-5 active at any time. Drift outward 8px from beam center over their 0.8s lifetime. Alternate colors between `#00ff88` and `#ff66ff`.
5. `LaserImpact` — burst effect at the point where beam intersects the boss. 70px diameter radial gradient (white center -> `#00ff88` -> `#ff00ff` -> transparent). Pulses scale 1.0-1.1x at 2.5Hz. Spawns 2-3 spark particles that fly outward from impact point.
6. `LaserMuzzle` — elliptical glow at player origin (40x20px). Radial gradient white to `#00ffcc`. Pulses opacity 0.7-1.0 at 3.3Hz.
7. `LaserChargeOrb` — reused from charge phase, shrinks from 24px to 0 over first 0.3s of active phase.

All beam entities follow player position and rotation each frame (same as current implementation).

**Fade-Out (0.8s)** — final 0.8s of the laser timer:
- Core beam width narrows from 6px to 1px
- Shell beam fades opacity from 0.35 to 0
- Electric arcs stop regenerating, existing ones fade
- Stream particles stop spawning, existing ones drift outward with increased speed
- Muzzle glow shrinks and fades
- Any remaining particles disperse in random directions over 0.5s

### Collision

Collision detection unchanged: uses the existing rotated-AABB approach from `laser_system`. The collision box matches the shell width (32px x beam_length), not the visual particles.

### Damage

Same as current: 1 HP per hit with 75ms cooldown per boss via `last_laser_hit_time`.

### Audio

- Charge-up: ascending frequency sweep sound (new `LaserCharge` variant)
- Fire flash: short impact burst sound (new `LaserFire` variant)
- Active: existing `LaserHum` continues as-is
- Fade-out: descending frequency sweep (new `LaserFadeOut` variant)

### Component Changes

Replace current components:
```rust
// Remove
pub struct LaserBeam;  // single marker

// Add
pub struct LaserCharge {
    pub timer: Timer,           // 0.8s charge duration
    pub orb_scale: f32,         // grows 0.0 -> 1.0
}

pub struct LaserBeamCore;       // marker for inner beam
pub struct LaserBeamShell {
    pub pulse_timer: f32,       // for width oscillation
}

pub struct LaserArc {
    pub regen_counter: u8,      // regenerate every 3 frames
}

pub struct LaserStreamParticle {
    pub lifetime: Timer,
    pub drift_offset: f32,      // lateral drift from beam center
}

pub struct LaserImpact;         // marker, positioned at boss intersection

pub struct LaserMuzzle;         // marker, positioned at player origin

pub struct LaserChargeParticle {
    pub target: Vec2,           // converges toward player
    pub speed: f32,
}
```

Update `LaserActive` to include phase tracking:
```rust
pub struct LaserActive {
    pub timer: Timer,           // total 6.8s (0.8 charge + 5.2 active + 0.8 fade)
    pub phase: LaserPhase,      // Charging, Active, FadingOut
    pub sound_timer: Timer,
}

pub enum LaserPhase {
    Charging,
    Active,
    FadingOut,
}
```

## 3. Testing

### Tier 1: Pure Logic Unit Tests (~12 tests)

Location: `#[cfg(test)]` modules within each source file.

**collision.rs** (4 tests):
- `test_collide_overlapping` — two overlapping AABBs return true
- `test_collide_separated` — two distant AABBs return false
- `test_collide_touching_edges` — edge-touching AABBs return false (strict inequality)
- `test_collide_one_contains_other` — fully contained AABB returns true

**boss/components.rs** (4 tests):
- `test_phase_thresholds_default` — verify (0.60, 0.30, 0.10) thresholds
- `test_boss_spawn_hp_per_type` — each boss type gets correct max_hp
- `test_phase_from_hp_percentage` — HP percentages map to correct phases
- `test_score_multiplier_per_round` — rounds 1-5 produce correct multipliers (1.0, 1.5, 2.0, 2.5, 3.0)

**powerups.rs** (2 tests):
- `test_laser_total_duration` — charge (0.8) + active (5.2) + fade (0.8) = 6.8s
- `test_laser_phase_from_elapsed` — elapsed time maps to correct LaserPhase

**round.rs** (2 tests):
- `test_round_boss_type_mapping` — rounds 1-5 map to correct BossType
- `test_round_progression_wraps` — round 5 completion triggers Won state

### Tier 2: ECS Integration Tests (~13 tests)

Location: `tests/` directory (integration tests) or `#[cfg(test)]` with Bevy `App`.

Each test creates a minimal `App`, adds required plugins/systems, and calls `app.update()`.

**Boss damage and phases** (5 tests):
- `test_boss_takes_damage` — spawn boss at 100 HP, simulate particle collision, verify HP decreases
- `test_boss_phase_transition_at_60_percent` — reduce HP to 60% threshold, verify phase changes to Phase2
- `test_boss_phase_transition_at_30_percent` — verify Phase3 at 30%
- `test_boss_enters_desperation_at_10_percent` — verify Phase4 at 10%
- `test_boss_death_event_at_zero_hp` — verify DeathEvent emitted when HP reaches 0

**Boss transition behavior** (2 tests):
- `test_boss_invulnerable_during_transition` — during PhaseTransition, boss.is_invulnerable == true
- `test_boss_attack_state_resets_on_transition` — attack state returns to Idle after transition

**Laser lifecycle** (3 tests):
- `test_laser_activation_adds_components` — picking up laser power-up adds LaserActive with Charging phase
- `test_laser_transitions_to_active` — after 0.8s, phase becomes Active and beam entities exist
- `test_laser_cleanup_on_expiry` — after full duration, all laser entities despawned, LaserActive removed

**Power-ups and rounds** (3 tests):
- `test_shockwave_damages_boss` — shockwave pickup deals 20 damage
- `test_shockwave_clears_projectiles` — all enemy projectiles despawned after shockwave
- `test_round_advances_after_boss_death` — boss death increments round counter

### Test Utilities

Create a `tests/helpers.rs` (or test module helper) with:
- `spawn_test_boss(app, boss_type, hp)` — spawns a boss entity with minimal components
- `spawn_test_player(app, position)` — spawns a player entity at given position
- `tick_app(app, duration)` — advances app time by duration and calls update

## 4. CI Pipeline

### GitHub Actions Workflow

File: `.github/workflows/ci.yml`

```yaml
name: CI
on:
  push:
    branches: [main, add-boss-stages]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
```

### CI Requirements

- All tests must pass
- No clippy warnings (deny all)
- Code formatted with rustfmt
- Cargo cache for fast subsequent runs

## Scope Boundaries

**In scope:**
- 4-phase boss system with cinematic transitions
- Boss death sequence (explosions, shatter, text)
- Desperation mode at 10% HP
- Full laser visual overhaul (charge, dual-layer beam, particles, fade)
- ~25 unit and integration tests
- GitHub Actions CI pipeline

**Out of scope:**
- New boss types or attack patterns (existing 5 bosses keep their attacks)
- Changes to player mechanics, movement, or shooting
- Changes to shockwave power-up
- UI/HUD redesign
- Audio overhaul (only 3 new sound variants for laser phases)
- Multiplayer or save system
