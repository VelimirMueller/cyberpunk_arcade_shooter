# Boss Stages & UI Overhaul Design

## Overview

Transform Cyberpunk Bloom Cube from a single-encounter arcade shooter into a 5-round souls-like boss gauntlet with pattern-based phase mechanics and AAA-quality cinematic HUD.

## Architecture: Monolithic Boss Enum

A single `BossType` enum with all 5 bosses. Each variant carries its own config. One `boss_ai_system` dispatches behavior via pattern matching. Phase state lives on the boss entity as a component.

---

## 1. Game State Changes

### Current States
`Menu → Playing → Paused → GameOver → Won`

### New States
`Menu → RoundAnnounce → RoundActive → Paused → RoundAnnounce (next) → ... → Won`

- **RoundAnnounce**: 2.5-second cinematic boss introduction. No player input except pause.
- **RoundActive**: Replaces `Playing`. The actual boss fight.
- **Paused**: Accessible from `RoundActive`. Pause resumes to `RoundActive` (not the old `Playing` state).
- **GameOver**: Unchanged, player HP reaches 0.
- **Won**: After Round 5 boss defeated.

`GameData.wave` renamed to `GameData.round` (1-5). Add `GameData.total_rounds: u32 = 5`.

### Round Flow

1. Game starts → `RoundAnnounce` for Round 1
2. Announcement ends → `RoundActive`, boss spawns
3. Boss defeated → 1-second score tally pause → `RoundAnnounce` for Round N+1
4. After Round 5 boss dies → `Won` state
5. Player dies at any point → `GameOver`
6. Restart from `GameOver` resets to Round 1 → transitions to `RoundAnnounce` (not `Playing`)

---

## 2. Boss Data Model

### Components

```
Boss {
    boss_type: BossType,
    phase: BossPhase,          // Phase1, Phase2, Phase3
    current_hp: u32,           // Current health points
    max_hp: u32,               // Scales per round
    phase_thresholds: (f32, f32),  // (0.50, 0.20) — HP percentage triggers
    transition_style: TransitionStyle,  // Stagger or RageBurst

    // Attack-specific timers (varies per boss type)
    primary_timer: Timer,
    secondary_timer: Option<Timer>,

    // State for current attack pattern
    attack_state: AttackState,
}

enum BossType { GridPhantom, NeonSentinel, ChromeBerserker, VoidWeaver, ApexProtocol }
enum BossPhase { Phase1, Phase2, Phase3 }
enum TransitionStyle { Stagger, RageBurst }
enum AttackState { Idle, WindUp(Timer), Attacking, Recovery(Timer) }

// Attack/hazard entity components:
DashTrail    { position, direction, lifetime: Timer, damage: u32 }
BeamSweep    { origin, angle, arc_width, rotation_speed, damage: u32 }
HazardZone   { position, radius, lifetime: Timer, drift_velocity: Option<Vec2>, explodes: bool, explosion_timer: Option<Timer> }
ChargeTelegraph { start: Vec2, end: Vec2, lifetime: Timer }  // Visual preview line
BossProjectile { velocity: Vec2, damage: u32 }  // Generic boss projectile
```

### Player Damage Mechanism

Player damages the boss via `PlayerParticle` collisions against the `Boss` entity (same AABB detection as current `Enemy` collision). Each hit reduces `boss.current_hp` by 1. Phase transitions are checked every frame: if `current_hp / max_hp <= threshold`, advance phase.

### Between-Round Restoration

After each boss is defeated, the player's HP is restored to 50% of max (or current HP, whichever is higher). Energy fully restores to 100. This rewards clean play without making later rounds impossible.

### Score Tally Pause

When a boss dies, `RoundActive` remains active for 1 second with a "ROUND CLEAR" overlay (centered text, boss kill score bonus shown). Player is invulnerable and cannot shoot during this window. After 1 second, state transitions to `RoundAnnounce` for the next round (or `Won` after Round 5).

