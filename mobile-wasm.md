# Mobile WASM Build Guide

How to build, debug, and optimize the mobile-optimized WASM target for Cyberpunk: The Incredible Bloom Cube.

## Prerequisites

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

## Build Commands

```bash
# Development (fast iteration, no optimizations)
trunk serve

# Release (optimized, output in dist/)
trunk build --release
```

Trunk reads `index.html` for the entry point, `Trunk.toml` for wasm-opt config, and `.cargo/config.toml` for WASM-specific rustflags.

## Architecture: How Mobile Detection Works

The build uses a two-layer platform detection strategy:

**Compile-time** (`#[cfg(target_arch = "wasm32")]`): Used for constants that feed into non-system functions where passing `Res<QualityTier>` would be impractical. Defined in `src/utils/config.rs`:

| Constant | Desktop | Mobile (WASM) |
|----------|---------|---------------|
| `ENTITY_SCALE` | 1.0 | 0.85 |
| `STAR_COUNT` | 40 | 25 |
| `DEATH_PARTICLE_MIN/MAX` | 12-20 | 6-10 |
| `AFTERIMAGE_INTERVAL` | 0.05s | 0.10s |
| `AMBIENT_PARTICLE_INTERVAL` | 0.4s | 0.8s |
| `PLAYER_SHOT_COOLDOWN` | 0.15s | 0.25s |
| `BEAM_SEGMENTS` | 12 | 6 |
| `LASER_CHARGE_PARTICLES` | 8 | 4 |

**Runtime** (`Res<QualityTier>`): Used in Bevy systems for camera, bloom, and CRT setup. Defaults to `Mobile` on `wasm32`, `Desktop` otherwise.

**Shader** (`#ifndef MOBILE`): The CRT post-process shader (`assets/shaders/crt_post_process.wgsl`) conditionally compiles barrel distortion and phosphor bleed. Mobile gets scanlines + vignette only. The `MOBILE` define is injected via `CrtPipelineKey::is_mobile` in `src/systems/post_processing.rs`.

## Errors You Will Hit (and Their Fixes)

### 1. `time not implemented on this platform`

**Cause**: `std::time::Instant::now()` panics on `wasm32-unknown-unknown`.

**Fix**: Always use `crate::utils::time_compat::Instant` instead of `std::time::Instant`. The module re-exports `web_time::Instant` on WASM and `std::time::Instant` on native.

```rust
// WRONG — panics on WASM
use std::time::Instant;

// CORRECT
use crate::utils::time_compat::Instant;
```

### 2. `UnsupportedPlatform("navigator not found")` from `getrandom`

**Cause**: `rand` depends on `getrandom`, which needs the `js` feature to use `crypto.getRandomValues()` in browsers.

**Fix**: Already in `Cargo.toml` under WASM-specific dependencies:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom = { version = "0.2", features = ["js"] }
```

If you add a new crate that pulls in `getrandom` transitively, this still applies.

### 3. WebGPU init failure / `requestAdapter` returns null

**Cause**: Using the `webgpu` Bevy feature in the Trunk build. Most mobile browsers only support WebGL2.

**Fix**: The `index.html` Trunk link must NOT enable WebGPU:

```html
<!-- CORRECT — uses WebGL2 -->
<link data-trunk rel="rust" />

