//! Shared data types of the core contract. Every ticket uses these; none renames them.

use std::collections::BTreeSet;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// A Google Workspace service the person can grant to their agents.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::Service;
/// assert_eq!(Service::ALL.len(), 5);
/// assert_eq!(serde_json::to_string(&Service::Gmail).unwrap(), "\"gmail\"");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Service {
    Gmail,
    Calendar,
    Chat,
    Meet,
    Drive,
}

impl Service {
    /// Every service, in the order the UI shows them.
    pub const ALL: [Service; 5] = [
        Service::Gmail,
        Service::Calendar,
        Service::Chat,
        Service::Meet,
        Service::Drive,
    ];

    /// The service's lowercase name, as used in gws skill folders (`gws-<slug>*`) and JSON.
    ///
    /// # Example
    ///
    /// ```
    /// use haulmer_connect_core::Service;
    /// assert_eq!(Service::Calendar.slug(), "calendar");
    /// ```
    pub fn slug(self) -> &'static str {
        match self {
            Service::Gmail => "gmail",
            Service::Calendar => "calendar",
            Service::Chat => "chat",
            Service::Meet => "meet",
            Service::Drive => "drive",
        }
    }
}

/// Access level for one service. `Write` always implies `Read` (see `scopes::normalize`).
///
/// # Example
///
/// ```
/// use haulmer_connect_core::Level;
/// assert_eq!(serde_json::to_string(&Level::Write).unwrap(), "\"write\"");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Read,
    Write,
}

/// One checkbox of the UI: a service at a level.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::{Level, Service, ServiceLevel};
/// let level = ServiceLevel { service: Service::Gmail, level: Level::Read };
/// assert_eq!(
///     serde_json::to_string(&level).unwrap(),
///     r#"{"service":"gmail","level":"read"}"#
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ServiceLevel {
    pub service: Service,
    pub level: Level,
}

/// A set of granted (or requested) service levels. Serialises as a JSON array of
/// [`ServiceLevel`]. It does not normalise itself; use `scopes::normalize`.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::{Level, Selection, Service};
/// let mut selection = Selection::new();
/// selection.insert(Service::Gmail, Level::Read);
/// assert!(selection.contains(Service::Gmail, Level::Read));
/// assert!(!selection.contains(Service::Gmail, Level::Write));
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Selection(BTreeSet<ServiceLevel>);

impl Selection {
    /// Creates an empty selection.
    ///
    /// # Example
    ///
    /// ```
    /// assert!(haulmer_connect_core::Selection::new().is_empty());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a service level; returns `true` if it was not there yet.
    ///
    /// # Example
    ///
    /// ```
    /// use haulmer_connect_core::{Level, Selection, Service};
    /// let mut selection = Selection::new();
    /// assert!(selection.insert(Service::Drive, Level::Write));
    /// assert!(!selection.insert(Service::Drive, Level::Write));
    /// ```
    pub fn insert(&mut self, service: Service, level: Level) -> bool {
        self.0.insert(ServiceLevel { service, level })
    }

    /// Removes a service level; returns `true` if it was there.
    ///
    /// # Example
    ///
    /// ```
    /// use haulmer_connect_core::{Level, Selection, Service};
    /// let mut selection = Selection::from_levels([(Service::Chat, Level::Read)]);
    /// assert!(selection.remove(Service::Chat, Level::Read));
    /// assert!(selection.is_empty());
    /// ```
    pub fn remove(&mut self, service: Service, level: Level) -> bool {
        self.0.remove(&ServiceLevel { service, level })
    }

    /// Whether the selection holds this service level.
    ///
    /// # Example
    ///
    /// ```
    /// use haulmer_connect_core::{Level, Selection, Service};
    /// let selection = Selection::from_levels([(Service::Meet, Level::Read)]);
    /// assert!(selection.contains(Service::Meet, Level::Read));
    /// ```
    pub fn contains(&self, service: Service, level: Level) -> bool {
        self.0.contains(&ServiceLevel { service, level })
    }

    /// Whether nothing is selected.
    ///
    /// # Example
    ///
    /// ```
    /// assert!(haulmer_connect_core::Selection::new().is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates the service levels in a stable order (by service, then level).
    ///
    /// # Example
    ///
    /// ```
    /// use haulmer_connect_core::{Level, Selection, Service};
    /// let selection = Selection::from_levels([(Service::Drive, Level::Read), (Service::Gmail, Level::Read)]);
    /// let services: Vec<Service> = selection.iter().map(|level| level.service).collect();
    /// assert_eq!(services, [Service::Gmail, Service::Drive]);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = ServiceLevel> + '_ {
        self.0.iter().copied()
    }

    /// Builds a selection from `(service, level)` pairs.
    ///
    /// # Example
    ///
    /// ```
    /// use haulmer_connect_core::{Level, Selection, Service};
    /// let selection = Selection::from_levels([(Service::Gmail, Level::Read), (Service::Gmail, Level::Write)]);
    /// assert_eq!(selection.iter().count(), 2);
    /// ```
    pub fn from_levels(levels: impl IntoIterator<Item = (Service, Level)>) -> Self {
        Self(
            levels
                .into_iter()
                .map(|(service, level)| ServiceLevel { service, level })
                .collect(),
        )
    }
}

