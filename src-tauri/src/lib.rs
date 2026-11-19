//! Tauri shell of haulmer-connect: one window, IPC commands only, no local HTTP server.
//! Every command calls the matching `haulmer_connect_core::Core` method 1:1, with no logic.

use std::path::PathBuf;
use std::process::Stdio;

use haulmer_connect_core::{
    BusinessConnector, BusinessConnectorInfo, BusinessSelection, Core, CoreError, KeyStorage,
    Level, Paths, RevokeOutcome, SaveOutcome, Selection, Service, ServiceInfo, Status, mcp,
    mcp_clients, scopes, skills,
};
use serde_json::Value;
use tauri::{
    AppHandle, Emitter, Manager, State, WindowEvent,
    image::Image,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};
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
    business_selection: Option<BusinessSelection>,
    confirm_replace: bool,
) -> Result<SaveOutcome, CoreError> {
    set_tray_menu(&app, TrayState::Connecting);
    let on_auth_url = |url: &str| {
        let _ = app.opener().open_url(url, None::<&str>);
        let _ = app.emit(AUTH_URL_EVENT, url);
    };
    let result = core.save_with_business(
        &selection,
        business_selection
            .as_ref()
            .unwrap_or(&BusinessSelection::new()),
        confirm_replace,
        &on_auth_url,
    );
    match &result {
        Ok(SaveOutcome::Connected(status)) => {
            set_tray_menu(&app, TrayState::Status(status.clone()))
        }
        Ok(SaveOutcome::NeedsReplaceConfirmation) => refresh_tray_menu(&app, &core),
        Err(_) => set_tray_menu(&app, TrayState::Error),
    }
    result
}

/// `Core::cancel_login`.
///
/// # Example
///
/// ```js
/// await invoke("cancel_login");
/// ```
#[tauri::command]
fn cancel_login(app: AppHandle, core: State<'_, Core>) -> Result<(), CoreError> {
    let result = core.cancel_login();
    refresh_tray_menu(&app, &core);
    result
}

/// `Core::revoke_all`; `confirmForeign` allows revoking credentials created by another app.
///
/// # Example
///
/// ```js
/// const outcome = await invoke("revoke_all", { confirmForeign: false });
/// ```
#[tauri::command(async)]
fn revoke_all(
    app: AppHandle,
    core: State<'_, Core>,
    confirm_foreign: bool,
) -> Result<RevokeOutcome, CoreError> {
    let result = core.revoke_all(confirm_foreign);
    refresh_tray_menu(&app, &core);
    result
}