All boss entities, hazard zones, trails, and projectiles must carry the `GameEntity` marker component for cleanup on restart/round transitions.

### Boss Configurations

| Round | Boss | Max HP | Transition | Movement |
|-------|------|--------|------------|----------|
| 1 | GRID PHANTOM | 150 | Stagger | Dash-based (straight lines) |
| 2 | NEON SENTINEL | 200 | Stagger | Stationary with rotation |
| 3 | CHROME BERSERKER | 250 | RageBurst | Charge at player |
| 4 | VOID WEAVER | 300 | Stagger | Teleport between hazard zones |
| 5 | APEX PROTOCOL | 400 | RageBurst | Composite (all patterns) |

### Phase Thresholds

All bosses: Phase 2 at 50% HP, Phase 3 at 20% HP. Front-loaded design — Phase 1 is longest, Phase 3 is a desperate sprint.

---

## 3. Boss Roster — Attack Patterns

### Round 1: GRID PHANTOM (Tutorial Boss)

Teaches: reading telegraphs and dodging linear attacks.

**Phase 1 (100%-50% HP):**
- Straight-line dashes with 1-second telegraph (glowing line preview along dash path)
- 3-second cooldown between dashes
- Fires slow homing particles between dashes

**Phase 2 (50%-20% HP) — Stagger transition:**
- Dashes leave lingering trail hazard (damage zone, 2-second lifetime)
- Cooldown drops to 2 seconds

**Phase 3 (20%-0% HP) — Stagger transition:**
- Trail zones persist 4 seconds
- Dashes chain 2x before pausing

### Round 2: NEON SENTINEL (Turret Boss)

Teaches: positioning and spatial awareness.

**Phase 1 (100%-50% HP):**
- Stationary, rotates slowly
- Fires beam sweeps (thin line projectile) in 90-degree arcs
- 4-second rotation cycle

**Phase 2 (50%-20% HP) — Stagger transition:**
- Fires from 2 angles simultaneously
- Rotation speed increases

**Phase 3 (20%-0% HP) — Stagger transition:**
- Beam splits into 3 spread shots
- Rotation direction randomizes

### Round 3: CHROME BERSERKER (Melee Boss)

Teaches: timing and spacing.

**Phase 1 (100%-50% HP):**
- Charges at player position with 0.8-second wind-up (screen shake + glow)
- 2-second recovery after charge

**Phase 2 (50%-20% HP) — RageBurst transition:**
- Charges come in combos of 2-3
- Recovery drops to 1 second

**Phase 3 (20%-0% HP) — RageBurst transition:**
- Charges emit shockwave on landing (expanding ring, damages on contact)

### Round 4: VOID WEAVER (Area Denial Boss)

Teaches: arena management.

**Phase 1 (100%-50% HP):**
- Spawns hazard zones (glowing circles) at random positions
- Max 3 zones, zones last 5 seconds
- Boss teleports between zones

**Phase 2 (50%-20% HP) — Stagger transition:**
- Zones drift slowly toward player
- Max 4 zones

**Phase 3 (20%-0% HP) — Stagger transition:**
- Zones explode after 3 seconds, dealing area damage
- Boss teleports faster

### Round 5: APEX PROTOCOL (Final Exam)

Tests: all skills learned from previous bosses.

**Phase 1 (100%-50% HP):**
- Alternates between dash (Phantom) and beam sweep (Sentinel) on a cycle
- Slower versions of each original pattern

**Phase 2 (50%-20% HP) — RageBurst transition:**
- Adds charge attacks (Berserker) to the cycle
- Cycle speeds up

**Phase 3 (20%-0% HP) — RageBurst transition:**
- Adds hazard zones (Weaver)
- All patterns can overlap simultaneously

---

## 4. Phase Transition Effects

### Stagger (Phantom, Sentinel, Weaver)
1. Boss freezes in place for 1.5 seconds
2. Screen flash — brief white overlay fading out
3. Glitch effect — CRT distortion intensifies momentarily
4. `PHASE SHIFT` text appears center-screen, fades over 1 second
5. Boss color/glow intensifies to indicate new phase
6. Boss resumes with new patterns

