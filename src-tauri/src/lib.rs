//! Tauri shell of haulmer-connect: one window, IPC commands only, no local HTTP server.

/// Starts the Tauri application.
///
/// # Example
///
/// ```no_run
/// haulmer_connect_lib::run();
/// ```
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
