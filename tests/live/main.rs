//! Tests against the live Atlas Academy API. They are ignored by default;
//! run them with `cargo test --test live -- --ignored`.
#![cfg(not(target_arch = "wasm32"))]

mod atlas_additional;
mod full_update;
