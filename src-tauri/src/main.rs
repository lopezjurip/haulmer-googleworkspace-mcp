// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Binary entry point; delegates to [`haulmer_connect_lib::run`].
///
/// # Example
///
/// ```text
/// $ pnpm tauri dev
/// ```
fn main() {
    haulmer_connect_lib::run()
}
