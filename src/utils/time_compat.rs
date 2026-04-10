// Platform-compatible Instant for WASM + native
// On WASM, std::time::Instant panics — use web-time instead

#[cfg(target_arch = "wasm32")]
pub use web_time::Instant;

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;
