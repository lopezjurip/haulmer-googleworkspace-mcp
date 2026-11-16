//! T04 — persisted settings (`<app_data>/settings.json`) and their resolution.

use std::path::PathBuf;

use crate::{Core, CoreError, KeyStorage, Paths, Settings};

/// File name of the settings inside `Paths::app_data`.
///
/// # Example
///
/// ```
/// assert_eq!(haulmer_connect_core::settings::SETTINGS_FILE, "settings.json");
/// ```
pub const SETTINGS_FILE: &str = "settings.json";

impl Core {
    /// Reads the persisted settings; defaults when the file does not exist.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{Core, Paths};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// let settings = core.settings().unwrap();
    /// ```
    pub fn settings(&self) -> Result<Settings, CoreError> {
        Err(CoreError::NotImplemented("settings::settings (T04)"))
    }

    /// Sets (or with `None` resets to the gws default) the gws config folder and persists it.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{Core, Paths};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// core.set_config_dir(Some("/h/.config/haulmer-gws".into())).unwrap();
    /// ```
    pub fn set_config_dir(&self, dir: Option<PathBuf>) -> Result<(), CoreError> {
        let _ = dir;
        Err(CoreError::NotImplemented("settings::set_config_dir (T04)"))
    }

    /// Sets the key storage setting and persists it.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use haulmer_connect_core::{Core, KeyStorage, Paths};
    /// # let core = Core::system(Paths { home: "/h".into(), app_data: "/a".into() });
    /// core.set_key_storage(KeyStorage::File).unwrap();
    /// ```
    pub fn set_key_storage(&self, storage: KeyStorage) -> Result<(), CoreError> {
        let _ = storage;
        Err(CoreError::NotImplemented("settings::set_key_storage (T04)"))
    }
}

impl Settings {
    /// The active gws config folder: the custom one, or `<home>/.config/gws`.
    ///
    /// # Example
    ///
    /// ```should_panic
    /// use haulmer_connect_core::{Paths, Settings};
    /// let paths = Paths { home: "/h".into(), app_data: "/a".into() };
    /// assert_eq!(Settings::default().active_config_dir(&paths), std::path::PathBuf::from("/h/.config/gws"));
    /// ```
    pub fn active_config_dir(&self, paths: &Paths) -> PathBuf {
        let _ = paths;
        todo!("T04")
    }

    /// Whether the config folder is the gws default (no custom folder set).
    ///
    /// # Example
    ///
    /// ```should_panic
    /// use haulmer_connect_core::Settings;
    /// assert!(Settings::default().is_default_config_dir());
    /// ```
    pub fn is_default_config_dir(&self) -> bool {
        todo!("T04")
    }

    /// The backend passed to gws: `Auto` -> `Keyring` (default folder) or `File` (custom folder).
    ///
    /// # Example
    ///
    /// ```should_panic
    /// use haulmer_connect_core::{KeyStorage, Settings};
    /// assert_eq!(Settings::default().resolved_key_storage(), KeyStorage::Keyring);
    /// ```
    pub fn resolved_key_storage(&self) -> KeyStorage {
        todo!("T04")
    }
}
