# Enterprise Juice Pass — Design Spec

**Date:** 2026-03-18
**Goal:** Transform Cyberpunk Bloom Cube from a functional prototype into a premium-feeling arcade game through layered "juice" — visual effects, post-processing, and synthesized audio.
**Approach:** Layer Cake — each layer is independently playable and testable.

---

## Layer 1: Background System (Grid + Stars)

Replaces plain black background with a faint green grid and slowly drifting star particles.

### Components

- **`BackgroundGrid`** — Large quad with grid lines at z = -100 (behind everything).
  - Grid color: very faint green (`rgba(0, 1.0, 0.25, 0.04)`)
  - Cell size: ~40px
  - Rendered using Bevy `Gizmos` (line drawing API) — no texture assets, no custom shader. Draw vertical + horizontal lines each frame in a system. Zero-asset approach consistent with the rest of the project.

- **`BackgroundStar`** — 30-50 small sprite entities scattered randomly across the arena.
  - Colors: mix of white, faint green, faint magenta (matching game palette)
  - Sizes: 1-3px
  - Each has a slow random drift velocity (5-15 units/sec)
  - Stars wrap at arena bounds: X: -600..600 (`LEFT_BOUND`/`RIGHT_BOUND`), Y: -247..247 (`GROUND_Y`/`CEILING_Y`)
  - Rendered at z = -90 (in front of grid, behind gameplay)

### Cleanup Strategy

Background entities are **persistent** — they do NOT despawn on restart. The grid and stars are game-state-independent and should survive across Menu/Playing/GameOver/Won transitions. Do NOT tag them with `GameEntity`. They are spawned once in a startup system and never cleaned up.

### Systems

- `spawn_background()` — Startup system. Spawns star entities (grid is drawn via gizmos, not entity-based).
- `draw_background_grid()` — Update system. Uses `Gizmos` to draw grid lines each frame. Runs in all game states.
- `animate_stars()` — Update system. Drifts stars, wraps positions at arena boundaries. Runs in all game states.

### Dependencies

None. Uses existing Bevy sprite + Gizmos APIs.

---

## Layer 2: Particle Overhaul

Replaces silent bullet disappearance with dramatic death explosions, ring shockwaves, and player movement trails.

### Enemy Death: Shatter + Shockwave

**Event-driven death system:** When the collision system detects enemy HP reaching 0, it sends a `DeathEvent`:
```rust
struct DeathEvent {
    position: Vec3,  // enemy's transform position
    color: Color,    // from enemy's Sprite.color
    entity: Entity,  // the enemy entity to despawn
}
```
The collision system sets `is_dead = true` on the `Enemy` component (new field, default `false`) to prevent duplicate triggers, and sends the event. A separate `handle_death_events` system reads `EventReader<DeathEvent>`, spawns the shatter particles + shockwave ring, and despawns the enemy entity via `commands.entity(event.entity).despawn()`. The collision system must skip enemies where `is_dead == true`.

**`ShatterParticle`** component:
- Small cubes, 4-8px (random)
- 12-20 particles per enemy death
- Random velocity: 200-400 units/sec, all directions
- Inherited enemy color from `Sprite.color` (high HDR values for bloom)
- Random rotation speed
- Lifetime: 0.5s with alpha fade-out
- Downward gravity acceleration: 150 units/sec^2

**`ShockwaveRing`** component:
- Spawns at enemy death position
- Starts radius 0, expands to ~150px over 0.3s
- Rendered as a `Circle` mesh with `MeshMaterial2d` — Bevy 0.16 has `Circle` as a `Meshable` primitive. Use a `ColorMaterial` with the enemy's color. Scale the mesh transform to animate expansion.
- Alpha fades to 0 as it expands (update material alpha each frame)
- Single entity per death

**Systems:**
- `handle_death_events()` — Bevy system reading `EventReader<DeathEvent>`. For each event: spawns shatter particles + shockwave ring at the event position/color, despawns the enemy entity.
- `animate_shatter()` — Moves particles (velocity + gravity), applies rotation, fades alpha, despawns on lifetime expiry.
- `animate_shockwave()` — Scales ring transform up, fades material alpha, despawns when done. On despawn, also remove the `Handle<ColorMaterial>` and `Handle<Mesh>` from asset stores to prevent GPU resource leaks (or use a shared mesh/material resource — see below).

