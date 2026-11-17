// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// Binary entry point: `haulmer-connect mcp` serves MCP on stdin/stdout, anything else opens
/// the app ([`haulmer_connect_lib::run`]).
///
/// # Example
///
/// ```text
/// $ pnpm tauri dev
/// $ haulmer-connect mcp
/// ```
fn main() {
    if std::env::args().nth(1).as_deref() == Some("mcp") {
        if let Err(error) = haulmer_connect_lib::run_mcp() {
            eprintln!("haulmer-connect mcp: {error}");
            std::process::exit(1);
        }
    } else {
        haulmer_connect_lib::run()
    }
}
