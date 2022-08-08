#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    // Create an application window.
    let app = radio::App::default();
    // Get default window properties, such as always-on-top, minimized,
    // drag-and-drop support, etc.
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