**Shockwave resource optimization:** To avoid per-death mesh/material allocation and leaks, create a `ShockwaveAssets` resource at startup holding a single shared `Handle<Mesh>` (unit circle) and a template `Handle<ColorMaterial>`. Each shockwave entity clones the material handle and tints it. On despawn, the cloned material handle is dropped, and Bevy's ref-counted asset system frees it automatically. The shared mesh is never freed.

**All new particle types (`ShatterParticle`, `ShockwaveRing`, `Afterimage`, `AmbientParticle`) must also be tagged with `GameEntity`** so they are cleaned up by the existing `restart_listener` system on restart/game-over. Their per-type `animate_*` systems handle normal lifetime despawn; the `GameEntity` tag handles abnormal cleanup (state transition mid-animation).

### Player Trail: Afterimage + Ambient Particles

**`Afterimage`** component:
- Ghost copy of player sprite
- Spawned every 0.05s while player is moving
- Same color as player at 50% alpha, fading to 0 over 0.15s
- Natural cap: ~5-6 visible at once (spawn rate vs fade time)
- Tagged with `GameEntity`

**`AmbientParticle`** component:
- Tiny 1-2px particles floating off player cube at all times (even idle)
- 2-3 spawned per second
- Random drift direction, speed: 20-40 units/sec
- Lifetime: 0.8s
- Very faint green glow
- Tagged with `GameEntity`

**Systems:**
- `spawn_afterimages()` — Update system. Checks if player moved, spawns ghost if enough time elapsed.
- `animate_afterimages()` — Fades and despawns.
- `spawn_ambient_particles()` — Timer-based spawning around player position.
- `animate_ambient_particles()` — Drift + fade + despawn.

### System Ordering

All particle systems run in `Update` schedule during `Playing` state:
1. `detect_collisions` (existing) — detects kills, sets `is_dead`, sends `DeathEvent`
2. `handle_death_events` — reads `DeathEvent`, spawns effects, despawns enemy. Runs after `detect_collisions` (use `.after()`)
3. `animate_shatter`, `animate_shockwave`, `animate_afterimages`, `animate_ambient_particles` — no ordering constraints between them, can run in parallel
4. `spawn_afterimages`, `spawn_ambient_particles` — no ordering constraints, can run any time in Update

**Pause behavior:** Particle timers use Bevy `Time`, which does not advance while systems are paused — no special handling needed.

**Performance:** Peak particle count ~80-100 entities during boss death + player moving. Trivial for Bevy.

---

## Layer 3: CRT Post-Processing Shader

Full-screen post-processing pass applying scanlines, vignette, barrel distortion, and phosphor glow.

### Component (on Camera)

**`CrtSettings`** — Attached as a `Component` on the camera entity (not a global Resource). This follows Bevy 0.16's per-camera post-processing pattern where settings are queried via `ViewQuery` in the `ViewNode`. Runtime-tweakable parameters:
- `scanline_intensity`: 0.0-1.0 (default 0.15)
- `scanline_count`: number of scanline cycles across screen height (default 200.0) — used in shader as `sin(uv.y * scanline_count * PI * 2.0)`, producing 200 dark bands across the full screen height
- `vignette_intensity`: 0.0-1.0 (default 0.4)
- `vignette_radius`: extent from center (default 0.7)
- `curvature_amount`: barrel distortion (default 0.02 — very subtle)

### Shader (WGSL) — `src/shaders/crt_post_process.wgsl`

- **Input:** Rendered frame as texture (bound as `texture_2d<f32>`)
- **Barrel distortion:** Apply UV warp: `uv = uv + (uv - 0.5) * dot(uv - 0.5, uv - 0.5) * curvature_amount` before sampling
- **Scanlines:** After sampling, multiply by `1.0 - scanline_intensity * (0.5 + 0.5 * sin(uv.y * scanline_count * PI * 2.0))` — darkens alternating horizontal bands
- **Vignette:** Multiply by `smoothstep(0.0, vignette_radius, 1.0 - length((uv - 0.5) * 2.0))` raised to `vignette_intensity`
- **Phosphor bleed:** Use `textureDimensions(screen_texture)` in WGSL to get screen resolution (no need to pass as uniform). Sample at `uv + vec2(1.0/f32(dims.x), 0.0)` and `uv - vec2(1.0/f32(dims.x), 0.0)`, add 5% of each neighbor's brightness to the center pixel. This creates a subtle horizontal glow on bright pixels without duplicating Bevy's bloom.
- **Output:** Modified color

### Implementation — `src/systems/post_processing.rs`