### RageBurst (Berserker, Apex)
1. Boss emits expanding shockwave ring (damages player on contact)
2. Hard screen shake (0.5 seconds)
3. Boss color shifts immediately — more intense, faster glow pulse
4. No pause — new phase patterns begin instantly
5. `PHASE SHIFT` text still appears but gameplay does not stop

---

## 5. Round Announcement System

State: `RoundAnnounce` — lasts 2.5 seconds total.

### Sequence (timed reveal)

| Time | Element | Style |
|------|---------|-------|
| 0.0s | `// INCOMING THREAT //` | Small, magenta, letter-spaced, typewriter reveal |
| 0.4s | `ROUND N` | Large, cyan, neon glow, fade-in |
| 0.8s | Boss name | Medium, magenta glow, fade-in |
| 1.2s | Flavor text | Tiny, gray, fade-in |
| 1.5s | Hold | All elements visible |
| 2.2s | Fade out | All elements dissolve |
| 2.5s | Transition to `RoundActive` | Boss spawns |

### Visual Treatment
- Dark overlay dims the arena
- CRT scanline effect intensifies during announcement
- Text uses HDR colors that interact with existing bloom
- All implemented as Bevy UI nodes with timer-driven alpha animation
- No external assets — purely text and color

---

## 6. HUD Overhaul — Cinematic Frame Layout

Replaces all current plain-text UI with structured Bevy UI nodes.

### Layout Regions

