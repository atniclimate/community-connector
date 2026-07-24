//! Event-sourced store and append-only operation log.

pub mod authz;
pub mod fold;
#[cfg(not(target_arch = "wasm32"))]
pub mod log;
pub mod op;
#[cfg(not(target_arch = "wasm32"))]
pub mod seam;

pub use authz::*;
pub use fold::*;
#[cfg(not(target_arch = "wasm32"))]
pub use log::*;
pub use op::*;
#[cfg(not(target_arch = "wasm32"))]
pub use seam::*;