Bevy 0.16 post-processing pipeline setup:
1. Define a `CrtPostProcessNode` implementing `ViewNode` trait
2. Create a `CrtPostProcessPipeline` resource holding the cached render pipeline (vertex + fragment shader, bind group layout for the input texture + sampler + uniforms)
3. Register the node in the `Core2d` render graph, inserted after Bevy's tonemapping node but before the upscaling/output node
4. Uniform buffer: upload `CrtSettings` fields each frame via `PrepareViewUniforms`
5. The node samples the rendered frame texture, applies the shader, and writes to the output

**Reference:** Follow Bevy's official `post_processing` example (exists in the Bevy repo under `examples/shader/post_processing.rs`). Consult Bevy 0.16 source if the example has API drift.

Plugin registered in `app.rs`.

---

## Layer 4: Synthesized Audio

Procedurally generated sound effects — no audio files. Everything synthesized at runtime.

### Dependency

**`kira = "0.9"`** (or latest compatible). Fallback: `rodio` if kira has Bevy 0.16 compatibility issues.

### Thread Safety & Event-Based Architecture

To avoid forcing gameplay systems (collision, combat) onto the main thread via `NonSend`, use an **event-based sound architecture**:

1. Gameplay systems send `SoundEvent(SoundEffect)` events (lightweight, no thread constraints)
2. A dedicated `play_sounds` system reads `EventReader<SoundEvent>` and plays via `NonSend<SynthAudio>`
3. Only the `play_sounds` system touches the `NonSend` resource — only it is main-thread-constrained

This decouples sound playback from game logic entirely.

`kira::AudioManager` may be `!Send` depending on backend. Store as `NonSend<SynthAudio>`. If kira's manager happens to be `Send + Sync`, use a regular `Resource` instead — but design for `NonSend` as the safe default.

### Migration from existing AudioManager

The existing `AudioManager` resource in `src/systems/audio.rs` is replaced by `SynthAudio`. Files that need updating:
- `src/systems/audio.rs` — Rewrite entirely (new SynthAudio resource, SoundEvent event, play_sounds system)
- `src/systems/collision.rs` — Replace `play_sound()` calls with `event_writer.write(SoundEvent(SoundEffect::Foo))`
- `src/systems/combat.rs` — Same event-based pattern
- `src/app.rs` — Replace `AudioManager` resource init with `SynthAudio`, register `SoundEvent` event, add `play_sounds` system

The `SoundEffect` enum is kept as-is. Only the backing implementation changes.

### Sound Definitions

| Sound | Synthesis | Duration |
|-------|-----------|----------|
| PlayerShoot | Sine burst 800Hz→400Hz pitch drop, fast attack/decay | 0.08s |
| EnemyShoot | Lower sine 400Hz→200Hz, slightly longer | 0.12s |
| PlayerHit | White noise burst + low sine thump 150Hz | 0.15s |
| EnemyHit | Short noise click + mid sine 500Hz | 0.1s |
| Explosion | Layered: white noise burst + low sine sweep 200Hz→50Hz + distortion | 0.4s |
| GameOver | Descending sine sweep 600Hz→100Hz | 0.8s |
| GameWon | Ascending arpeggio: 3 sine tones (C-E-G) | 0.5s |
| MenuSelect | Quick sine blip 1000Hz | 0.05s |

### Systems

- `setup_synth_audio()` — Startup system. Initializes kira AudioManager, inserts `NonSend<SynthAudio>`.
- `play_synth_sound(sound: SoundEffect)` — Generates waveform and plays. Respects `sound_enabled` and `volume`. Runs on main thread (NonSend constraint).
- Hooks into existing trigger points: collision system, combat system, game state transitions.

---

## Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Effect style | CRT Retro (scanlines + vignette + curvature) | Matches arcade cyberpunk identity |
| Death effect | Ring shockwave + geometry shatter | Maximum drama for boss kills |
| Player trail | Afterimage ghosts + ambient particles | Premium feel, constant visual presence |
| Background | Grid (Gizmos) + drifting star particles | Depth without distraction, zero assets |
| Audio approach | Procedural synthesis (kira, NonSend) | Zero asset management, fully cyberpunk |
| Build order | Layer Cake (background → particles → shader → audio) | Each layer independently playable |

## Out of Scope (Future Passes)

- UI polish (animated health bars, styled menus, transitions, floating score popups)
- Gameplay depth (wave system, power-ups, progression, combo scoring)
- Leaderboard
- Additional enemy types
