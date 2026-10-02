#[path = "np_mechanics/components_damage.rs"]
mod components_damage;
#[cfg(not(target_arch = "wasm32"))]
#[path = "np_mechanics/import.rs"]
mod import;
#[path = "np_mechanics/runtime.rs"]
mod runtime;
#[path = "np_mechanics/space_eresh_damage.rs"]
mod space_eresh_damage;
#[cfg(not(target_arch = "wasm32"))]
#[path = "np_mechanics/space_eresh_import.rs"]
mod space_eresh_import;
