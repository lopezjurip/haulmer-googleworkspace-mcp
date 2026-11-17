//! Tauri shell of haulmer-connect: one window, IPC commands only, no local HTTP server.
//! Every command calls the matching `haulmer_connect_core::Core` method 1:1, with no logic.

use std::path::PathBuf;

use haulmer_connect_core::{
    Core, CoreError, InstalledSkills, KeyStorage, Paths, RevokeOutcome, SaveOutcome, Selection,
    ServiceInfo, Status, mcp, scopes,
};
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

/// Bundle identifier of the app (`identifier` in `tauri.conf.json`); the app data folder is named after it.
const IDENTIFIER: &str = "com.haulmer.connect";

/// Event emitted with the Google sign-in URL while `save` waits for the login.
const AUTH_URL_EVENT: &str = "auth-url";

/// `Core::status`.
///
/// # Example
///
/// ```js
/// const status = await invoke("status");
/// ```
#[tauri::command(async)]
fn status(core: State<'_, Core>) -> Result<Status, CoreError> {
    core.status()
}

/// `Core::save`; the sign-in URL opens in the default browser and is also emitted as the
/// `auth-url` event, so the UI can offer it again if the browser did not open.
///
/// # Example
///
/// ```js
/// const outcome = await invoke("save", { selection, confirmReplace: false });
/// ```
#[tauri::command(async)]
fn save(
    app: AppHandle,
    core: State<'_, Core>,
    selection: Selection,
    confirm_replace: bool,
) -> Result<SaveOutcome, CoreError> {
    let on_auth_url = |url: &str| {
        let _ = app.opener().open_url(url, None::<&str>);
        let _ = app.emit(AUTH_URL_EVENT, url);
    };
    core.save(&selection, confirm_replace, &on_auth_url)
}

/// `Core::cancel_login`.
///
/// # Example
///
/// ```js
/// await invoke("cancel_login");
/// ```
#[tauri::command]
fn cancel_login(core: State<'_, Core>) -> Result<(), CoreError> {
    core.cancel_login()
}

/// `Core::revoke_all`; `confirmForeign` allows revoking credentials created by another app.
///
/// # Example
///
/// ```js
/// const outcome = await invoke("revoke_all", { confirmForeign: false });
/// ```
#[tauri::command(async)]
fn revoke_all(core: State<'_, Core>, confirm_foreign: bool) -> Result<RevokeOutcome, CoreError> {
    core.revoke_all(confirm_foreign)
}

/// `Core::install_skills`.
///
/// # Example
///
/// ```js
/// const installed = await invoke("install_skills");
/// ```
#[tauri::command(async)]
fn install_skills(core: State<'_, Core>) -> Result<Vec<InstalledSkills>, CoreError> {
    core.install_skills()
}

/// `Core::set_config_dir`; `null` resets to the gws default folder.
///
/// # Example
///
/// ```js
/// await invoke("set_config_dir", { dir: "/Users/ana/.config/haulmer-gws" });
/// ```
#[tauri::command]
fn set_config_dir(core: State<'_, Core>, dir: Option<PathBuf>) -> Result<(), CoreError> {
    core.set_config_dir(dir)
}

/// `Core::set_key_storage` (`"auto"`, `"keyring"` or `"file"`).
///
/// # Example
///
/// ```js
/// await invoke("set_key_storage", { storage: "file" });
/// ```
#[tauri::command]
fn set_key_storage(core: State<'_, Core>, storage: KeyStorage) -> Result<(), CoreError> {
    core.set_key_storage(storage)
}

/// `scopes::catalogue`: plain-language labels for every service row.
///
/// # Example
///
/// ```js
/// const services = await invoke("catalogue");
/// ```
#[tauri::command]
fn catalogue() -> Vec<ServiceInfo> {
    scopes::catalogue()
}

/// `scopes::default_selection`.
///
/// # Example
///
/// ```js
/// const selection = await invoke("default_selection");
/// ```
#[tauri::command]
fn default_selection() -> Selection {
    scopes::default_selection()
}

/// `Core::add_to_claude_desktop` with this executable; returns the config file path.
///
/// # Example
///
/// ```js
/// const path = await invoke("add_to_claude_desktop");
/// ```
#[tauri::command]
fn add_to_claude_desktop(core: State<'_, Core>) -> Result<PathBuf, CoreError> {
    core.add_to_claude_desktop(&std::env::current_exe()?)
}

/// `Core::add_to_chatgpt` with this executable; returns the config file path.
///
/// # Example
///
/// ```js
/// const path = await invoke("add_to_chatgpt");
/// ```
#[tauri::command]
fn add_to_chatgpt(core: State<'_, Core>) -> Result<PathBuf, CoreError> {
    core.add_to_chatgpt(&std::env::current_exe()?)
}

/// Starts the Tauri application with one shared `Core` built from the real folders.
///
/// # Example
///
/// ```no_run
/// haulmer_connect_lib::run();
/// ```
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let paths = Paths {
                home: app.path().home_dir()?,
                app_data: app.path().app_data_dir()?,
            };
            app.manage(Core::system(paths));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            status,
            save,
            cancel_login,
            revoke_all,
            install_skills,
            set_config_dir,
            set_key_storage,
            catalogue,
            default_selection,
            add_to_claude_desktop,
            add_to_chatgpt
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Runs the MCP server on stdin/stdout (`haulmer-connect mcp`), with no window. Uses the same
/// folders as the app: `home` and `<data dir>/com.haulmer.connect` as Tauri's `app_data_dir`
/// (`dirs::data_dir`: macOS `~/Library/Application Support`, Windows `%APPDATA%`, Linux
/// `$XDG_DATA_HOME` or `~/.local/share`).
///
/// # Example
///
/// ```no_run
/// haulmer_connect_lib::run_mcp().unwrap();
/// ```
pub fn run_mcp() -> Result<(), CoreError> {
    let home_var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
    let home = PathBuf::from(std::env::var_os(home_var).unwrap_or_default());
    let data_dir: PathBuf;
    if cfg!(target_os = "macos") {
        data_dir = home.join("Library/Application Support");
    } else if cfg!(windows) {
        data_dir = PathBuf::from(std::env::var_os("APPDATA").unwrap_or_default());
    } else {
        match std::env::var_os("XDG_DATA_HOME") {
            Some(dir) if !dir.is_empty() => data_dir = PathBuf::from(dir),
            _ => data_dir = home.join(".local/share"),
        }
    }
    let core = Core::system(Paths {
        app_data: data_dir.join(IDENTIFIER),
        home,
    });
    mcp::serve(&core, std::io::stdin().lock(), std::io::stdout().lock())
}