/// Exports the granted skills to a user-selected path.
#[tauri::command(async)]
fn export_skills_zip(core: State<'_, Core>, path: PathBuf) -> Result<(), CoreError> {
    std::fs::write(path, core.export_skills_zip()?)?;
    Ok(())
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
fn catalogue(core: State<'_, Core>) -> Result<Vec<ServiceInfo>, CoreError> {
    let granted = core.status()?.granted;
    Ok(scopes::catalogue_for(&granted))
}

/// Business APIs reserved for dedicated connectors that share the OAuth account.
///
/// # Example
///
/// ```js
/// const connectors = await invoke("business_catalogue");
/// ```
#[tauri::command]
fn business_catalogue(core: State<'_, Core>) -> Result<Vec<BusinessConnectorInfo>, CoreError> {
    let granted = core.status()?.business_granted;
    Ok(scopes::business_catalogue_for(&granted))
}

/// Returns the connected user's YouTube channel through the shared OAuth account.
///
/// # Example
///
/// ```js
/// const channel = await invoke("youtube_channel");
/// ```
#[tauri::command(async)]
fn youtube_channel(core: State<'_, Core>) -> Result<Value, CoreError> {
    core.youtube_channel()
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

/// `mcp_clients::install_prompt` for this executable. The prompt tells an agent how to connect
/// itself to haulmer-connect. It fails with `UnstableExe` when this executable runs from a path
/// that will not last (for example a macOS App Translocation folder).
///
/// # Example
///
/// ```js
/// const { text, warning } = await invoke("install_prompt");
/// ```
#[tauri::command]
fn install_prompt() -> Result<mcp_clients::InstallPrompt, CoreError> {
    mcp_clients::install_prompt(&std::env::current_exe()?)
}

/// `skills::skills_add_command`: the `npx skills add` command of the Skills window, the same as in
/// the install prompt.
///
/// # Example
///
/// ```js
/// const command = await invoke("skills_install_command"); // "npx -y skills add … -s gws-shared …"
/// ```
#[tauri::command]
fn skills_install_command() -> String {
    skills::skills_add_command()
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
        // Must be the first plugin: a second launch shows the settings window of the running app.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            show_window(app, "main")
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            // The menu is set by `refresh_tray_menu` below, once the `Core` exists.
            let tray = TrayIconBuilder::with_id("main")
                .icon(Image::from_bytes(include_bytes!("../icons/tray-icon.png"))?)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "open" | "connect" => show_window(app, "main"),
                    "skills" => show_window(app, "skills"),
                    "connect-agent" => show_refreshed_window(app, "install"),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .build(app)?;
            tray.set_icon_as_template(true)?;
            let paths = Paths {
                home: app.path().home_dir()?,
                app_data: app.path().app_data_dir()?,
            };
            app.manage(Core::system(paths));
            refresh_tray_menu(app.handle(), &app.state::<Core>());
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            // `run_mcp` launches the app with `--show` when an agent asks to open it.
            if std::env::args().any(|arg| arg == "--show") {
                show_window(app.handle(), "main");
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            status,
            save,
            cancel_login,
            revoke_all,
            export_skills_zip,
            set_config_dir,
            set_key_storage,
            catalogue,
            business_catalogue,
            youtube_channel,
            default_selection,
            install_prompt,
            skills_install_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[derive(Clone)]
enum TrayState {
    Connecting,
    Status(Status),
    Error,
}

fn refresh_tray_menu(app: &AppHandle, core: &Core) {
    let state = match core.status() {
        Ok(status) => TrayState::Status(status),
        Err(error) => {
            eprintln!("Could not refresh tray status: {error}");
            TrayState::Error
        }
    };
    set_tray_menu(app, state);
}

fn set_tray_menu(app: &AppHandle, state: TrayState) {
    let Some(tray) = app.tray_by_id("main") else {
        return;
    };

    let (status_text, account_text, permissions) = match state {
        TrayState::Connecting => (
            "🟡 Conectando…".to_string(),
            "Abrir haulmer-connect".to_string(),
            Vec::new(),
        ),
        TrayState::Error => (
            "🔴 Estado no disponible".to_string(),
            "Abrir haulmer-connect".to_string(),
            Vec::new(),
        ),
        TrayState::Status(status) => {
            let (indicator, account_text) = match status.account.as_deref() {
                Some(account) if status.token_valid => ("🟢 Conectado", account.to_string()),
                Some(account) => ("🟡 Reautenticación requerida", account.to_string()),
                None => ("⚪ Sin conexión", "Conectar cuenta de Google".to_string()),
            };
            (
                indicator.to_string(),
                account_text,
                active_permission_labels(&status),
            )
        }
    };

    let status_item = match MenuItemBuilder::with_id("status", status_text)
        .enabled(false)
        .build(app)
    {
        Ok(item) => item,
        Err(error) => {
            eprintln!("Could not build tray status item: {error}");
            return;
        }
    };
    let account_item = match MenuItemBuilder::with_id(
        if account_text == "Conectar cuenta de Google" {
            "connect"
        } else {
            "open"
        },
        account_text,
    )
    .build(app)
    {
        Ok(item) => item,
        Err(error) => {
            eprintln!("Could not build tray account item: {error}");
            return;
        }
    };
    let open = match MenuItemBuilder::with_id("open", "Abrir configuración").build(app) {
        Ok(item) => item,
        Err(error) => {
            eprintln!("Could not build tray open item: {error}");
            return;
        }
    };
    let skills = match MenuItemBuilder::with_id("skills", "Abrir skills").build(app) {
        Ok(item) => item,
        Err(error) => {
            eprintln!("Could not build tray skills item: {error}");
            return;
        }
    };
    let connect_agent =
        match MenuItemBuilder::with_id("connect-agent", "Conectar un agente…").build(app) {
            Ok(item) => item,
            Err(error) => {
                eprintln!("Could not build tray connect agent item: {error}");
                return;
            }
        };
    let quit = match MenuItemBuilder::with_id("quit", "Salir de haulmer-connect").build(app) {
        Ok(item) => item,
        Err(error) => {
            eprintln!("Could not build tray quit item: {error}");
            return;
        }
    };
    let permission_items: Vec<_> = permissions
        .into_iter()
        .enumerate()
        .filter_map(|(index, label)| {
            MenuItemBuilder::with_id(format!("permission-{index}"), label)
                .enabled(false)
                .build(app)
                .map_err(|error| {
                    eprintln!("Could not build tray permission item: {error}");
                    error
                })
                .ok()
        })
        .collect();

    let mut builder = MenuBuilder::new(app)
        .item(&status_item)
        .item(&account_item)
        .separator();
    for item in &permission_items {
        builder = builder.item(item);
    }
    let menu = match builder
        .separator()
        .item(&open)
        .item(&skills)
        .item(&connect_agent)
        .separator()
        .item(&quit)
        .build()
    {
        Ok(menu) => menu,
        Err(error) => {
            eprintln!("Could not build tray menu: {error}");
            return;
        }
    };
    if let Err(error) = tray.set_menu(Some(menu)) {
        eprintln!("Could not update tray menu: {error}");
    }
}

fn active_permission_labels(status: &Status) -> Vec<String> {
    let mut permissions = Vec::new();
    for service in Service::ALL {
        if status.granted.contains(service, Level::Write) {
            permissions.push(format!("{} — Acceso completo", service_label(service)));
        } else if status.granted.contains(service, Level::Read) {
            permissions.push(format!("{} — Solo lectura", service_label(service)));
        }
    }
    for connector in [
        BusinessConnector::Youtube,
        BusinessConnector::GoogleAds,
        BusinessConnector::Analytics,
        BusinessConnector::SearchConsole,
        BusinessConnector::BusinessProfile,
        BusinessConnector::MerchantCenter,
    ] {
        if status.business_granted.contains(connector, Level::Write) {
            permissions.push(format!("{} — Acceso completo", business_label(connector)));
        } else if status.business_granted.contains(connector, Level::Read) {
            permissions.push(format!("{} — Solo lectura", business_label(connector)));
        }
    }
    permissions
}

fn service_label(service: Service) -> &'static str {
    match service {
        Service::Gmail => "Gmail",
        Service::Calendar => "Calendar",
        Service::Chat => "Chat",
        Service::Meet => "Meet",
        Service::Drive => "Drive",
        Service::Docs => "Docs",
        Service::Sheets => "Sheets",
        Service::Slides => "Slides",
        Service::Forms => "Forms",
        Service::Tasks => "Tasks",
        Service::People => "People",
        Service::AdminReports => "Admin Reports",
        Service::AdminDirectory => "Admin Directory",
        Service::AdminGroups => "Admin Groups",
        Service::CloudIdentity => "Cloud Identity",
    }
}

fn business_label(connector: BusinessConnector) -> &'static str {
    match connector {
        BusinessConnector::Youtube => "YouTube",
        BusinessConnector::GoogleAds => "Google Ads",
        BusinessConnector::Analytics => "Analytics",
        BusinessConnector::SearchConsole => "Search Console",
        BusinessConnector::BusinessProfile => "Business Profile",
        BusinessConnector::MerchantCenter => "Merchant Center",
    }
}

/// Unminimizes, shows and focuses the window with this label. Does nothing when no window has it.
///
/// # Example
///
/// ```ignore
/// show_window(app, "main"); // from the tray menu or a second launch
/// ```
fn show_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Tells the window `label` to reload its data (`<label>-window-shown`, sent to that window only),
/// then shows and focuses it. The event goes first, so the window resets before the person sees it.
///
/// # Example
///
/// ```ignore
/// show_refreshed_window(app, "install"); // from the "connect-agent" tray item
/// ```
fn show_refreshed_window(app: &AppHandle, label: &str) {
    if app.get_webview_window(label).is_none() {
        return;
    }
    if let Err(error) = app.emit_to(label, &format!("{label}-window-shown"), ()) {
        eprintln!("Could not notify the {label} window: {error}");
    }
    show_window(app, label);
}

/// Launches the GUI app (`<this executable> --show`) and returns at once, without waiting for it.
/// The child is detached from the MCP client's process group, so it survives when the client stops
/// the server. If the app already runs, single-instance shows its window and the child exits.
///
/// # Example
///
/// ```ignore
/// open_app_detached()?; // called by the `open_haulmer_connect` MCP tool
/// ```
fn open_app_detached() -> Result<(), CoreError> {
    let mut command = std::process::Command::new(std::env::current_exe()?);
    command
        .arg("--show")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    let mut child = command.spawn()?;
    // Reap the child in the background so that no zombie stays.
    std::thread::spawn(move || child.wait());
    Ok(())
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
    mcp::serve(
        &core,
        &open_app_detached,
        std::io::stdin().lock(),
        std::io::stdout().lock(),
    )
}