/// Where gws keeps its encryption key (`GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND`).
/// `Auto` resolves to `Keyring` for the default config folder and `File` for a custom one.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::KeyStorage;
/// assert_eq!(KeyStorage::default(), KeyStorage::Auto);
/// assert_eq!(serde_json::to_string(&KeyStorage::File).unwrap(), "\"file\"");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyStorage {
    #[default]
    Auto,
    Keyring,
    File,
}

impl KeyStorage {
    /// The value of `GOOGLE_WORKSPACE_CLI_KEYRING_BACKEND` for a resolved setting:
    /// `"file"` for `File`, `"keyring"` otherwise. Resolve `Auto` first with
    /// `Settings::resolved_key_storage`.
    ///
    /// # Example
    ///
    /// ```
    /// use haulmer_connect_core::KeyStorage;
    /// assert_eq!(KeyStorage::File.backend(), "file");
    /// assert_eq!(KeyStorage::Keyring.backend(), "keyring");
    /// ```
    pub fn backend(self) -> &'static str {
        if self == KeyStorage::File {
            "file"
        } else {
            "keyring"
        }
    }
}

/// Folders injected into the core. Tests pass temp dirs; the app passes the real ones.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::Paths;
/// let paths = Paths { home: "/home/ana".into(), app_data: "/home/ana/.local/share/haulmer-connect".into() };
/// assert!(paths.app_data.starts_with(&paths.home));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Paths {
    /// The person's home folder (gws default config is `<home>/.config/gws`; agent skills live here).
    pub home: PathBuf,
    /// The app's own data folder (settings, downloaded gws, run folder).
    pub app_data: PathBuf,
}

/// Persisted app settings (`<app_data>/settings.json`). Behaviour lives in the `settings` module.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::{KeyStorage, Settings};
/// let settings = Settings::default();
/// assert_eq!(settings.config_dir, None);
/// assert_eq!(settings.key_storage, KeyStorage::Auto);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// Custom gws config folder; `None` means the gws default `<home>/.config/gws`.
    pub config_dir: Option<PathBuf>,
    /// Key storage setting as chosen by the person (may be `Auto`).
    pub key_storage: KeyStorage,
}

/// What `Core::status()` reports to the UI.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::{KeyStorage, Selection, Status};
/// let status = Status {
///     gws_available: true,
///     account: None,
///     granted: Selection::new(),
///     token_valid: false,
///     config_dir: "/home/ana/.config/gws".into(),
///     config_dir_is_default: true,
///     key_storage: KeyStorage::Auto,
///     resolved_key_storage: KeyStorage::Keyring,
///     foreign_credentials: false,
/// };
/// assert!(status.account.is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Status {
    /// The pinned gws binary is on the machine (if not, Save downloads it).
    pub gws_available: bool,
    /// Connected Google account email; `None` when not connected.
    pub account: Option<String>,
    /// Levels Google actually granted (from `gws auth status`).
    pub granted: Selection,
    /// gws could refresh the token.
    pub token_valid: bool,
    /// Active gws config folder.
    pub config_dir: PathBuf,
    /// `config_dir` is the gws default folder.
    pub config_dir_is_default: bool,
    /// Key storage setting (may be `Auto`).
    pub key_storage: KeyStorage,
    /// Backend actually passed to gws (`Keyring` or `File`, never `Auto`).
    pub resolved_key_storage: KeyStorage,
    /// Credentials exist in the active folder and were not created by haulmer-connect.
    pub foreign_credentials: bool,
}

/// Result of `Core::save`.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::SaveOutcome;
/// assert_eq!(
///     serde_json::to_string(&SaveOutcome::NeedsReplaceConfirmation).unwrap(),
///     r#"{"kind":"needs_replace_confirmation"}"#
/// );
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "status", rename_all = "snake_case")]
pub enum SaveOutcome {
    /// Login finished; the fresh status.
    Connected(Status),
    /// Foreign gws credentials exist; ask "Replace?" and call `save` again with `confirm_replace = true`.
    NeedsReplaceConfirmation,
}

/// Result of `Core::revoke_all`.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::RevokeOutcome;
/// assert_eq!(
///     serde_json::to_string(&RevokeOutcome::NeedsForeignConfirmation).unwrap(),
///     r#"{"kind":"needs_foreign_confirmation"}"#
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RevokeOutcome {
    /// Access was revoked and local credentials cleared.
    Revoked,
    /// The credentials were created by another app; nothing ran. Ask "Revoke them anyway?"
    /// and call `revoke_all` again with `confirm_foreign = true`.
    NeedsForeignConfirmation,
}

