# Building for Web (WASM)

## Prerequisites

```bash
# Rust WASM target
rustup target add wasm32-unknown-unknown

# Trunk (WASM build tool)
cargo install trunk
```

## Build

```bash
trunk build --release
```

Output lands in `dist/` — an `index.html`, a `.js` wrapper, and a `.wasm` binary (~38MB).

## Key WASM Compatibility Fixes

The following changes are required for the game to run in browsers. Without them, the WASM build will panic at runtime.

### 1. Remove `webgpu` feature from Trunk build

In `index.html`, the Trunk link must **not** include `data-cargo-features="webgpu"`. WebGPU has limited browser support; the default WebGL2 backend works everywhere.

```html
<!-- CORRECT -->
<link data-trunk rel="rust" />

<!-- WRONG — will crash in most browsers -->
<link data-trunk rel="rust" data-cargo-features="webgpu" />
```

### 2. Add `rodio` WAV decoder

Bevy's `bevy_audio` enables `rodio` with only the `vorbis` feature. The game generates procedural WAV audio, but there's no WAV decoder compiled in by default. Add `rodio` with the `wav` feature to `Cargo.toml`:

```toml
[dependencies]
rodio = { version = "0.20", default-features = false, features = ["wav"] }
```

### 3. Add `web-time` for WASM-compatible `Instant`

`std::time::Instant::now()` panics on `wasm32-unknown-unknown` with "time not implemented on this platform". The crate already has `src/utils/time_compat.rs` which re-exports `web_time::Instant` on WASM. Add the dependency:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
web-time = "1.1"
```

Then use `crate::utils::time_compat::Instant` everywhere instead of `std::time::Instant`.

### 4. WAV encoding must use PCM 16-bit

In `src/systems/audio.rs`, the `samples_to_wav_bytes` function must encode as **PCM 16-bit** (format code 1), not IEEE float 32-bit (format code 3). The `hound` WAV decoder (pulled in by `rodio`) supports both, but PCM 16-bit is the most universally compatible format.

## Deploying to Next.js Portfolio

After `trunk build --release`, copy the build artifacts to the Next.js project:

```bash
# From the game repo root:
cp dist/*.js   /path/to/next-app/public/games/cyberpunk/game.js
cp dist/*.wasm /path/to/next-app/public/games/cyberpunk/game_bg.wasm
```

The HTML page at `public/projects/cyberpunk-arcade/index.html` loads these files via:

```js
import('/games/cyberpunk/game.js')
  .then(mod => mod.default({ module_or_path: '/games/cyberpunk/game_bg.wasm' }))
```

No changes needed to the HTML — just replace the two files.
