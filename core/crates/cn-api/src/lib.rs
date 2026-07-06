//! cn-api: the native-safe facade over the core crates (ADR-003 A-B5).
//!
//! The single public API surface for both frontends: cn-wasm wraps this crate
//! with wasm-bindgen (cdylib, target-gated deps); the cn CLI links it
//! directly (rlib). No wasm dependencies may ever appear here.
//!
//! Placeholder scaffolded in Phase 1; implemented in Phase 2 per ADR-003.