<!-- WRONG — crashes on most mobile browsers -->
<link data-trunk rel="rust" data-cargo-features="webgpu" />
```

The `webgpu` Cargo feature exists for opt-in desktop browser testing only.

### 4. Audio decode error / silence on WASM

**Status**: PARTIALLY FIXED — still crashes on mobile (see 4b below).

**Cause**: Bevy's `bevy_audio` pulls `rodio` with Vorbis only. The game generates procedural WAV audio at runtime.

**Fix (applied)**: `rodio` is added as a direct dependency with the `wav` feature:

```toml
rodio = { version = "0.20", default-features = false, features = ["wav"] }
```

WAV samples must be encoded as **PCM 16-bit** (format code 1), not IEEE Float32. The `samples_to_wav_bytes` function in `src/systems/audio.rs` handles this.

### 4b. `UnrecognizedFormat` panic on WASM when pressing Enter (OPEN)

**Error**: `panicked at bevy_audio-0.16.1/src/audio_source.rs:102:56: called Result::unwrap() on an Err value: UnrecognizedFormat`

**When**: Triggers on first `SoundEvent` (pressing Enter fires `MenuSelect`). Game loads and renders fine — crashes only when audio playback is attempted.

**Cause**: Despite `rodio` having the `wav` feature and `samples_to_wav_bytes` producing valid PCM 16-bit WAV, rodio's decoder chain fails to recognize the format on WASM. This works on desktop (native rodio) but fails in the browser. Likely a symphonia/rodio WASM compatibility issue with `Decoder::new()` when running under wasm32.

**Workaround options** (not yet applied):

1. **Make `play_sounds` audio-failure-safe**: The system at `audio.rs:137` uses `ResMut<Assets<AudioSource>>` (hard requirement). If audio init failed or decode will fail, this panics. Change to catch the error:

```rust
pub fn play_sounds(
    mut commands: Commands,
    mut library: ResMut<SoundLibrary>,
    mut events: EventReader<SoundEvent>,
    audio_assets: Option<ResMut<Assets<AudioSource>>>,  // <- Option
) {
    let Some(mut audio_assets) = audio_assets else {
        events.clear();
        return;
    };
    // ... rest unchanged
}
```

2. **Disable audio entirely on WASM**: In the plugin setup, conditionally skip adding the audio systems on `wasm32`:

```rust
#[cfg(not(target_arch = "wasm32"))]
app.add_systems(Update, play_sounds);
```

3. **Investigate rodio WASM decode path**: Run `rodio::Decoder::new(std::io::Cursor::new(&wav_bytes))` in an isolated WASM test to confirm whether the issue is rodio's WAV decoder on WASM or Bevy's wrapping of it.

### 4c. `Resource does not exist: Assets<AudioSource>` (dist/optimized build)

**Error**: `Parameter ResMut<Assets<AudioSource>> failed validation: Resource does not exist`

**When**: Immediately on game start in the `play_sounds` system.

**Cause**: The optimized `dist` build (hash `d4da93eb`, compiled with `wasm-release` profile / `panic="abort"`) appears to have the `AudioPlugin` not initializing the `Assets<AudioSource>` resource. With `panic="abort"`, any initialization failure is fatal without unwinding. The `setup_audio` system handles this gracefully via `Option<ResMut<...>>`, but `play_sounds` does not.

**Fix**: Same as workaround #1 above — make `play_sounds` use `Option<ResMut<Assets<AudioSource>>>`.

### 5. Shader compilation failure on mobile GPU

**Cause**: Complex shader operations (barrel distortion, phosphor bleed) can exceed mobile GPU instruction limits or trigger driver bugs.

**Fix**: The CRT shader uses `#ifndef MOBILE` to gate expensive effects. On mobile, `CrtSettings` is initialized with `curvature_amount: 0.0`, which triggers `is_mobile: true` in the pipeline key, which injects the `MOBILE` shader define.

## Build Pipeline Details

### Cargo profiles

```
cargo build                    → [dev] profile, debug, native
cargo build --release          → [release] opt-level=3, thin LTO, native
trunk build --release          → [release] profile, then wasm-opt
```

The `[profile.wasm-release]` in `Cargo.toml` is defined but **not used by Trunk by default**. Trunk builds with `--release` which uses `[profile.release]`. To use the more aggressive WASM profile, you'd need to pass `--cargo-profile wasm-release` to Trunk (currently not done — see optimization proposals below).

### WASM target features (`.cargo/config.toml`)

```toml
[target.wasm32-unknown-unknown]
rustflags = ["-C", "target-feature=+bulk-memory,+mutable-globals,+nontrapping-fptoint,+sign-ext"]
```

| Feature | Purpose |
|---------|---------|
| `bulk-memory` | Fast `memory.copy` / `memory.fill` — replaces slow byte-by-byte loops |
| `mutable-globals` | Required by wasm-bindgen for stack pointer management |
| `nontrapping-fptoint` | Prevents panics on `f32 as i32` for out-of-range values |
| `sign-ext` | Sign-extension opcodes — smaller and faster integer conversions |

