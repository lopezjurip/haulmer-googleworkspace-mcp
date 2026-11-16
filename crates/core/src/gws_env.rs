//! T04 — the environment and working folder of every gws call, plus credential helpers.

use std::path::Path;

use crate::{CommandSpec, Core, CoreError};

/// Ownership marker written into the config folder after a successful haulmer-connect login.
///
/// # Example
///
/// ```
/// assert_eq!(haulmer_connect_core::gws_env::MARKER_FILE, "haulmer-connect.json");
/// ```
pub const MARKER_FILE: &str = "haulmer-connect.json";

impl Core {
    /// Builds the command for one gws call: `program args...` with the gws env
    /// (config folder, key backend, client id/secret, `NO_COLOR=1`, removed credential
    /// overrides) and cwd `<app_data>/run` (created if missing).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{Core, Paths};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// let command = core.gws_command("gws".as_ref(), &["auth", "status"]).unwrap();
    /// ```
    pub fn gws_command(&self, program: &Path, args: &[&str]) -> Result<CommandSpec, CoreError> {
        let _ = (program, args);
        Err(CoreError::NotImplemented("gws_env::gws_command (T04)"))
    }
}

/// Whether gws credentials exist in `config_dir` (`credentials.enc` or `credentials.json`).
///
/// # Example
///
/// ```should_panic
/// use haulmer_connect_core::gws_env::credentials_exist;
/// assert!(!credentials_exist("/nonexistent".as_ref()));
/// ```
pub fn credentials_exist(config_dir: &Path) -> bool {
    let _ = config_dir;
    todo!("T04")
}

/// Whether `config_dir` holds credentials not created by haulmer-connect (no marker).
///
/// # Example
///
/// ```should_panic
/// use haulmer_connect_core::gws_env::foreign_credentials;
/// assert!(!foreign_credentials("/nonexistent".as_ref()));
/// ```
pub fn foreign_credentials(config_dir: &Path) -> bool {
    let _ = config_dir;
    todo!("T04")
}

/// Writes the ownership marker [`MARKER_FILE`] into `config_dir`.
///
/// # Example
///
/// ```no_run
/// haulmer_connect_core::gws_env::write_marker("/h/.config/gws".as_ref()).unwrap();
/// ```
pub fn write_marker(config_dir: &Path) -> Result<(), CoreError> {
    let _ = config_dir;
    Err(CoreError::NotImplemented("gws_env::write_marker (T04)"))
}

/// Removes the ownership marker from `config_dir` (no error if it is absent).
///
/// # Example
///
/// ```no_run
/// haulmer_connect_core::gws_env::remove_marker("/h/.config/gws".as_ref()).unwrap();
/// ```
pub fn remove_marker(config_dir: &Path) -> Result<(), CoreError> {
    let _ = config_dir;
    Err(CoreError::NotImplemented("gws_env::remove_marker (T04)"))
}
