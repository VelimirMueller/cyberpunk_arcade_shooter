<div align="center">

# Cyberpunk Arcade Shooter

[![CI](https://github.com/VelimirMueller/cyberpunk_arcade_shooter/actions/workflows/ci.yml/badge.svg)](https://github.com/VelimirMueller/cyberpunk_arcade_shooter/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024_edition-orange.svg)](https://www.rust-lang.org/)
[![Bevy](https://img.shields.io/badge/Bevy-0.16-blue.svg)](https://bevyengine.org/)

**A neon-drenched arcade shooter built with [Rust](https://www.rust-lang.org/) and the [Bevy](https://bevyengine.org/) engine.**

*Step into a minimalist, glowing world where every shape pulses with danger.*

</div>

---

## About

Cyberpunk Arcade Shooter is a fast-paced, bloom-soaked geometry shooter where sleek visuals meet brutal intensity. Engage in multi-stage boss battles, dodge through bullet hells, and unleash chaos in a neon-lit arena of pure arcade action.

## Features

- **High-performance Rust + Bevy** — buttery-smooth gameplay powered by an ECS architecture
- **Bloom-soaked neon aesthetic** — every shape glows, every explosion radiates
- **Multi-stage boss fights** — evolving mechanics and unique attack patterns per phase
- **CRT post-processing** — scanlines, vignette, and barrel distortion for that retro feel
- **Tight arcade controls** — responsive movement and shooting that feels just right
- **WASM support** — play directly in the browser via WebAssembly

## Controls

| Action | Key |
| --- | --- |
| Move | `W` / `A` / `S` / `D` |
| Shoot | `Space` |

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable, 2024 edition)
- A GPU with Vulkan, Metal, or DirectX 12 support

### Build & Run

```bash
git clone https://github.com/VelimirMueller/cyberpunk_arcade_shooter.git
cd cyberpunk_arcade_shooter
cargo run --release
```

### WASM (Browser)

To build for the web, install the WASM target and use the included build tooling:

```bash
rustup target add wasm32-unknown-unknown
cargo build --release --target wasm32-unknown-unknown
```

See [WASM_BUILD.md](WASM_BUILD.md) for full instructions on building and serving the web version.

### Platform Notes

| Platform | Notes |
| --- | --- |
| Windows | Works out of the box |
| macOS | Requires Metal support and a recent OS version |
| Linux | Install `libasound2-dev`, `libudev-dev`, and Vulkan drivers |

## Project Structure

```
cyberpunk_arcade_shooter/
├── src/             # Game source code
├── assets/          # Sprites, shaders, and audio
├── tests/           # Integration tests
├── dist/            # WASM distribution files
├── .github/         # CI workflows
├── Cargo.toml       # Rust dependencies and metadata
└── index.html       # Web entrypoint for WASM builds
```

## Contributing

Contributions are welcome! Feel free to open an issue or submit a pull request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Make sure CI passes (`cargo fmt --check && cargo clippy -- -D warnings && cargo test`)
4. Commit your changes (`git commit -m 'feat: add amazing feature'`)
5. Push to the branch (`git push origin feature/amazing-feature`)
6. Open a Pull Request

## License

Distributed under the MIT License. See [LICENSE](LICENSE) for details.