These are supported by all browsers since ~2020. Safe to keep enabled.

### wasm-opt (`Trunk.toml`)

```toml
[tools]
wasm_opt = "version_116"
```

Trunk runs wasm-opt automatically on `--release` builds. The default pass is `-O` (balanced). Can be tuned further (see proposals).

## Deployment

```bash
trunk build --release
# Output: dist/index.html, dist/*.js, dist/*.wasm
```

For embedding in a Next.js portfolio:

```bash
cp dist/*.js   /path/to/next-app/public/games/cyberpunk/game.js
cp dist/*.wasm /path/to/next-app/public/games/cyberpunk/game_bg.wasm
```

---

## Applied Optimizations

### Frame rate cap (30fps on WASM)

`src/app.rs` inserts `WinitSettings` with `UpdateMode::Reactive { wait: 33ms }` on `wasm32`.
This halves GPU/CPU work compared to uncapped 60fps. Input events still trigger immediate
redraws, so responsiveness is preserved.

### Bloom disabled on mobile

The `Bloom` component is only attached to the camera on `QualityTier::Desktop`. On mobile,
the multi-pass bloom render pipeline is skipped entirely. HDR framebuffer is kept (needed for
neon colors > 1.0 and CRT post-processing).

### Entity spawning rate-limited on mobile

Compile-time constants in `src/utils/config.rs`:

| Constant | Desktop | Mobile (WASM) |
|----------|---------|---------------|
| `PLAYER_SHOT_COOLDOWN` | 0.15s | 0.25s |
| `BEAM_SEGMENTS` | 12 | 6 |
| `LASER_CHARGE_PARTICLES` | 8 | 4 |

These reduce per-frame entity counts during intense gameplay (sentinel beams, laser
powerups, rapid shooting).

### `wasm-release` profile active

`Trunk.toml` uses `cargo_profile = "wasm-release"` (`opt-level="z"`, fat LTO, `panic="abort"`).

### Aggressive wasm-opt

`index.html` uses `data-wasm-opt="z"` for maximum size optimization.

---

## Remaining Optimization Proposals

### 1. ~~Use the `wasm-release` profile~~ ✅ DONE

### 2. ~~Aggressive wasm-opt pass~~ ✅ DONE

### 3. Serve with Brotli compression

```toml
# Trunk.toml
[build]
filehash = true
cargo_profile = "wasm-release"    # opt-level="z", fat LTO, panic="abort"

[tools]
wasm_opt = "version_116"
```

`opt-level = "z"` + `lto = "fat"` + `panic = "abort"` should shrink the 14.4MB binary significantly. `panic = "abort"` alone removes all unwind tables and the `std::panic` machinery.

### 2. Aggressive wasm-opt pass

The current default is `-O`. Switch to `-Oz` with specific passes:

```toml
# Trunk.toml
[build]
filehash = true
cargo_profile = "wasm-release"

[tools]
wasm_opt = "version_116"

# Add to index.html:
# <link data-trunk rel="rust" data-wasm-opt="z" />
```

Or run manually after build for maximum control:

```bash
wasm-opt -Oz --enable-bulk-memory --enable-sign-ext \
  --vacuum --duplicate-function-elimination --inlining-optimizing \
  dist/cyberpunk_rpg_bg.wasm -o dist/cyberpunk_rpg_bg.wasm
```

Expected: ~20-30% smaller binary.

### 3. Serve with Brotli compression

The 14.4MB `.wasm` file compresses extremely well. Configure your web server or CDN:

```
cyberpunk_rpg_bg.wasm      14.4 MB (raw)
cyberpunk_rpg_bg.wasm.br   ~3-4 MB (brotli, ~75% reduction)
cyberpunk_rpg_bg.wasm.gz   ~4-5 MB (gzip, ~65% reduction)
```

Pre-compress at build time:

```bash
# After trunk build --release
brotli -q 11 dist/*.wasm
gzip -k -9 dist/*.wasm
```