/// Plain-language labels for one service row of the UI (built by `scopes::catalogue`).
///
/// # Example
///
/// ```
/// use haulmer_connect_core::{Service, ServiceInfo};
/// let info = ServiceInfo { service: Service::Gmail, name: "Gmail", read: "Read: read email", write: "Write: send and modify email" };
/// assert_eq!(info.name, "Gmail");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServiceInfo {
    pub service: Service,
    /// Display name, e.g. "Gmail".
    pub name: &'static str,
    /// Description of the Read level.
    pub read: &'static str,
    /// Description of the Write level.
    pub write: &'static str,
}

/// Skills written into one agent skills folder by `Core::install_skills`.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::InstalledSkills;
/// let installed = InstalledSkills {
///     target: "/home/ana/.claude/skills".into(),
///     skills: vec!["gws-shared".into()],
///     skipped: vec!["gws-gmail".into()],
/// };
/// assert_eq!(installed.skills, ["gws-shared"]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledSkills {
    /// The skills folder, e.g. `<home>/.claude/skills`.
    pub target: PathBuf,
    /// Skill folder names written there, e.g. `gws-shared`, `gws-gmail`.
    pub skills: Vec<String>,
    /// Wanted skill folders left untouched because something other than haulmer-connect
    /// put them there.
    pub skipped: Vec<String>,
}

/// Every error the core returns. Serialises as its display message for the UI.
///
/// # Example
///
/// ```
/// use haulmer_connect_core::CoreError;
/// let error = CoreError::Gws { exit_code: 2, message: "No credentials".into() };
/// assert_eq!(error.to_string(), "gws failed (exit 2): No credentials");
/// assert_eq!(serde_json::to_string(&CoreError::Cancelled).unwrap(), "\"Google sign-in was cancelled\"");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CoreError {
    /// A stub that a later ticket implements.
    #[error("not implemented yet: {0}")]
    NotImplemented(&'static str),
    /// A local file operation failed.
    #[error("file error: {0}")]
    Io(String),
    /// A program could not be started or awaited (e.g. not found).
    #[error("could not run a program: {0}")]
    Process(String),
    /// Network failure (no HTTP response at all).
    #[error("network error: {0}")]
    Http(String),
    /// The gws download failed (bad HTTP status, bad archive, missing binary).
    #[error("gws download failed: {0}")]
    Download(String),
    /// A download (gws or the skills tarball) did not match its expected SHA-256.
    #[error("download checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
    /// gws exited non-zero; `message` is `error.message` from its JSON when present.
    #[error("gws failed (exit {exit_code}): {message}")]
    Gws { exit_code: i32, message: String },
    /// Unexpected output (e.g. JSON that does not parse).
    #[error("unexpected output: {0}")]
    Parse(String),
    /// The person cancelled the Google sign-in.
    #[error("Google sign-in was cancelled")]
    Cancelled,
    /// Local credentials were cleared but Google did not confirm the revocation.
    #[error("local credentials were cleared but Google revocation failed: {0}")]
    RevokeFailed(String),
    /// `save` revoked the old access (a level was removed) but the new sign-in was
    /// cancelled or failed: nothing is connected now.
    #[error(
        "Access was removed, but the new Google sign-in did not finish. Save again to reconnect."
    )]
    AccessRemoved,
}

impl From<std::io::Error> for CoreError {
    /// Maps any I/O error to [`CoreError::Io`].
    ///
    /// # Example
    ///
    /// ```
    /// use haulmer_connect_core::CoreError;
    /// let error: CoreError = std::io::Error::other("disk full").into();
    /// assert_eq!(error, CoreError::Io("disk full".into()));
    /// ```
    fn from(error: std::io::Error) -> Self {
        CoreError::Io(error.to_string())
    }
}

impl From<serde_json::Error> for CoreError {
    /// Maps a JSON error to [`CoreError::Parse`].
    ///
    /// # Example
    ///
    /// ```
    /// use haulmer_connect_core::CoreError;
    /// let error: CoreError = serde_json::from_str::<u8>("x").unwrap_err().into();
    /// assert!(matches!(error, CoreError::Parse(_)));
    /// ```
    fn from(error: serde_json::Error) -> Self {
        CoreError::Parse(error.to_string())
    }
}

impl Serialize for CoreError {
    /// Serialises the error as its display message (what the UI shows).
    ///
    /// # Example
    ///
    /// ```
    /// use haulmer_connect_core::CoreError;
    /// assert_eq!(serde_json::to_string(&CoreError::Io("x".into())).unwrap(), "\"file error: x\"");
    /// ```
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
