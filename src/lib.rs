//! Shared, platform-independent logic for FateGrandCalculator.

pub mod damage;
pub mod loader;
pub mod model;
pub mod np_mechanics;
#[cfg(not(target_arch = "wasm32"))]
pub mod servant_data;