Then configure your hosting (Vercel, nginx, etc.) to serve the compressed version with `Content-Encoding: br`.

### 4. Streaming WASM compilation

Replace the synchronous `init()` call with `WebAssembly.instantiateStreaming` for faster startup. The Trunk-generated JS already does this if the server sends `Content-Type: application/wasm`. Verify your hosting returns this MIME type — some static hosts default to `application/octet-stream`, which forces a slower download-then-compile path.

### 5. Drop `tonemapping_luts` on WASM

The `tonemapping_luts` Bevy feature embeds lookup tables for tonemapping algorithms. These add to the binary size. If the game only uses one tonemapping mode (e.g., `Tonemapping::TonyMcMapface` or `Tonemapping::None`), this feature can be conditionally excluded:

```toml
# Shared features
[dependencies.bevy]
version = "0.16.1"
default-features = false
features = [
    "bevy_asset", "bevy_audio", "bevy_color", "bevy_core_pipeline",
    "bevy_gizmos", "bevy_log", "bevy_render", "bevy_sprite", "bevy_state",
    "bevy_text", "bevy_ui", "bevy_window", "bevy_winit", "default_font",
    "hdr", "std", "webgl2",
]

# Native-only features (tonemapping LUTs are large, only needed on desktop)
[target.'cfg(not(target_arch = "wasm32"))'.dependencies.bevy]
version = "0.16.1"
features = ["multi_threaded", "x11", "tonemapping_luts"]
```

Expected: noticeable binary size reduction.

### 6. Lazy audio generation

Currently all 22 sound effects are generated at startup (`setup_audio`). On mobile, defer non-critical sounds:

```rust
fn setup_audio(mut commands: Commands, quality: Res<QualityTier>) {
    let critical = [SoundEffect::PlayerShoot, SoundEffect::PlayerHit, SoundEffect::Explosion];
    // Generate critical sounds immediately
    // Queue the rest for lazy generation on first play
}
```

This would reduce initial load time by spreading CPU work across the first few seconds of gameplay.

### 7. Entity budget cap

Add a hard entity cap for mobile to prevent frame drops during intense phases (boss desperation mode, multiple hazard zones, particle bursts):

```rust
fn spawn_ambient_particles(
    query: Query<&AmbientParticle>,
    quality: Res<QualityTier>,
    // ...
) {
    let max = match *quality {
        QualityTier::Mobile => 50,
        QualityTier::Desktop => 200,
    };
    if query.iter().count() >= max { return; }
    // spawn...
}
```

Apply similar caps to projectiles, afterimages, and shatter particles.

### 8. Touch input layer

The game currently relies on keyboard input. For actual mobile playability, add a virtual joystick / touch control layer:

```rust
// In src/core/player/systems.rs, add alongside keyboard input:
#[cfg(target_arch = "wasm32")]
fn touch_input(touches: Res<Touches>, mut query: Query<&mut Transform, With<Player>>) {
    for touch in touches.iter() {
        // Map touch position to player movement
    }
}
```

This is a feature addition, not a build optimization, but it's the biggest gap for actual mobile use.

### 9. Reduce `hdr` pipeline overhead

The HDR pipeline is expensive on mobile GPUs. Consider a non-HDR path for mobile that skips the HDR framebuffer and tonemapping pass entirely, using direct SDR output with emulated bloom via sprite overlay:

```rust
let camera_bundle = match *quality {
    QualityTier::Desktop => Camera2d::default(), // HDR + bloom + CRT
    QualityTier::Mobile => {
        // SDR camera, lighter bloom, no tonemapping
    }
};
```

This would be the single biggest performance win on low-end mobile GPUs.

### 10. Build size audit

Run `twiggy` to find what's actually taking space in the binary:

```bash
cargo install twiggy
twiggy top -n 30 dist/cyberpunk_rpg_bg.wasm
twiggy dominators dist/cyberpunk_rpg_bg.wasm | head -50
```

This reveals which crates/functions contribute most to the 14.4MB. Common culprits in Bevy games: `wgpu` shader compilation tables, `naga` (shader translator), embedded fonts, tonemapping LUTs.
