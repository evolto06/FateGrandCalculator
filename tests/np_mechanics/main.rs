//! NP mechanic runtime, damage and import tests.

mod components_damage;
#[cfg(not(target_arch = "wasm32"))]
mod import;
mod runtime;
mod space_eresh_damage;
#[cfg(not(target_arch = "wasm32"))]
mod space_eresh_import;