**Top Center — Boss Info:**
- Boss name: small text, red (#ff003c), letter-spaced, glow
- Boss HP bar: full-width container with fill child
  - Depletes right-to-left
  - Phase threshold markers at 50% and 20% (vertical lines on the bar)
  - Damage flash sliver — when boss takes damage, a lighter section briefly appears at the damage edge
- Phase pips below bar: small squares, hollow = cleared, filled = remaining

**Bottom Left — Player Stats:**
- "OPERATOR" label: tiny, cyan, letter-spaced
- HP bar: icon square + bar container (green gradient #00cc66→#00ff88) + numeric value
- Energy bar: icon square + thinner bar (purple gradient #6600cc→#aa44ff) + numeric value

**Bottom Center — Round Progress:**
- 5 small pips in a row — filled = remaining rounds, hollow = completed
- `ROUND N / 5` label below

**Bottom Right — Score:**
- "SCORE" label: tiny, gray
- Score value: medium text, cyan (#00ffcc), neon glow
- Pop animation on score increase

### Color System

| Element | Color | Hex |
|---------|-------|-----|
| Player HP | Green | #00ff88 |
| Player Energy | Purple | #aa44ff |
| Boss HP | Red | #ff003c |
| Score / Round | Cyan | #00ffcc |
| Labels | Gray | #555555 |
| Backgrounds | Near-black | #0a0a0a |
| Borders | Dark tone | #222222 |

### Glow Implementation
Bevy 0.16 bloom applies to the 2D/3D render pass, NOT to the UI overlay. Standard Bevy UI nodes will not bloom. Two approaches for neon glow on HUD elements:

1. **Layered text shadows** (recommended): Render duplicate text nodes behind the primary text with lower opacity and slight offset/scale to simulate glow. Use the existing colored text with high-saturation colors.
2. **World-space sprites**: Render HUD elements as world-space sprites that DO receive bloom, positioned via camera-relative coordinates. More complex but true bloom.

Approach 1 is simpler and sufficient for the cyberpunk aesthetic. The CRT post-processing shader already adds scanlines and vignette over the entire frame (including UI), which helps sell the effect.

### Menu Screens Overhaul

**Title Screen:**
- Game title with neon glow and subtle pulse animation
- Bordered container with padding
- High score display with proper styling
- `PRESS ENTER TO START` with subtle flash animation

**Pause Screen:**
- Semi-transparent dark overlay
- Centered container with border
- Menu options in a structured list with highlight states
- Sound status indicator

**Game Over / Won:**
- Large title text with glow
- Final score display
- Round reached indicator
- `PRESS SPACE` instruction

---

## 7. Enemy Spawn Changes

### Current
- 3 enemies spawned simultaneously in `create_enemies()`
- All share the same figure-8 patrol and rotation logic
- Only first has `fire_timer`

### New
- `create_enemies()` replaced by `spawn_boss(round: u32)`
- Spawns exactly 1 boss entity per round based on `BossType`
- Boss size, color, and glow vary per type
- Boss-specific movement replaces the generic waypoint patrol
- `total_enemies` becomes 1 per round, `enemies_killed` resets each round

### Boss Visual Identity

| Boss | Base Color | Glow | Size Multiplier |
|------|-----------|------|-----------------|
| GRID PHANTOM | Cyan (#00ffff) | Cyan pulse | 1.0x |
| NEON SENTINEL | Magenta (#ff00ff) | Magenta steady | 1.2x |
| CHROME BERSERKER | Orange (#ff8800) | Orange rapid pulse | 1.4x |
| VOID WEAVER | Purple (#8800ff) | Purple shimmer | 1.1x |
| APEX PROTOCOL | White (#ffffff) | Multi-color cycle | 1.6x |

---

## 8. Difficulty Scaling

Beyond boss-specific mechanics, global difficulty scales across rounds:

- Boss projectile speed: +10% per round
- Boss HP: 150 → 200 → 250 → 300 → 400
- Score multiplier: 1x → 1.5x → 2x → 2.5x → 3x
- Player energy regen stays constant (no nerf — reward aggression)

---

## 9. Audio Additions

New synthesized sounds (extending existing Kira audio system):

- **BossSpawn**: Low rumble + rising tone (announcement sequence)
- **PhaseShift**: Glitch/distortion burst
- **RageBurst**: Impact + bass drop
- **DashTelegraph**: Rising whine during wind-up
- **BeamSweep**: Sustained mid-frequency tone
- **ChargeWindUp**: Accelerating rumble
- **HazardSpawn**: Bubble/pop effect
- **HazardExplode**: Sharp crack
- **RoundClear**: Triumphant chord

---

## 10. Files to Modify/Create

### New Files
- `src/core/boss/mod.rs` — Boss module
- `src/core/boss/components.rs` — Boss, BossType, BossPhase, AttackState components
- `src/core/boss/systems.rs` — Boss AI, phase transitions, spawning
- `src/core/boss/attacks.rs` — Per-boss attack pattern implementations
- `src/systems/round.rs` — Round announcement system, round flow management
- `src/ui/hud.rs` — New cinematic HUD (bars, pips, layout)
- `src/ui/menus.rs` — Overhauled menu screens (title, pause, game over, won)
- `src/ui/announcement.rs` — Round announcement UI entities

### Modified Files
- `src/data/game_state.rs` — Add RoundAnnounce, RoundActive states; remove Playing
- `src/app.rs` — Remove old UI setup, old enemy spawn, integrate new systems; rename GameData.wave to GameData.round
- `src/systems/combat.rs` — Replace `boss_shoot_system` (queries Enemy) with boss-aware combat; adapt player shooting to target Boss entities
- `src/systems/collision.rs` — Adapt to single Boss entity per round (replace Enemy queries with Boss queries)
- `src/systems/game_over.rs` — Round progression logic (next round vs Won); restart transitions to RoundAnnounce; pause resumes to RoundActive
- `src/core/mod.rs` — Register new boss module, remove enemies module
- `src/core/enemies/` — Removed, replaced by boss module
- `src/systems/audio.rs` — Add new sound effect synthesizers
- `src/systems/particles.rs` — Add boss-specific particle effects (dash trails, hazard zones, charge shockwaves)

### Removed
- `src/core/enemies/` — Replaced by `src/core/boss/`
