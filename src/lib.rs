pub mod app;
pub mod enemies;
pub mod game;
pub mod mempool;
pub mod resources;
pub mod towers;
pub mod transactions;
pub mod ui;

pub use app::run_app;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// WASM entry point — called automatically by the browser after the JS glue loads.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    run_app();
}
